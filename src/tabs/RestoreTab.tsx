import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  History,
  Plus,
  RotateCcw,
  Clock,
  Shield,
} from "lucide-react";
import { RestorePoint } from "../types";
import { useFeedback } from "../hooks/useFeedback";
import { FeedbackBanner } from "../components/FeedbackBanner";

export const RestoreTab: React.FC = () => {
  const [points, setPoints] = useState<RestorePoint[]>([]);
  const { message: feedback, isError, notify } = useFeedback(4000);

  // Modal
  const [showModal, setShowModal] = useState(false);
  const [desc, setDesc] = useState("");

  const loadPoints = async () => {
    try {
      const res = await invoke<RestorePoint[]>("get_restore_points");
      setPoints(res);
    } catch (e: any) {
      notify(`복원 지점 로드 실패: ${e}`, true);
    }
  };

  useEffect(() => {
    loadPoints();
  }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const res = await invoke<RestorePoint>("create_restore_point", {
        description: desc.trim(),
      });
      notify(`새 복원 지점('${res.description}')이 생성되었습니다.`);
      setShowModal(false);
      setDesc("");
      loadPoints();
    } catch (e: any) {
      notify(`복원 지점 생성 실패: ${e}`, true);
    }
  };

  const handleRollback = async (pt: RestorePoint) => {
    if (
      !confirm(
        `'${pt.description}' (${pt.created_at}) 시점의 시스템 트윅 상태로 복원하시겠습니까?`
      )
    )
      return;

    try {
      const res = await invoke<string>("restore_from_point", { id: pt.id });
      notify(res, false);
    } catch (e: any) {
      notify(`롤백 실패: ${e}`, true);
    }
  };

  return (
    <div className="space-y-6">
      {/* Top Banner */}
      <div className="flex items-center justify-between bg-slate-900/60 p-4 rounded-xl border border-slate-800">
        <div>
          <div className="flex items-center gap-2">
            <History className="w-5 h-5 text-emerald-400" />
            <span className="text-sm font-semibold text-slate-200">
              트윅 복원 센터 (1클릭 시스템 롤백)
            </span>
          </div>
          <p className="text-xs text-slate-400 mt-1">
            원하는 시점의 레지스트리 및 트윅 설정 상태로 즉시 되돌릴 수 있습니다.
          </p>
        </div>

        <button
          onClick={() => setShowModal(true)}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold shadow-sm transition"
        >
          <Plus className="w-4 h-4" />
          <span>새 복원 지점 생성</span>
        </button>
      </div>

      {/* Feedback Banner */}
      <FeedbackBanner message={feedback} isError={isError} />

      {/* Points List */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5">
        <h4 className="font-bold text-xs uppercase tracking-wider text-slate-400 mb-4">
          저장된 복원 지점 ({points.length}개)
        </h4>

        <div className="space-y-3">
          {points.length === 0 ? (
            <div className="p-8 text-center text-slate-500 text-xs">
              저장된 복원 지점이 없습니다. 상단 '새 복원 지점 생성' 버튼으로 안전하게 저장해두세요.
            </div>
          ) : (
            points.map((pt) => (
              <div
                key={pt.id}
                className="p-4 rounded-xl bg-slate-950/60 border border-slate-800 flex flex-col md:flex-row md:items-center justify-between gap-4 text-xs hover:border-slate-700 transition"
              >
                <div>
                  <div className="flex items-center gap-2">
                    <Shield className="w-4 h-4 text-emerald-400" />
                    <span className="font-bold text-slate-100 text-sm">{pt.description}</span>
                  </div>
                  <div className="flex items-center gap-4 text-[11px] text-slate-400 mt-1">
                    <span className="flex items-center gap-1 font-mono">
                      <Clock className="w-3 h-3" />
                      {pt.created_at}
                    </span>
                    <span className="text-slate-500">ID: {pt.id}</span>
                  </div>
                </div>

                <div className="flex items-center gap-3">
                  <div className="text-[11px] text-slate-400 hidden lg:block">
                    클래식 메뉴: {pt.shell_state.win11_classic_menu ? "ON" : "OFF"} | 확장자 표시: {pt.shell_state.show_file_extensions ? "ON" : "OFF"}
                  </div>
                  <button
                    onClick={() => handleRollback(pt)}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-emerald-600 text-slate-200 hover:text-white font-semibold transition shadow-sm"
                  >
                    <RotateCcw className="w-3.5 h-3.5" />
                    <span>이 시점으로 롤백</span>
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Modal */}
      {showModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 z-50">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-md p-6 shadow-2xl">
            <h3 className="font-bold text-base text-slate-100 mb-4 flex items-center gap-2">
              <History className="w-5 h-5 text-emerald-400" />
              새 시스템 트윅 복원 지점 생성
            </h3>
            <form onSubmit={handleCreate} className="space-y-4">
              <div>
                <label className="block text-xs font-medium text-slate-400 mb-1">
                  설명 / 메모
                </label>
                <input
                  type="text"
                  value={desc}
                  onChange={(e) => setDesc(e.target.value)}
                  placeholder="예: 윈도우 11 쉘 트윅 일괄 적용 전 백업"
                  className="w-full px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 text-xs focus:outline-none focus:border-emerald-500"
                  required
                />
              </div>
              <div className="flex justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={() => setShowModal(false)}
                  className="px-3.5 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium transition"
                >
                  취소
                </button>
                <button
                  type="submit"
                  className="px-3.5 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold shadow-md transition"
                >
                  생성
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
