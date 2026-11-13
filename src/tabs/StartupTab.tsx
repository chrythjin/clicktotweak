import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Zap,
  Plus,
  Trash2,
  ToggleLeft,
  ToggleRight,
  Folder,
  Database,
} from "lucide-react";
import { StartupItem } from "../types";
import { useFeedback } from "../hooks/useFeedback";
import { FeedbackBanner } from "../components/FeedbackBanner";

export const StartupTab: React.FC = () => {
  const [items, setItems] = useState<StartupItem[]>([]);
  const [loading, setLoading] = useState(false);
  const { message, isError, notify: showMessage } = useFeedback(4000);

  // Add modal state
  const [showAddModal, setShowAddModal] = useState(false);
  const [newName, setNewName] = useState("");
  const [newCommand, setNewCommand] = useState("");

  const loadItems = async () => {
    try {
      setLoading(true);
      const res = await invoke<StartupItem[]>("get_startup_items");
      setItems(res);
    } catch (err: any) {
      showMessage(`시작프로그램 로드 실패: ${err}`, true);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadItems();
  }, []);

  const handleToggle = async (item: StartupItem) => {
    try {
      const nextState = !item.is_enabled;
      const res = await invoke<string>("toggle_startup_item", {
        id: item.id,
        enable: nextState,
      });
      showMessage(res, false);
      loadItems();
    } catch (err: any) {
      showMessage(`상태 변경 실패: ${err}`, true);
    }
  };

  const handleDelete = async (id: string, name: string) => {
    if (!confirm(`'${name}' 시작프로그램을 정말로 삭제하시겠습니까?`)) return;
    try {
      const res = await invoke<string>("delete_startup_item", { id });
      showMessage(res, false);
      loadItems();
    } catch (err: any) {
      showMessage(`삭제 실패: ${err}`, true);
    }
  };

  const handleAddSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim() || !newCommand.trim()) return;

    try {
      const res = await invoke<string>("add_startup_item", {
        name: newName.trim(),
        command: newCommand.trim(),
      });
      showMessage(res, false);
      setShowAddModal(false);
      setNewName("");
      setNewCommand("");
      loadItems();
    } catch (err: any) {
      showMessage(`추가 실패: ${err}`, true);
    }
  };

  const getLocationBadge = (loc: string) => {
    if (loc.startsWith("비활성화됨")) {
      return (
        <span className="inline-flex items-center gap-1 text-[11px] px-2 py-0.5 rounded bg-slate-700/40 text-slate-300 border border-slate-600/40">
          <Database className="w-3 h-3" /> 격리됨
        </span>
      );
    }
    if (loc.includes("HKCU")) {
      return (
        <span className="inline-flex items-center gap-1 text-[11px] px-2 py-0.5 rounded bg-blue-500/10 text-blue-400 border border-blue-500/20">
          <Database className="w-3 h-3" /> HKCU Run
        </span>
      );
    }
    if (loc.includes("HKLM")) {
      return (
        <span className="inline-flex items-center gap-1 text-[11px] px-2 py-0.5 rounded bg-purple-500/10 text-purple-400 border border-purple-500/20">
          <Database className="w-3 h-3" /> HKLM Run
        </span>
      );
    }
    if (loc.includes("Folder")) {
      return (
        <span className="inline-flex items-center gap-1 text-[11px] px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
          <Folder className="w-3 h-3" /> 시작 폴더
        </span>
      );
    }
    return (
      <span className="inline-flex items-center gap-1 text-[11px] px-2 py-0.5 rounded bg-slate-800 text-slate-400">
        {loc}
      </span>
    );
  };

  return (
    <div className="space-y-4">
      {/* Top action bar */}
      <div className="flex items-center justify-between bg-slate-900/60 p-4 rounded-xl border border-slate-800">
        <div className="flex items-center gap-2">
          <Zap className="w-5 h-5 text-amber-400" />
          <span className="text-sm font-semibold text-slate-200">
            총 {items.length}개의 시작프로그램 감지됨
          </span>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowAddModal(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-sm transition"
          >
            <Plus className="w-4 h-4" />
            <span>새 시작프로그램 등록</span>
          </button>
        </div>
      </div>

      {/* Message Banner */}
      <FeedbackBanner message={message} isError={isError} />

      {/* Table Card */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-2xl overflow-hidden shadow-sm">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-slate-950/70 border-b border-slate-800 text-slate-400 font-semibold uppercase tracking-wider">
              <tr>
                <th className="py-3 px-4">상태</th>
                <th className="py-3 px-4">프로그램 이름</th>
                <th className="py-3 px-4">등록 위치</th>
                <th className="py-3 px-4">실행 명령 / 경로</th>
                <th className="py-3 px-4 text-right">작업</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {loading ? (
                <tr>
                  <td colSpan={5} className="py-8 text-center text-slate-500">
                    시작프로그램 목록을 검색하는 중...
                  </td>
                </tr>
              ) : items.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-8 text-center text-slate-500">
                    등록된 시작프로그램이 없습니다.
                  </td>
                </tr>
              ) : (
                items.map((item) => (
                  <tr key={item.id} className="hover:bg-slate-800/40 transition">
                    <td className="py-3 px-4 whitespace-nowrap">
                      {item.id.startsWith("FOLDER_") ? (
                        <span className="flex items-center gap-1 text-[11px] text-slate-500">
                          <Folder className="w-3.5 h-3.5" />
                          <span>폴더 항목</span>
                        </span>
                      ) : (
                        <button
                          onClick={() => handleToggle(item)}
                          title={item.is_enabled ? "비활성화하기" : "활성화하기"}
                          className="flex items-center gap-1.5 focus:outline-none"
                        >
                          {item.is_enabled ? (
                            <span className="flex items-center gap-1 text-emerald-400 font-medium">
                              <ToggleRight className="w-6 h-6 text-emerald-400" />
                              <span>활성</span>
                            </span>
                          ) : (
                            <span className="flex items-center gap-1 text-slate-500 font-medium">
                              <ToggleLeft className="w-6 h-6 text-slate-600" />
                              <span>꺼짐</span>
                            </span>
                          )}
                        </button>
                      )}
                    </td>
                    <td className="py-3 px-4 font-semibold text-slate-100 max-w-[200px] truncate" title={item.name}>
                      {item.name}
                    </td>
                    <td className="py-3 px-4 whitespace-nowrap">
                      {getLocationBadge(item.location)}
                    </td>
                    <td className="py-3 px-4 text-slate-400 max-w-xs truncate font-mono text-[11px]" title={item.command}>
                      {item.command}
                    </td>
                    <td className="py-3 px-4 text-right whitespace-nowrap">
                      <button
                        onClick={() => handleDelete(item.id, item.name)}
                        className="p-1.5 rounded-lg text-slate-400 hover:text-rose-400 hover:bg-rose-500/10 transition"
                        title="시작프로그램 삭제"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Add Modal */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 z-50">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-md p-6 shadow-2xl">
            <h3 className="font-bold text-base text-slate-100 mb-4 flex items-center gap-2">
              <Zap className="w-5 h-5 text-blue-400" />
              새 시작프로그램 등록
            </h3>
            <form onSubmit={handleAddSubmit} className="space-y-4">
              <div>
                <label className="block text-xs font-medium text-slate-400 mb-1">
                  프로그램 이름
                </label>
                <input
                  type="text"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  placeholder="예: MyUtility"
                  className="w-full px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 text-xs focus:outline-none focus:border-blue-500"
                  required
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-slate-400 mb-1">
                  실행 파일 경로 / 명령어
                </label>
                <input
                  type="text"
                  value={newCommand}
                  onChange={(e) => setNewCommand(e.target.value)}
                  placeholder="예: C:\Program Files\App\app.exe"
                  className="w-full px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 text-xs focus:outline-none focus:border-blue-500"
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
                  className="px-3.5 py-2 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-md transition"
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

