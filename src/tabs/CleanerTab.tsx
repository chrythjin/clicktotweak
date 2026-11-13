import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Trash2,
  Search,
  CheckCircle2,
  AlertCircle,
  HardDrive,
  FileX,
  ShieldAlert,
  Database,
  FileLock2,
  Sparkles,
  ShieldCheck,
} from "lucide-react";
import {
  CleanerCategoryScan,
  CleanResult,
  RegistryJunkItem,
  RegistryCleanResult,
} from "../types";
import { formatBytesAdaptive } from "../lib/format";

export const CleanerTab: React.FC = () => {
  const [subTab, setSubTab] = useState<"files" | "registry" | "shredder">("files");

  // =================== 1. 파일 클리너 상태 ===================
  const [categories, setCategories] = useState<CleanerCategoryScan[]>([]);
  const [selected, setSelected] = useState<Record<string, boolean>>({});
  const [fileScanning, setFileScanning] = useState(false);
  const [fileCleaning, setFileCleaning] = useState(false);
  const [fileResult, setFileResult] = useState<CleanResult | null>(null);
  const [fileErrorMsg, setFileErrorMsg] = useState<string | null>(null);
  const [showFileConfirm, setShowFileConfirm] = useState(false);

  // =================== 2. 레지스트리 클리너 상태 ===================
  const [regItems, setRegItems] = useState<RegistryJunkItem[]>([]);
  const [selectedRegKeys, setSelectedRegKeys] = useState<Record<string, boolean>>({});
  const [regScanning, setRegScanning] = useState(false);
  const [regCleaning, setRegCleaning] = useState(false);
  const [regResult, setRegResult] = useState<RegistryCleanResult | null>(null);
  const [regErrorMsg, setRegErrorMsg] = useState<string | null>(null);

  // =================== 3. 파일 영구 파쇄기 상태 ===================
  const [shredPath, setShredPath] = useState("");
  const [shredPasses, setShredPasses] = useState<number>(3);
  const [shredding, setShredding] = useState(false);
  const [shredResult, setShredResult] = useState<string | null>(null);
  const [shredError, setShredError] = useState<string | null>(null);
  const [showShredConfirm, setShowShredConfirm] = useState(false);

  // 1. 파일 클리너 스캔
  const startFileScan = async () => {
    try {
      setFileScanning(true);
      setFileErrorMsg(null);
      setFileResult(null);
      const res = await invoke<CleanerCategoryScan[]>("scan_cleaner_targets");
      setCategories(res);

      const initialSel: Record<string, boolean> = {};
      res.forEach((c) => {
        initialSel[c.category_id] = false;
      });
      setSelected(initialSel);
    } catch (e: any) {
      setFileErrorMsg(`파일 스캔 실패: ${e}`);
    } finally {
      setFileScanning(false);
    }
  };

  // 1. 파일 클리너 삭제 실행
  const executeFileClean = async () => {
    setShowFileConfirm(false);
    const activeCats = Object.entries(selected)
      .filter(([_, isSel]) => isSel)
      .map(([cat]) => cat);

    if (activeCats.length === 0) return;

    try {
      setFileCleaning(true);
      setFileErrorMsg(null);
      const res = await invoke<CleanResult>("clean_selected_targets", {
        categoryIds: activeCats,
      });
      setFileResult(res);
      await startFileScan();
    } catch (e: any) {
      setFileErrorMsg(`청소 실행 실패: ${e}`);
    } finally {
      setFileCleaning(false);
    }
  };

  // 2. 레지스트리 클리너 스캔
  const startRegScan = async () => {
    try {
      setRegScanning(true);
      setRegErrorMsg(null);
      setRegResult(null);
      const res = await invoke<RegistryJunkItem[]>("scan_registry_junk");
      setRegItems(res);

      // 기본 전체 선택
      const initialSel: Record<string, boolean> = {};
      res.forEach((item) => {
        initialSel[item.id] = true;
      });
      setSelectedRegKeys(initialSel);
    } catch (e: any) {
      setRegErrorMsg(`레지스트리 스캔 실패: ${e}`);
    } finally {
      setRegScanning(false);
    }
  };

  // 2. 레지스트리 청소 실행
  const executeRegClean = async () => {
    const selectedList = regItems.filter((i) => selectedRegKeys[i.id]);
    if (selectedList.length === 0) return;

    try {
      setRegCleaning(true);
      setRegErrorMsg(null);
      const res = await invoke<RegistryCleanResult>("clean_registry_junk", {
        itemIds: selectedList.map((item) => item.id),
      });
      setRegResult(res);
      await startRegScan();
    } catch (e: any) {
      setRegErrorMsg(`레지스트리 정리 실패: ${e}`);
    } finally {
      setRegCleaning(false);
    }
  };

  // 3. 파일 영구 파쇄 실행
  const executeShred = async () => {
    if (!shredPath.trim()) {
      setShredError("파쇄할 파일 경로를 입력하세요.");
      return;
    }
    setShowShredConfirm(false);
    try {
      setShredding(true);
      setShredError(null);
      setShredResult(null);
      const res = await invoke<string>("shred_file", {
        filePath: shredPath.trim(),
        passes: shredPasses,
      });
      setShredResult(res);
      setShredPath("");
    } catch (e: any) {
      setShredError(`파쇄 실패: ${e}`);
    } finally {
      setShredding(false);
    }
  };

  useEffect(() => {
    startFileScan();
  }, []);

  const totalFileBytes = categories.reduce((acc, c) => acc + c.total_size_bytes, 0);
  const selectedCategories = categories.filter((c) => selected[c.category_id]);
  const selectedFileBytes = selectedCategories.reduce((acc, c) => acc + c.total_size_bytes, 0);
  const selectedFileCount = selectedCategories.reduce((acc, c) => acc + c.file_count, 0);

  return (
    <div className="space-y-6">
      {/* 서브 탭 네비게이션 */}
      <div className="flex border-b border-slate-800 gap-2">
        <button
          onClick={() => {
            setSubTab("files");
            if (categories.length === 0) startFileScan();
          }}
          className={`pb-3 px-4 font-semibold text-xs flex items-center gap-2 border-b-2 transition ${
            subTab === "files"
              ? "border-blue-500 text-blue-400"
              : "border-transparent text-slate-400 hover:text-slate-200"
          }`}
        >
          <HardDrive className="w-4 h-4" />
          정크 & 임시 파일 청소
        </button>
        <button
          onClick={() => {
            setSubTab("registry");
            if (regItems.length === 0) startRegScan();
          }}
          className={`pb-3 px-4 font-semibold text-xs flex items-center gap-2 border-b-2 transition ${
            subTab === "registry"
              ? "border-blue-500 text-blue-400"
              : "border-transparent text-slate-400 hover:text-slate-200"
          }`}
        >
          <Database className="w-4 h-4" />
          레지스트리 불필요 항목 청소
        </button>
        <button
          onClick={() => setSubTab("shredder")}
          className={`pb-3 px-4 font-semibold text-xs flex items-center gap-2 border-b-2 transition ${
            subTab === "shredder"
              ? "border-blue-500 text-blue-400"
              : "border-transparent text-slate-400 hover:text-slate-200"
          }`}
        >
          <FileLock2 className="w-4 h-4" />
          파일 파쇄기 (덮어쓰기 후 삭제)
        </button>
      </div>

      {/* ========================================================
          서브 탭 1: 정크 & 임시 파일 청소
         ======================================================== */}
      {subTab === "files" && (
        <div className="space-y-6">
          {fileErrorMsg && (
            <div className="p-4 rounded-xl bg-red-950/80 border border-red-500/50 text-red-200 text-xs flex items-center gap-2">
              <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />
              <span>{fileErrorMsg}</span>
            </div>
          )}

          {fileResult && (
            <div className="p-4 rounded-xl bg-emerald-950/80 border border-emerald-500/50 text-emerald-200 text-xs flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
              <div>
                <span className="font-bold">
                  {formatBytesAdaptive(fileResult.freed_bytes)}
                </span>
                의 디스크 공간을 안전하게 확보했습니다! ({fileResult.deleted_files_count}개 파일 제거)
              </div>
            </div>
          )}

          {/* 대시보드 상단 요약 카드 */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
              <div>
                <div className="text-[11px] text-slate-400">발견된 불필요 항목</div>
                <div className="text-xl font-bold text-slate-100 mt-1">
                  {formatBytesAdaptive(totalFileBytes)}
                </div>
              </div>
              <Trash2 className="w-8 h-8 text-slate-700" />
            </div>

            <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
              <div>
                <div className="text-[11px] text-slate-400">선택된 청소 대상</div>
                <div className="text-xl font-bold text-blue-400 mt-1">
                  {formatBytesAdaptive(selectedFileBytes)}
                </div>
                <div className="text-[11px] text-slate-500 mt-0.5">
                  {selectedFileCount.toLocaleString()}개 파일
                </div>
              </div>
              <CheckCircle2 className="w-8 h-8 text-blue-500/30" />
            </div>

            <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 flex items-center gap-2">
              <button
                onClick={startFileScan}
                disabled={fileScanning || fileCleaning}
                className="flex-1 py-2.5 px-3 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold flex items-center justify-center gap-2 transition disabled:opacity-50"
              >
                <Search className={`w-3.5 h-3.5 ${fileScanning ? "animate-spin" : ""}`} />
                <span>{fileScanning ? "스캔 중..." : "다시 스캔"}</span>
              </button>

              <button
                onClick={() => setShowFileConfirm(true)}
                disabled={fileScanning || fileCleaning || selectedFileCount === 0}
                className="flex-1 py-2.5 px-3 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-bold flex items-center justify-center gap-2 shadow-md shadow-blue-600/20 transition disabled:opacity-50"
              >
                <Trash2 className="w-3.5 h-3.5" />
                <span>지금 청소</span>
              </button>
            </div>
          </div>

          {/* 청소 대상 목록 */}
          <div className="bg-slate-900/80 border border-slate-800 rounded-2xl overflow-hidden shadow-sm">
            <div className="p-4 bg-slate-950/40 border-b border-slate-800 flex justify-between items-center text-xs">
              <span className="font-bold text-slate-300">정크 파일 카테고리</span>
              <div className="flex gap-2">
                <button
                  onClick={() => {
                    const allTrue: Record<string, boolean> = {};
                    categories.forEach((c) => (allTrue[c.category_id] = true));
                    setSelected(allTrue);
                  }}
                  className="text-[11px] text-blue-400 hover:underline"
                >
                  모두 선택
                </button>
                <span className="text-slate-600">|</span>
                <button
                  onClick={() => {
                    const allFalse: Record<string, boolean> = {};
                    categories.forEach((c) => (allFalse[c.category_id] = false));
                    setSelected(allFalse);
                  }}
                  className="text-[11px] text-slate-400 hover:underline"
                >
                  모두 해제
                </button>
              </div>
            </div>

            <div className="divide-y divide-slate-800/60">
              {categories.map((cat) => {
                const isSel = selected[cat.category_id] ?? false;
                return (
                  <div
                    key={cat.category_id}
                    onClick={() => setSelected((prev) => ({ ...prev, [cat.category_id]: !prev[cat.category_id] }))}
                    className={`p-4 flex items-center justify-between cursor-pointer transition select-none ${
                      isSel ? "bg-blue-950/20" : "hover:bg-slate-800/30"
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      <input
                        type="checkbox"
                        checked={isSel}
                        onChange={() => {}}
                        className="rounded border-slate-700 bg-slate-800 text-blue-600 focus:ring-0 cursor-pointer"
                      />
                      <div>
                        <div className="text-xs font-bold text-slate-200">{cat.name}</div>
                        <div className="text-[11px] text-slate-400 mt-0.5">{cat.description}</div>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-xs font-bold text-slate-200">
                        {formatBytesAdaptive(cat.total_size_bytes)}
                      </div>
                      <div className="text-[10px] text-slate-500">
                        {cat.file_count.toLocaleString()}개 파일
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* 삭제 확인 모달 */}
          {showFileConfirm && (
            <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
              <div className="bg-slate-900 border border-slate-700 rounded-2xl max-w-md w-full p-6 shadow-2xl space-y-4">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-xl bg-amber-500/20 text-amber-400 flex items-center justify-center">
                    <ShieldAlert className="w-6 h-6" />
                  </div>
                  <div>
                    <h3 className="text-base font-bold text-slate-100">파일 영구 삭제 확인</h3>
                    <p className="text-xs text-slate-400">선택된 불필요 파일을 디스크에서 영구 제거합니다.</p>
                  </div>
                </div>

                <div className="p-3 bg-slate-950/60 rounded-xl text-xs space-y-1">
                  <div className="flex justify-between text-slate-300">
                    <span>선택된 파일 수:</span>
                    <span className="font-bold text-slate-100">{selectedFileCount.toLocaleString()}개</span>
                  </div>
                  <div className="flex justify-between text-slate-300">
                    <span>확보 예상 용량:</span>
                    <span className="font-bold text-blue-400">{formatBytesAdaptive(selectedFileBytes)}</span>
                  </div>
                  <div className="text-[11px] text-slate-500 mt-1">
                    * 임시 파일 및 캐시는 안전하게 삭제되며 개인 문서나 시스템 필수 파일은 보존됩니다.
                  </div>
                </div>

                <div className="flex gap-2 pt-2">
                  <button
                    onClick={() => setShowFileConfirm(false)}
                    className="flex-1 py-2 px-4 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold transition"
                  >
                    취소
                  </button>
                  <button
                    onClick={executeFileClean}
                    className="flex-1 py-2 px-4 rounded-xl bg-red-600 hover:bg-red-500 text-white text-xs font-bold transition shadow-lg shadow-red-600/20"
                  >
                    삭제 진행
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* ========================================================
          서브 탭 2: 레지스트리 불필요 항목 청소
         ======================================================== */}
      {subTab === "registry" && (
        <div className="space-y-6">
          {regErrorMsg && (
            <div className="p-4 rounded-xl bg-red-950/80 border border-red-500/50 text-red-200 text-xs flex items-center gap-2">
              <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />
              <span>{regErrorMsg}</span>
            </div>
          )}

          {regResult && (
            <div className="p-4 rounded-xl bg-emerald-950/80 border border-emerald-500/50 text-emerald-200 text-xs flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
              <div>
                <span className="font-bold">{regResult.cleaned_count}개</span>의 무효한 레지스트리 항목을 성공적으로 정리했습니다!
              </div>
            </div>
          )}

          {/* 대시보드 상단 요약 카드 */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
              <div>
                <div className="text-[11px] text-slate-400">무효/고아 레지스트리</div>
                <div className="text-xl font-bold text-slate-100 mt-1">
                  {regItems.length} 개 발견
                </div>
              </div>
              <Database className="w-8 h-8 text-slate-700" />
            </div>

            <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
              <div>
                <div className="text-[11px] text-slate-400">정리 대상 선택</div>
                <div className="text-xl font-bold text-blue-400 mt-1">
                  {Object.values(selectedRegKeys).filter(Boolean).length} 개
                </div>
              </div>
              <CheckCircle2 className="w-8 h-8 text-blue-500/30" />
            </div>

            <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-4 flex items-center gap-2">
              <button
                onClick={startRegScan}
                disabled={regScanning || regCleaning}
                className="flex-1 py-2.5 px-3 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold flex items-center justify-center gap-2 transition disabled:opacity-50"
              >
                <Search className={`w-3.5 h-3.5 ${regScanning ? "animate-spin" : ""}`} />
                <span>{regScanning ? "스캔 중..." : "정밀 스캔"}</span>
              </button>

              <button
                onClick={executeRegClean}
                disabled={
                  regScanning ||
                  regCleaning ||
                  Object.values(selectedRegKeys).filter(Boolean).length === 0
                }
                className="flex-1 py-2.5 px-3 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-bold flex items-center justify-center gap-2 shadow-md shadow-blue-600/20 transition disabled:opacity-50"
              >
                <Sparkles className="w-3.5 h-3.5" />
                <span>선택 정리</span>
              </button>
            </div>
          </div>

          {/* 레지스트리 항목 목록 */}
          <div className="bg-slate-900/80 border border-slate-800 rounded-2xl overflow-hidden shadow-sm">
            <div className="p-4 bg-slate-950/40 border-b border-slate-800 flex justify-between items-center text-xs">
              <span className="font-bold text-slate-300">
                스캔된 고아/무효 레지스트리 항목 (App Paths, Shared DLLs, MUI Cache)
              </span>
              <div className="flex gap-2">
                <button
                  onClick={() => {
                    const allTrue: Record<string, boolean> = {};
                    regItems.forEach((i) => (allTrue[i.id] = true));
                    setSelectedRegKeys(allTrue);
                  }}
                  className="text-[11px] text-blue-400 hover:underline"
                >
                  모두 선택
                </button>
                <span className="text-slate-600">|</span>
                <button
                  onClick={() => setSelectedRegKeys({})}
                  className="text-[11px] text-slate-400 hover:underline"
                >
                  모두 해제
                </button>
              </div>
            </div>

            {regItems.length === 0 ? (
              <div className="p-12 text-center text-slate-500 text-xs">
                {regScanning
                  ? "레지스트리를 정밀 분석 중입니다..."
                  : "발견된 무효 레지스트리 항목이 없거나 스캔 전입니다."}
              </div>
            ) : (
              <div className="divide-y divide-slate-800/60 max-h-[460px] overflow-y-auto">
                {regItems.map((item) => {
                  const isSel = selectedRegKeys[item.id] ?? false;
                  return (
                    <div
                      key={item.id}
                      onClick={() =>
                        setSelectedRegKeys((prev) => ({
                          ...prev,
                          [item.id]: !prev[item.id],
                        }))
                      }
                      className={`p-3.5 flex items-start gap-3 cursor-pointer transition select-none ${
                        isSel ? "bg-blue-950/15" : "hover:bg-slate-800/30"
                      }`}
                    >
                      <input
                        type="checkbox"
                        checked={isSel}
                        onChange={() => {}}
                        className="rounded border-slate-700 bg-slate-800 text-blue-600 focus:ring-0 cursor-pointer mt-0.5"
                      />
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <span className="text-xs font-bold text-slate-200">
                            {item.value_name}
                          </span>
                          <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 font-medium">
                            {item.category}
                          </span>
                        </div>
                        <div className="text-[11px] text-red-400/90 font-mono truncate mt-0.5">
                          {item.target_path}
                        </div>
                        <div className="text-[10px] text-slate-500 mt-0.5">{item.description}</div>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      )}

      {/* ========================================================
          서브 탭 3: 파일 파쇄기 (덮어쓰기 후 삭제)
         ======================================================== */}
      {subTab === "shredder" && (
        <div className="space-y-6">
          {shredError && (
            <div className="p-4 rounded-xl bg-red-950/80 border border-red-500/50 text-red-200 text-xs flex items-center gap-2">
              <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />
              <span>{shredError}</span>
            </div>
          )}

          {shredResult && (
            <div className="p-4 rounded-xl bg-emerald-950/80 border border-emerald-500/50 text-emerald-200 text-xs flex items-center gap-2">
              <ShieldCheck className="w-4 h-4 text-emerald-400 shrink-0" />
              <span className="font-bold">{shredResult}</span>
            </div>
          )}

          {/* 파쇄기 안내 배너 */}
          <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-6 shadow-sm">
            <div className="flex items-center gap-3 mb-2">
              <div className="w-10 h-10 rounded-xl bg-red-600/20 text-red-400 flex items-center justify-center">
                <FileLock2 className="w-5 h-5" />
              </div>
              <div>
                <h2 className="text-base font-bold text-slate-100">
                  클릭투트윅 파일 파쇄기
                </h2>
                <p className="text-xs text-slate-400">
                  파일 내용을 1~7회 덮어쓴 뒤 삭제합니다. 저장장치 매핑, 파일 시스템, 스냅샷 및 백업에 따라 원본 데이터가 남을 수 있어 복구 불가나 특정 보안 규격 준수를 보장하지 않습니다.
                </p>
              </div>
            </div>
          </div>

          <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-6 space-y-5">
            <div>
              <label className="text-xs font-bold text-slate-200 block mb-2">
                완전 삭제할 파일 경로 (절대 경로)
              </label>
              <input
                type="text"
                value={shredPath}
                onChange={(e) => setShredPath(e.target.value)}
                placeholder="C:\Users\username\Desktop\sensitive_document.docx"
                className="w-full bg-slate-950 border border-slate-700 rounded-xl px-4 py-3 text-xs text-slate-100 font-mono outline-none focus:border-red-500 transition"
              />
              <span className="text-[11px] text-slate-500 mt-1 block">
                * 탐색기에서 파일을 Shift+우클릭 후 '경로로 복사'하여 붙여넣으세요.
              </span>
            </div>

            <div>
              <label className="text-xs font-bold text-slate-200 block mb-2">
                보안 덮어쓰기 알고리즘 단계
              </label>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                {[
                  { passes: 1, title: "1단계 (빠름)", desc: "0x00 제로 필 (기본 파쇄)" },
                  { passes: 3, title: "3단계 (권장)", desc: "0x00 -> 0xFF -> 무작위 난수" },
                  { passes: 7, title: "7단계 (강력)", desc: "7회 반복 덮어쓰기" },
                ].map((opt) => (
                  <div
                    key={opt.passes}
                    onClick={() => setShredPasses(opt.passes)}
                    className={`p-3.5 rounded-xl border cursor-pointer transition select-none ${
                      shredPasses === opt.passes
                        ? "bg-red-950/30 border-red-500/60 text-slate-100"
                        : "bg-slate-950/60 border-slate-800 text-slate-400 hover:border-slate-700"
                    }`}
                  >
                    <div className="text-xs font-bold">{opt.title}</div>
                    <div className="text-[10px] text-slate-400 mt-1">{opt.desc}</div>
                  </div>
                ))}
              </div>
            </div>

            <div className="pt-2">
              <button
                onClick={() => setShowShredConfirm(true)}
                disabled={shredding || !shredPath.trim()}
                className="w-full py-3 rounded-xl bg-red-600 hover:bg-red-500 text-white text-xs font-bold flex items-center justify-center gap-2 shadow-lg shadow-red-600/25 transition disabled:opacity-50"
              >
                <FileX className={`w-4 h-4 ${shredding ? "animate-spin" : ""}`} />
                <span>{shredding ? "보안 덮어쓰기 및 파쇄 진행 중..." : "영구 파쇄 시작 (복구 불가)"}</span>
              </button>
            </div>
          </div>

          {/* 파쇄 경고 모달 */}
          {showShredConfirm && (
            <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
              <div className="bg-slate-900 border border-red-500/50 rounded-2xl max-w-md w-full p-6 shadow-2xl space-y-4">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-xl bg-red-600/20 text-red-400 flex items-center justify-center">
                    <ShieldAlert className="w-6 h-6" />
                  </div>
                  <div>
                    <h3 className="text-base font-bold text-red-200">경고: 영구 파쇄 확인</h3>
                    <p className="text-xs text-slate-400">이 작업은 어떤 포렌식 툴로도 복구할 수 없습니다.</p>
                  </div>
                </div>

                <div className="p-3.5 bg-slate-950 rounded-xl text-xs space-y-2 border border-slate-800">
                  <div className="text-slate-400 text-[11px]">파쇄 대상:</div>
                  <div className="font-mono text-red-300 break-all bg-red-950/30 p-2 rounded border border-red-900/30">
                    {shredPath}
                  </div>
                  <div className="text-[11px] text-slate-400">
                    선택 단계: <span className="font-bold text-slate-200">{shredPasses}회 다중 덮어쓰기</span>
                  </div>
                </div>

                <div className="flex gap-2 pt-2">
                  <button
                    onClick={() => setShowShredConfirm(false)}
                    className="flex-1 py-2 px-4 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold transition"
                  >
                    취소
                  </button>
                  <button
                    onClick={executeShred}
                    className="flex-1 py-2 px-4 rounded-xl bg-red-600 hover:bg-red-500 text-white text-xs font-bold transition shadow-lg shadow-red-600/30"
                  >
                    파쇄 확인 및 실행
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
