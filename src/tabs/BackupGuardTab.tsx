import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ShieldCheck,
  Plus,
  Trash2,
  Camera,
  History,
  RotateCcw,
  CheckCircle2,
  AlertCircle,
  FileText,
  Clock,
} from "lucide-react";
import { GuardTarget, SnapshotInfo } from "../types";

export const BackupGuardTab: React.FC = () => {
  const [targets, setTargets] = useState<GuardTarget[]>([]);
  const [selectedTargetId, setSelectedTargetId] = useState<string | null>(null);
  const [snapshots, setSnapshots] = useState<SnapshotInfo[]>([]);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);

  // New target modal
  const [showAddModal, setShowAddModal] = useState(false);
  const [newFilePath, setNewFilePath] = useState("");
  const [newName, setNewName] = useState("");

  const loadTargets = async () => {
    try {
      const res = await invoke<GuardTarget[]>("get_guard_targets");
      setTargets(res);
      if (res.length > 0 && selectedTargetId === null) {
        setSelectedTargetId(res[0].id);
        loadSnapshots(res[0].id);
      }
    } catch (e: any) {
      notify(`보호 대상 로드 실패: ${e}`, true);
    }
  };

  const loadSnapshots = async (targetId: string) => {
    try {
      const res = await invoke<SnapshotInfo[]>("list_snapshots", { targetId });
      setSnapshots(res);
    } catch (e: any) {
      notify(`스냅샷 목록 조회 실패: ${e}`, true);
    }
  };

  useEffect(() => {
    loadTargets();
  }, []);

  const notify = (msg: string, err = false) => {
    setFeedback(msg);
    setIsError(err);
    setTimeout(() => setFeedback(null), 4000);
  };

  const handleCreateSnapshot = async (id: string) => {
    try {
      const res = await invoke<SnapshotInfo>("create_target_snapshot", { id });
      notify(`새 스냅샷(${res.timestamp})이 안전하게 보관되었습니다.`);
      loadTargets();
      loadSnapshots(id);
    } catch (e: any) {
      notify(`스냅샷 생성 실패: ${e}`, true);
    }
  };

  const handleRestore = async (snap: SnapshotInfo) => {
    const target = targets.find((t) => t.id === snap.target_id);
    if (!target) return;
    if (!confirm(`'${target.name}' 파일을 스냅샷 버전으로 덮어써 복원하시겠습니까?`)) return;

    try {
      const res = await invoke<string>("restore_snapshot", {
        snapshotPath: snap.snapshot_path,
        targetPath: target.path,
      });
      notify(res, false);
    } catch (e: any) {
      notify(`복원 실패: ${e}`, true);
    }
  };

  const handleDeleteTarget = async (id: string) => {
    try {
      const res = await invoke<string>("remove_guard_target", { id });
      notify(res, false);
      loadTargets();
      setSelectedTargetId(null);
      setSnapshots([]);
    } catch (e: any) {
      notify(`대상 제거 실패: ${e}`, true);
    }
  };

  const handleAddSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newFilePath.trim()) return;

    try {
      const res = await invoke<string>("add_guard_target", {
        path: newFilePath.trim(),
        name: newName.trim(),
      });
      notify(res, false);
      setShowAddModal(false);
      setNewFilePath("");
      setNewName("");
      loadTargets();
    } catch (e: any) {
      notify(`추가 실패: ${e}`, true);
    }
  };

  const selectedTarget = targets.find((t) => t.id === selectedTargetId);

  return (
    <div className="space-y-6">
      {/* Top Banner */}
      <div className="flex items-center justify-between bg-slate-900/60 p-4 rounded-xl border border-slate-800">
        <div>
          <div className="flex items-center gap-2">
            <ShieldCheck className="w-5 h-5 text-emerald-400" />
            <span className="text-sm font-semibold text-slate-200">
              유저모드 파일 보호 및 스냅샷 복원
            </span>
          </div>
          <p className="text-xs text-slate-400 mt-1">
            파일의 스냅샷을 생성하고 복원합니다. 기존에 등록된 폴더는 스냅샷을 지원하지 않으므로 삭제 후 파일을 등록해 주세요.
          </p>
        </div>

        <button
          onClick={() => setShowAddModal(true)}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold shadow-sm transition"
        >
          <Plus className="w-4 h-4" />
          <span>보호 파일 등록</span>
        </button>
      </div>

      {/* Feedback Banner */}
      {feedback && (
        <div
          className={`p-3 rounded-xl border text-xs flex items-center gap-2 ${
            isError
              ? "bg-rose-950/40 border-rose-500/40 text-rose-300"
              : "bg-emerald-950/40 border-emerald-500/40 text-emerald-300"
          }`}
        >
          {isError ? (
            <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />
          ) : (
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
          )}
          <span>{feedback}</span>
        </div>
      )}

      {/* Grid: Target List (Left) + Snapshots History (Right) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Targets */}
        <div className="lg:col-span-5 bg-slate-900/80 border border-slate-800 rounded-2xl p-4 flex flex-col h-[480px]">
          <h4 className="font-bold text-xs uppercase tracking-wider text-slate-400 mb-3">
            보호 대상 파일 목록 ({targets.length}개)
          </h4>
          <div className="flex-1 overflow-y-auto space-y-1 pr-1">
            {targets.length === 0 ? (
              <div className="p-8 text-center text-slate-500 text-xs">
                등록된 보호 대상이 없습니다. 상단 '보호 파일 등록'을 눌러 추가하세요.
              </div>
            ) : (
              targets.map((t) => {
                const isSelected = t.id === selectedTargetId;
                return (
                  <div
                    key={t.id}
                    onClick={() => {
                      setSelectedTargetId(t.id);
                      loadSnapshots(t.id);
                    }}
                    className={`p-3 rounded-xl border transition flex items-center justify-between text-xs cursor-pointer ${
                      isSelected
                        ? "bg-emerald-600/20 border-emerald-500/50 text-emerald-200"
                        : "bg-slate-950/40 border-slate-800/80 hover:bg-slate-800/50 text-slate-300"
                    }`}
                  >
                    <div className="truncate pr-2">
                      <div className="font-semibold text-slate-100 truncate flex items-center gap-1.5">
                        <FileText className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                        <span>{t.name}</span>
                        {!t.snapshot_capable && (
                          <span
                            className="inline-flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20 shrink-0"
                            title={
                              t.is_folder
                                ? "폴더는 스냅샷을 지원하지 않습니다. 삭제 후 파일을 등록해 주세요."
                                : "원본 파일이 존재하지 않아 스냅샷을 만들 수 없습니다."
                            }
                          >
                            <AlertCircle className="w-3 h-3" />
                            {t.is_folder ? "폴더 미지원" : "원본 없음"}
                          </span>
                        )}
                      </div>
                      <div className="text-[11px] text-slate-400 truncate mt-0.5 font-mono">
                        {t.path}
                      </div>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteTarget(t.id);
                      }}
                      className="p-1 rounded text-slate-500 hover:text-rose-400 transition"
                      title="대상 삭제"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                );
              })
            )}
          </div>
        </div>

        {/* Right: Snapshots & Controls */}
        <div className="lg:col-span-7 bg-slate-900/80 border border-slate-800 rounded-2xl p-6 flex flex-col justify-between">
          {selectedTarget ? (
            <div className="space-y-6">
              <div className="border-b border-slate-800 pb-4 flex items-center justify-between">
                <div>
                  <h3 className="text-base font-bold text-slate-100 flex items-center gap-2">
                    {selectedTarget.name}
                  </h3>
                  <div className="text-xs text-slate-400 font-mono mt-0.5 truncate max-w-md">
                    {selectedTarget.path}
                  </div>
                </div>
                <button
                  onClick={() => handleCreateSnapshot(selectedTarget.id)}
                  disabled={!selectedTarget.snapshot_capable}
                  title={
                    selectedTarget.snapshot_capable
                      ? "현재 파일 상태를 스냅샷으로 저장합니다."
                      : selectedTarget.is_folder
                        ? "폴더는 스냅샷을 지원하지 않습니다. 삭제 후 파일을 등록해 주세요."
                        : "원본 파일이 존재하지 않아 스냅샷을 만들 수 없습니다."
                  }
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold shadow-sm transition disabled:bg-slate-800 disabled:text-slate-500 disabled:cursor-not-allowed disabled:shadow-none"
                >
                  <Camera className="w-4 h-4" />
                  <span>스냅샷 즉시 생성</span>
                </button>
              </div>

              {!selectedTarget.snapshot_capable && (
                <div className="flex items-start gap-2 rounded-xl border border-amber-500/30 bg-amber-950/20 px-3 py-2 text-[11px] text-amber-300">
                  <AlertCircle className="w-3.5 h-3.5 mt-0.5 shrink-0" />
                  <span>
                    {selectedTarget.is_folder
                      ? "이 항목은 폴더라 스냅샷을 만들 수 없습니다. 폴더를 삭제한 뒤 보호할 파일을 개별 등록해 주세요."
                      : "원본 파일이 없어 스냅샷을 만들 수 없습니다. 경로가 변경되었거나 삭제된 항목입니다."}
                  </span>
                </div>
              )}

              {/* Snapshots Timeline */}
              <div>
                <h4 className="font-bold text-xs uppercase tracking-wider text-slate-400 mb-3 flex items-center gap-1.5">
                  <History className="w-3.5 h-3.5" />
                  저장된 스냅샷 히스토리 ({snapshots.length}개)
                </h4>
                <div className="space-y-2 max-h-[300px] overflow-y-auto pr-1">
                  {snapshots.length === 0 ? (
                    <div className="p-6 text-center text-slate-500 text-xs">
                      아직 생성된 스냅샷이 없습니다. '스냅샷 즉시 생성'을 눌러 백업하세요.
                    </div>
                  ) : (
                    snapshots.map((snap) => (
                      <div
                        key={snap.id}
                        className="p-3 rounded-xl bg-slate-950/60 border border-slate-800 flex items-center justify-between text-xs"
                      >
                        <div className="flex items-center gap-2.5">
                          <Clock className="w-4 h-4 text-emerald-400" />
                          <div>
                            <div className="font-semibold text-slate-200">
                              스냅샷: {snap.id}
                            </div>
                            <div className="text-[10px] text-slate-500 font-mono">
                              크기: {(snap.file_size / 1024).toFixed(1)} KB
                            </div>
                          </div>
                        </div>

                        <button
                          onClick={() => handleRestore(snap)}
                          className="flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-emerald-600 text-slate-300 hover:text-white text-xs font-semibold transition"
                          title="이 버전으로 복원"
                        >
                          <RotateCcw className="w-3 h-3" />
                          <span>복원</span>
                        </button>
                      </div>
                    ))
                  )}
                </div>
              </div>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center text-slate-500 text-xs">
              좌측 목록에서 보호 대상 파일을 선택해주세요.
            </div>
          )}
        </div>
      </div>

      {/* Add Modal */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 z-50">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-md p-6 shadow-2xl">
            <h3 className="font-bold text-base text-slate-100 mb-4 flex items-center gap-2">
              <ShieldCheck className="w-5 h-5 text-emerald-400" />
              보호 대상 파일 등록
            </h3>
            <form onSubmit={handleAddSubmit} className="space-y-4">
              <div>
                <label className="block text-xs font-medium text-slate-400 mb-1">
                  식별 이름
                </label>
                <input
                  type="text"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  placeholder="예: 중요 설정 파일, 게임 세이브"
                  className="w-full px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 text-xs focus:outline-none focus:border-emerald-500"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-slate-400 mb-1">
                  보호 대상 파일 절대 경로
                </label>
                <input
                  type="text"
                  value={newFilePath}
                  onChange={(e) => setNewFilePath(e.target.value)}
                  placeholder="예: C:\Users\user\Documents\important.docx"
                  className="w-full px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 text-xs font-mono focus:outline-none focus:border-emerald-500"
                  required
                />
              </div>
              <div className="flex justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={() => setShowAddModal(false)}
                  className="px-3.5 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium transition"
                >
                  취소
                </button>
                <button
                  type="submit"
                  className="px-3.5 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold shadow-md transition"
                >
                  등록
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
