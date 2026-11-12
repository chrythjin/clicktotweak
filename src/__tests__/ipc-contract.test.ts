import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

/**
 * 프론트엔드 `invoke()` 호출과 Rust 커맨드 등록/시그니처가 어긋나지 않는지 확인한다.
 * 명령어 이름이 바뀌거나 인자 키가 틀리면 런타임에 "Command not found" /
 * "invalid args" 로 실패하므로, 빌드 단계에서 계약을 고정한다.
 */

const PROJECT_ROOT = fileURLToPath(new URL("../../", import.meta.url));
const RUST_SRC_DIR = join(PROJECT_ROOT, "src-tauri", "src");
const FRONTEND_DIR = join(PROJECT_ROOT, "src");

function collectFiles(dir: string, extensions: string[]): string[] {
  const result: string[] = [];
  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    if (statSync(fullPath).isDirectory()) {
      result.push(...collectFiles(fullPath, extensions));
      continue;
    }
    if (extensions.some((ext) => entry.endsWith(ext))) {
      result.push(fullPath);
    }
  }
  return result;
}

function toCamelCase(snakeCase: string): string {
  return snakeCase.replace(/_([a-z0-9])/g, (_match, char: string) =>
    char.toUpperCase()
  );
}

/** `tauri::generate_handler![ ... ]` 에 등록된 커맨드 이름 목록 */
function readRegisteredCommands(): string[] {
  const libRs = readFileSync(join(RUST_SRC_DIR, "lib.rs"), "utf8");
  const match = libRs.match(/generate_handler!\[([\s\S]*?)\]/);
  if (!match) {
    throw new Error("lib.rs 에서 generate_handler! 목록을 찾지 못했습니다.");
  }
  // 주석 줄은 뒤에 오는 커맨드와 쉼표로 구분되지 않으므로 먼저 제거한다.
  const withoutComments = match[1].replace(/^\s*\/\/.*$/gm, "");

  return withoutComments
    .split(",")
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);
}

/** 커맨드 이름 -> Rust 시그니처의 스네이크케이스 인자 이름 목록 */
function readCommandParameters(): Map<string, string[]> {
  const parameters = new Map<string, string[]>();

  for (const file of collectFiles(RUST_SRC_DIR, [".rs"])) {
    const source = readFileSync(file, "utf8");
    const fnRegex = /pub\s+fn\s+([a-z0-9_]+)\s*\(/g;
    let fnMatch: RegExpExecArray | null;
    while ((fnMatch = fnRegex.exec(source)) !== null) {
      const name = fnMatch[1];
      const argsStart = fnMatch.index + fnMatch[0].length;
      const argsText = readBalanced(source, argsStart);
      if (argsText === null) continue;

      const names = argsText
        .split(",")
        .map((arg) => arg.split(":")[0].trim())
        .filter((arg) => /^[a-z0-9_]+$/.test(arg));

      parameters.set(name, names);
    }
  }

  return parameters;
}

interface InvokeCall {
  command: string;
  argumentKeys: string[] | null;
  file: string;
}

/** 프론트엔드 코드에 등장하는 `invoke()` 호출 목록 */
function readInvokeCalls(): InvokeCall[] {
  const calls: InvokeCall[] = [];

  for (const file of collectFiles(FRONTEND_DIR, [".ts", ".tsx"])) {
    if (file.includes("__tests__")) continue;
    const source = readFileSync(file, "utf8");
    const invokeRegex = /invoke(?:<[^>]*>)?\(/g;
    let invokeMatch: RegExpExecArray | null;

    while ((invokeMatch = invokeRegex.exec(source)) !== null) {
      const argsStart = invokeMatch.index + invokeMatch[0].length;
      const argsText = readBalanced(source, argsStart);
      if (argsText === null) continue;

      const nameMatch = argsText.match(/^\s*"([^"]+)"/);
      if (!nameMatch) continue;

      const rest = argsText.slice(nameMatch[0].length);
      const commaIndex = rest.indexOf(",");
      const objectText = commaIndex >= 0 ? rest.slice(commaIndex + 1).trim() : null;

      calls.push({
        command: nameMatch[1],
        argumentKeys: objectText === null ? null : extractObjectKeys(objectText),
        file: file.slice(PROJECT_ROOT.length),
      });
    }
  }

  return calls;
}

/** 여는 괄호 다음부터 닫는 괄호 직전까지의 문자열을 반환 */
function readBalanced(source: string, start: number): string | null {
  let depth = 1;
  let index = start;
  let inString: string | null = null;

  while (index < source.length) {
    const char = source[index];

    if (inString) {
      if (char === "\\") {
        index += 2;
        continue;
      }
      if (char === inString) inString = null;
      index += 1;
      continue;
    }

    if (char === '"' || char === "'" || char === "`") {
      inString = char;
      index += 1;
      continue;
    }
    if (char === "(" || char === "[" || char === "{") depth += 1;
    else if (char === ")" || char === "]" || char === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(start, index);
      }
    }
    index += 1;
  }

  return null;
}

/** 객체 리터럴의 최상위 키만 추출 (중첩 객체/배열/문자열은 무시) */
function extractObjectKeys(argText: string): string[] | null {
  const trimmed = argText.trim();
  if (!trimmed.startsWith("{")) return null;

  const keys: string[] = [];
  let depth = 0;
  let current = "";
  let inString: string | null = null;

  for (let index = 0; index < trimmed.length; index += 1) {
    const char = trimmed[index];

    if (inString) {
      current += char;
      if (char === "\\") {
        current += trimmed[index + 1] ?? "";
        index += 1;
        continue;
      }
      if (char === inString) inString = null;
      continue;
    }

    if (char === '"' || char === "'" || char === "`") {
      inString = char;
      current += char;
      continue;
    }
    if (char === "{" || char === "[" || char === "(") {
      depth += 1;
      if (depth === 1) continue;
    } else if (char === "}" || char === "]" || char === ")") {
      depth -= 1;
      if (depth === 0) break;
    } else if (char === "," && depth === 1) {
      keys.push(current);
      current = "";
      continue;
    }

    current += char;
  }
  keys.push(current);

  return keys
    .map((entry) => entry.split(":")[0].trim())
    .filter((key) => /^[A-Za-z_][A-Za-z0-9_]*$/.test(key));
}

const registeredCommands = readRegisteredCommands();
const commandParameters = readCommandParameters();
const invokeCalls = readInvokeCalls();

describe("Tauri IPC 계약", () => {
  it("프론트엔드에서 호출하는 커맨드가 실제 존재한다", () => {
    expect(invokeCalls.length).toBeGreaterThan(0);

    const unknown = invokeCalls.filter(
      (call) => !registeredCommands.includes(call.command)
    );

    expect(unknown.map((call) => `${call.command} (${call.file})`)).toEqual([]);
  });

  it("인자 키가 Rust 시그니처와 일치한다", () => {
    const mismatches: string[] = [];

    for (const call of invokeCalls) {
      const rustParams = commandParameters.get(call.command);
      if (!rustParams) continue;

      const expected = rustParams.map(toCamelCase).sort();
      const actual = call.argumentKeys === null ? [] : [...call.argumentKeys].sort();

      if (expected.join(",") !== actual.join(",")) {
        mismatches.push(
          `${call.command} (${call.file}): Rust(${expected.join(", ")}) vs JS(${actual.join(", ")})`
        );
      }
    }

    expect(mismatches).toEqual([]);
  });

  it("등록된 커맨드는 모두 Rust에 구현되어 있다", () => {
    const missing = registeredCommands.filter(
      (command) => !commandParameters.has(command)
    );

    expect(missing).toEqual([]);
  });
});
