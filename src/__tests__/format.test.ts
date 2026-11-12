import { describe, expect, it } from "vitest";
import {
  formatBytesAdaptive,
  formatGigabytes,
  formatUptime,
} from "../lib/format";

describe("formatBytesAdaptive", () => {
  it("renders zero as 0 B", () => {
    expect(formatBytesAdaptive(0)).toBe("0 B");
  });

  it("scales to the largest fitting unit", () => {
    expect(formatBytesAdaptive(512)).toBe("512 B");
    expect(formatBytesAdaptive(2048)).toBe("2 KB");
    expect(formatBytesAdaptive(5 * 1024 * 1024)).toBe("5 MB");
    expect(formatBytesAdaptive(3 * 1024 * 1024 * 1024)).toBe("3 GB");
  });

  it("keeps two decimals and drops trailing zeros", () => {
    expect(formatBytesAdaptive(1536)).toBe("1.5 KB");
    expect(formatBytesAdaptive(1024)).toBe("1 KB");
  });

  it("never reports a negative or non finite size", () => {
    expect(formatBytesAdaptive(-1)).toBe("0 B");
    expect(formatBytesAdaptive(Number.NaN)).toBe("0 B");
    expect(formatBytesAdaptive(Number.POSITIVE_INFINITY)).toBe("0 B");
  });
});

describe("formatGigabytes", () => {
  it("always reports one decimal place in GB", () => {
    expect(formatGigabytes(15 * 1024 * 1024 * 1024)).toBe("15.0 GB");
    expect(formatGigabytes(1024 * 1024 * 1024)).toBe("1.0 GB");
    expect(formatGigabytes(0)).toBe("0.0 GB");
  });
});

describe("formatUptime", () => {
  it("reports minutes below one hour", () => {
    expect(formatUptime(0)).toBe("0분");
    expect(formatUptime(59)).toBe("0분");
    expect(formatUptime(61)).toBe("1분");
  });

  it("reports hours and minutes below one day", () => {
    expect(formatUptime(3600)).toBe("1시간 0분");
    expect(formatUptime(3661)).toBe("1시간 1분");
  });

  it("reports days, hours and minutes above one day", () => {
    expect(formatUptime(86400)).toBe("1일 0시간 0분");
    expect(formatUptime(90061)).toBe("1일 1시간 1분");
  });

  it("clamps negative and non finite input", () => {
    expect(formatUptime(-100)).toBe("0분");
    expect(formatUptime(Number.NaN)).toBe("0분");
  });
});
