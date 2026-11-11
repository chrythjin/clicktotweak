import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  AppWindow,
  Crosshair,
  Sliders,
  Pin,
  Eye,
  EyeOff,
  Pause,
  Play,
  Sparkles,
  RefreshCw,
  CheckCircle2,
  AlertCircle,
  Edit2,
} from "lucide-react";
import { WindowInfo } from "../types";

export const WindowToolTab: React.FC = () => {
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [selectedHwnd, setSelectedHwnd] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);

  // Control states for selected window
  const [alpha, setAlpha] = useState<number>(255);
  const [editTitle, setEditTitle] = useState<string>("");

  const loadWindows = async () => {
    try {
      setLoading(true);
      const res = await invoke<WindowInfo[]>("get_open_windows");
      setWindows(res);
      if (res.length > 0 && selectedHwnd === null) {
        setSelectedHwnd(res[0].hwnd);
        setEditTitle(res[0].title);
      }
    } catch (e: any) {
      notify(`창 목록 조회 실패: ${e}`, true);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadWindows();
  }, []);

  const notify = (msg: string, err = false) => {
    setFeedback(msg);
    setIsError(err);
    setTimeout(() => setFeedback(null), 4000);
  };

  const handlePickWindowAtCursor = async () => {
    try {
      notify("커서 아래 창을 획득했습니다.");
      const info = await invoke<WindowInfo>("get_window_at_cursor");
      setSelectedHwnd(info.hwnd);
      setEditTitle(info.title);
      // Ensure it's in the list
      setWindows((prev) => {
        if (!prev.some((w) => w.hwnd === info.hwnd)) {
          return [info, ...prev];
        }
        return prev;
      });
      notify(`'${info.title}' (${info.process_name}) 창이 선택되었습니다.`);
    } catch (e: any) {
      notify(`창 피커 오류: ${e}`, true);
    }
  };

  const selectedWin = windows.find((w) => w.hwnd === selectedHwnd);

  const commitAlpha = async () => {
    if (!selectedHwnd) return;
    try {
      const res = await invoke<string>("set_window_transparency", {
        hwnd: selectedHwnd,
        alpha,
      });
      notify(res, false);
    } catch (e: any) {
      notify(`투명도 변경 실패: ${e}`, true);
    }
  };

  const handleToggleTopmost = async () => {
    if (!selectedWin) return;
    try {
      const next = !selectedWin.is_topmost;
      const res = await invoke<string>("set_window_topmost", {
        hwnd: selectedWin.hwnd,
        enable: next,
      });
      notify(res, false);
      setWindows((prev) =>
        prev.map((w) => (w.hwnd === selectedWin.hwnd ? { ...w, is_topmost: next } : w))
      );
    } catch (e: any) {
      notify(`항상 위 변경 실패: ${e}`, true);
    }
  };

  const handleSaveTitle = async () => {
    if (!selectedHwnd || !editTitle.trim()) return;
    try {
      const res = await invoke<string>("set_window_title", {
        hwnd: selectedHwnd,
        title: editTitle.trim(),
      });
      notify(res, false);
      setWindows((prev) =>
        prev.map((w) => (w.hwnd === selectedHwnd ? { ...w, title: editTitle.trim() } : w))
      );
    } catch (e: any) {
      notify(`창 제목 변경 실패: ${e}`, true);
    }
  };

  const handleToggleVisibility = async () => {
    if (!selectedWin) return;
    try {
      const next = !selectedWin.is_visible;
      const res = await invoke<string>("set_window_visibility", {
        hwnd: selectedWin.hwnd,
        show: next,
      });
      notify(res, false);
      setWindows((prev) =>
        prev.map((w) => (w.hwnd === selectedWin.hwnd ? { ...w, is_visible: next } : w))
      );
    } catch (e: any) {
      notify(`가시성 변경 실패: ${e}`, true);
    }
  };

  const handleSuspendProcess = async () => {
    if (!selectedWin) return;
    try {
      const res = await invoke<string>("suspend_process", { pid: selectedWin.pid });
      notify(res, false);
    } catch (e: any) {
      notify(`프로세스 일시중지 실패: ${e}`, true);
    }
  };

  const handleResumeProcess = async () => {
    if (!selectedWin) return;
    try {
      const res = await invoke<string>("resume_process", { pid: selectedWin.pid });
      notify(res, false);
    } catch (e: any) {
      notify(`프로세스 재개 실패: ${e}`, true);
    }
  };

  const handleTrimMemory = async () => {
    if (!selectedWin) return;
    try {
      const res = await invoke<string>("trim_process_memory", { pid: selectedWin.pid });
      notify(res, false);
    } catch (e: any) {
      notify(`메모리 정리 실패: ${e}`, true);
    }
  };

  return (
    <div className="space-y-6">
      {/* Top Banner */}
      <div className="flex items-center justify-between bg-slate-900/60 p-4 rounded-xl border border-slate-800">
        <div className="flex items-center gap-2">
          <AppWindow className="w-5 h-5 text-indigo-400" />
          <span className="text-sm font-semibold text-slate-200">
            고급 창 속성 및 프로세스 직접 제어
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handlePickWindowAtCursor}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-sm transition"
          >
            <Crosshair className="w-4 h-4" />
            <span>커서 위치 창 획득</span>
          </button>
          <button
            onClick={loadWindows}
            disabled={loading}
            className="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 transition"
            title="창 목록 새로고침"
          >
            <RefreshCw className={`w-4 h-4 ${loading ? "animate-spin" : ""}`} />
          </button>
        </div>
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

      {/* Main Grid: Window List (Left) + Detail Control Panel (Right) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Window List */}
        <div className="lg:col-span-5 bg-slate-900/80 border border-slate-800 rounded-2xl p-4 flex flex-col h-[480px]">
          <h4 className="font-bold text-xs uppercase tracking-wider text-slate-400 mb-3">
            열린 윈도우 목록 ({windows.length}개)
          </h4>
          <div className="flex-1 overflow-y-auto space-y-1 pr-1">
            {windows.map((win) => {
              const isSelected = win.hwnd === selectedHwnd;
              return (
                <button
                  key={win.hwnd}
                  onClick={() => {
                    setSelectedHwnd(win.hwnd);
                    setEditTitle(win.title);
                  }}
                  className={`w-full text-left p-3 rounded-xl border transition flex items-center justify-between text-xs ${
                    isSelected
                      ? "bg-indigo-600/20 border-indigo-500/50 text-indigo-200"
                      : "bg-slate-950/40 border-slate-800/80 hover:bg-slate-800/50 text-slate-300"
                  }`}
                >
                  <div className="truncate pr-2">
                    <div className="font-semibold text-slate-100 truncate flex items-center gap-1.5">
                      {win.title}
                      {!win.is_visible && (
                        <span className="inline-flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 shrink-0">
                          <EyeOff className="w-3 h-3" />
                          숨김
                        </span>
                      )}
                    </div>
                    <div className="text-[11px] text-slate-400 truncate mt-0.5">
                      {win.process_name} (PID: {win.pid})
                    </div>
                  </div>
                  {win.is_topmost && (
                    <span title="항상 위 고정됨">
                      <Pin className="w-3.5 h-3.5 text-amber-400 shrink-0" />
                    </span>
                  )}
                </button>
              );
            })}
          </div>
        </div>

        {/* Right: Window Controls */}
        <div className="lg:col-span-7 bg-slate-900/80 border border-slate-800 rounded-2xl p-6 flex flex-col justify-between">
          {selectedWin ? (
            <div className="space-y-6">
              {/* Selected Window Header */}
              <div className="border-b border-slate-800 pb-4">
                <div className="flex items-center justify-between">
                  <span className="text-[11px] font-semibold text-indigo-400 uppercase tracking-wider">
                    선택된 타겟 윈도우
                  </span>
                  <span className="text-xs font-mono text-slate-400">
                    HWND: 0x{selectedWin.hwnd.toString(16).toUpperCase()}
                  </span>
                </div>
                <h3 className="text-base font-bold text-slate-100 mt-1 truncate">
                  {selectedWin.title}
                </h3>
                <div className="text-xs text-slate-400 mt-0.5">
                  프로세스: <strong>{selectedWin.process_name}</strong> | PID: <strong>{selectedWin.pid}</strong> | 클래스: <code>{selectedWin.class_name}</code>
                </div>
              </div>

              {/* 1. Transparency Slider */}
              <div className="bg-slate-950/60 p-4 rounded-xl border border-slate-800/80">
                <div className="flex items-center justify-between text-xs mb-2">
                  <span className="font-semibold text-slate-200 flex items-center gap-1.5">
                    <Sliders className="w-3.5 h-3.5 text-blue-400" />
                    창 투명도 조절 (0% ~ 100%)
                  </span>
                  <span className="font-bold text-blue-400">
                    {((alpha / 255) * 100).toFixed(0)}%
                  </span>
                </div>
                <input
                  type="range"
                  min="25"
                  max="255"
                  value={alpha}
                  onChange={(e) => setAlpha(Number(e.target.value))}
                  onPointerUp={commitAlpha}
                  onKeyUp={commitAlpha}
                  className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-blue-500"
                />
                <div className="flex justify-between text-[10px] text-slate-500 mt-1">
                  <span>반투명 (10%)</span>
                  <span>기본 불투명 (100%)</span>
                </div>
              </div>

              {/* 2. Window Title Changer */}
              <div className="bg-slate-950/60 p-4 rounded-xl border border-slate-800/80">
                <label className="block text-xs font-semibold text-slate-200 mb-2 flex items-center gap-1.5">
                  <Edit2 className="w-3.5 h-3.5 text-indigo-400" />
                  창 제목(타이틀) 실시간 변경
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={editTitle}
                    onChange={(e) => setEditTitle(e.target.value)}
                    className="flex-1 px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-200 text-xs focus:outline-none focus:border-indigo-500"
                  />
                  <button
                    onClick={handleSaveTitle}
                    className="px-3.5 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold transition"
                  >
                    변경 적용
                  </button>
                </div>
              </div>

              {/* 3. Action Buttons Grid */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-2.5">
                {/* Topmost Toggle */}
                <button
                  onClick={handleToggleTopmost}
                  className={`p-3 rounded-xl border flex flex-col items-center justify-center gap-1.5 text-xs font-medium transition ${
                    selectedWin.is_topmost
                      ? "bg-amber-500/20 border-amber-500/40 text-amber-300"
                      : "bg-slate-950/60 border-slate-800 text-slate-300 hover:bg-slate-800/60"
                  }`}
                >
                  <Pin className="w-4 h-4" />
                  <span>{selectedWin.is_topmost ? "항상 위 해제" : "항상 위 고정"}</span>
                </button>

                {/* Visibility Toggle */}
                <button
                  onClick={handleToggleVisibility}
                  className="p-3 rounded-xl border border-slate-800 bg-slate-950/60 text-slate-300 hover:bg-slate-800/60 flex flex-col items-center justify-center gap-1.5 text-xs font-medium transition"
                >
                  {selectedWin.is_visible ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  <span>{selectedWin.is_visible ? "창 숨기기" : "창 다시 표시"}</span>
                </button>

                {/* Suspend Process */}
                <button
                  onClick={handleSuspendProcess}
                  className="p-3 rounded-xl border border-slate-800 bg-slate-950/60 text-slate-300 hover:bg-slate-800/60 flex flex-col items-center justify-center gap-1.5 text-xs font-medium transition"
                  title="NtSuspendProcess"
                >
                  <Pause className="w-4 h-4 text-amber-400" />
                  <span>프로세스 일시정지</span>
                </button>

                {/* Resume Process */}
                <button
                  onClick={handleResumeProcess}
                  className="p-3 rounded-xl border border-slate-800 bg-slate-950/60 text-slate-300 hover:bg-slate-800/60 flex flex-col items-center justify-center gap-1.5 text-xs font-medium transition"
                  title="NtResumeProcess"
                >
                  <Play className="w-4 h-4 text-emerald-400" />
                  <span>프로세스 재개</span>
                </button>
              </div>

              {/* Memory Trim Action */}
              <button
                onClick={handleTrimMemory}
                className="w-full py-2.5 rounded-xl bg-slate-800 hover:bg-blue-600 text-slate-200 hover:text-white text-xs font-semibold flex items-center justify-center gap-2 border border-slate-700 transition"
              >
                <Sparkles className="w-4 h-4" />
                <span>해당 프로세스 워킹셋 메모리 최적화 (MemRes 현대화)</span>
              </button>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center text-slate-500 text-xs">
              제어할 창을 좌측 목록에서 선택하거나 상단 피커 도구를 사용해주세요.
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
