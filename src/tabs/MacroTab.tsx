import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  PlaySquare,
  Play,
  Square,
  Clock,
  CheckCircle2,
  AlertCircle,
  MousePointer,
  Keyboard,
} from "lucide-react";
import { MacroPreset } from "../types";

interface MacroStatus {
  running: boolean;
  failed_inputs: number;
}

export const MacroTab: React.FC = () => {
  const [presets, setPresets] = useState<MacroPreset[]>([]);
  const [selectedPresetId, setSelectedPresetId] = useState<string | null>(null);
  const [repeatCount, setRepeatCount] = useState<number>(10);
  const [isPlaying, setIsPlaying] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);

  const loadPresets = async () => {
    try {
      const res = await invoke<MacroPreset[]>("get_sample_macros");
      setPresets(res);
      if (res.length > 0 && selectedPresetId === null) {
        setSelectedPresetId(res[0].id);
        setRepeatCount(res[0].repeat_count);
      }
    } catch (e: any) {
      console.error(e);
    }
  };

  useEffect(() => {
    loadPresets();
    invoke<MacroStatus>("get_macro_status")
      .then((status) => setIsPlaying(status.running))
      .catch((error: unknown) => notify(`매크로 상태 확인 실패: ${error}`, true));
  }, []);

  const notify = (msg: string, err = false) => {
    setFeedback(msg);
    setIsError(err);
    setTimeout(() => setFeedback(null), 4000);
  };

  useEffect(() => {
    if (!isPlaying) return;
    const timer = setInterval(async () => {
      try {
        const status = await invoke<MacroStatus>("get_macro_status");
        if (!status.running) {
          setIsPlaying(false);
          if (status.failed_inputs > 0) {
            notify(
              `매크로 실행이 종료되었습니다. (입력 실패 ${status.failed_inputs}건 — 대상 앱의 권한 수준에 따라 차단될 수 있습니다)`,
              true
            );
          } else {
            notify("매크로 실행이 완료되었습니다.", false);
          }
        }
      } catch (e: any) {
        setIsPlaying(false);
        notify(`매크로 상태 확인 실패: ${e}`, true);
      }
    }, 500);
    return () => clearInterval(timer);
  }, [isPlaying]);

  const handlePlay = async () => {
    const preset = presets.find((p) => p.id === selectedPresetId);
    if (!preset) return;

    try {
      const res = await invoke<string>("play_macro", {
        actions: preset.actions,
        repeatCount: Number(repeatCount),
      });
      setIsPlaying(true);
      notify(res, false);
    } catch (e: any) {
      notify(`재생 오류: ${e}`, true);
      setIsPlaying(false);
    }
  };

  const handleStop = async () => {
    try {
      const res = await invoke<string>("stop_macro");
      notify(res, false);
    } catch (e: any) {
      notify(`중지 오류: ${e}`, true);
    }
  };

  const selectedPreset = presets.find((p) => p.id === selectedPresetId);

  return (
    <div className="space-y-6">
      {/* Top Banner */}
      <div className="flex items-center justify-between bg-slate-900/60 p-4 rounded-xl border border-slate-800">
        <div>
          <div className="flex items-center gap-2">
            <PlaySquare className="w-5 h-5 text-amber-400" />
            <span className="text-sm font-semibold text-slate-200">
              마우스 & 키보드 매크로 플레이어 (구 CTT 매크로 현대화)
            </span>
          </div>
          <p className="text-xs text-slate-400 mt-1">
            반복 작업 및 고속 클릭을 지원하며 Windows SendInput API 기반으로 부드럽게 작동합니다.
          </p>
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

      {/* Main Grid: Presets (Left) + Player Controls (Right) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Presets List */}
        <div className="lg:col-span-5 bg-slate-900/80 border border-slate-800 rounded-2xl p-4 flex flex-col h-[400px]">
          <h4 className="font-bold text-xs uppercase tracking-wider text-slate-400 mb-3">
            매크로 프리셋
          </h4>
          <div className="flex-1 overflow-y-auto space-y-2 pr-1">
            {presets.map((p) => {
              const isSelected = p.id === selectedPresetId;
              return (
                <div
                  key={p.id}
                  onClick={() => {
                    setSelectedPresetId(p.id);
                    setRepeatCount(p.repeat_count);
                  }}
                  className={`p-4 rounded-xl border transition cursor-pointer text-xs ${
                    isSelected
                      ? "bg-amber-500/10 border-amber-500/40 text-amber-200"
                      : "bg-slate-950/40 border-slate-800/80 hover:bg-slate-800/50 text-slate-300"
                  }`}
                >
                  <div className="font-bold text-slate-100 text-sm">{p.name}</div>
                  <div className="text-xs text-slate-400 mt-1">{p.description}</div>
                  <div className="mt-2 text-[11px] text-slate-500">
                    액션: {p.actions.length}단계 | 기본 반복: {p.repeat_count}회
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Right: Player Controls */}
        <div className="lg:col-span-7 bg-slate-900/80 border border-slate-800 rounded-2xl p-6 flex flex-col justify-between">
          {selectedPreset ? (
            <div className="space-y-6">
              <div className="border-b border-slate-800 pb-4">
                <span className="text-[11px] font-semibold text-amber-400 uppercase tracking-wider">
                  선택된 매크로
                </span>
                <h3 className="text-base font-bold text-slate-100 mt-1">
                  {selectedPreset.name}
                </h3>
                <p className="text-xs text-slate-400 mt-1">{selectedPreset.description}</p>
              </div>

              {/* Actions Preview */}
              <div className="bg-slate-950/60 p-4 rounded-xl border border-slate-800/80">
                <h5 className="font-semibold text-xs text-slate-300 mb-2">실행 시퀀스</h5>
                <div className="space-y-1.5">
                  {selectedPreset.actions.map((act, idx) => (
                    <div
                      key={idx}
                      className="flex items-center gap-2 text-xs text-slate-400 font-mono"
                    >
                      <span className="text-amber-400">{idx + 1}.</span>
                      {act.action_type === "click" && (
                        <span className="flex items-center gap-1 text-slate-300">
                          <MousePointer className="w-3.5 h-3.5 text-blue-400" />
                          마우스 {act.button === "right" ? "우클릭" : "좌클릭"} (딜레이: {act.delay_ms}ms)
                        </span>
                      )}
                      {act.action_type === "key" && (
                        <span className="flex items-center gap-1 text-slate-300">
                          <Keyboard className="w-3.5 h-3.5 text-indigo-400" />
                          키보드 입력 [{act.key}] (딜레이: {act.delay_ms}ms)
                        </span>
                      )}
                      {act.action_type === "delay" && (
                        <span className="flex items-center gap-1 text-slate-400">
                          <Clock className="w-3.5 h-3.5" />
                          대기 ({act.delay_ms}ms)
                        </span>
                      )}
                    </div>
                  ))}
                </div>
              </div>

              {/* Repeat Count */}
              <div>
                <label className="block text-xs font-semibold text-slate-200 mb-2">
                  반복 실행 횟수
                </label>
                <input
                  type="number"
                  min="1"
                  max="1000"
                  value={repeatCount}
                  onChange={(e) => setRepeatCount(Number(e.target.value))}
                  className="w-32 px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 text-xs font-mono focus:outline-none focus:border-amber-500"
                />
              </div>

              {/* Play & Stop Buttons */}
              <div className="flex gap-3 pt-4">
                <button
                  onClick={handlePlay}
                  disabled={isPlaying}
                  className="flex-1 py-3 rounded-xl bg-gradient-to-r from-amber-600 to-orange-600 hover:from-amber-500 hover:to-orange-500 text-white text-xs font-bold shadow-lg shadow-amber-600/20 flex items-center justify-center gap-2 transition disabled:opacity-50"
                >
                  <Play className="w-4 h-4 fill-current" />
                  <span>매크로 재생 시작</span>
                </button>

                <button
                  onClick={handleStop}
                  className="px-6 py-3 rounded-xl bg-slate-800 hover:bg-rose-600 text-slate-200 hover:text-white text-xs font-bold border border-slate-700 flex items-center justify-center gap-2 transition"
                >
                  <Square className="w-4 h-4 fill-current" />
                  <span>긴급 중지</span>
                </button>
              </div>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center text-slate-500 text-xs">
              좌측에서 매크로 프리셋을 선택해주세요.
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
