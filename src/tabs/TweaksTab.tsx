import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Sliders,
  CheckCircle2,
  AlertCircle,
  RotateCw,
  FolderOpen,
  Sparkles,
  ToggleLeft,
  ToggleRight,
  Eye,
  FileText,
  CornerUpRight,
  HardDrive,
  Search,
  LayoutGrid,
  Zap,
  Save,
  RotateCcw,
} from "lucide-react";
import { ShellTweaksState, DesktopLayoutSnapshot } from "../types";

export const TweaksTab: React.FC = () => {
  const [tweaks, setTweaks] = useState<ShellTweaksState | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);
  const [restartingExplorer, setRestartingExplorer] = useState(false);

  // Animation tweak state
  const [animationsDisabled, setAnimationsDisabled] = useState(false);

  // DesktopFixer states
  const [desktopLayouts, setDesktopLayouts] = useState<DesktopLayoutSnapshot[]>([]);
  const [backupLayoutName, setBackupLayoutName] = useState("");

  // Special folder edit
  const [selectedFolderKey, setSelectedFolderKey] = useState<string>("");
  const [newFolderPath, setNewFolderPath] = useState<string>("");

  const loadTweaks = async () => {
    try {
      const res = await invoke<ShellTweaksState>("get_shell_tweaks_state");
      setTweaks(res);
      if (res.special_folders.length > 0 && !selectedFolderKey) {
        setSelectedFolderKey(res.special_folders[0].key_name);
        setNewFolderPath(res.special_folders[0].current_path);
      }
    } catch (err: any) {
      notify(`트윅 상태 로드 실패: ${err}`, true);
    }
  };

  const loadLayouts = async () => {
    try {
      const res = await invoke<DesktopLayoutSnapshot[]>("list_desktop_icon_layouts");
      setDesktopLayouts(res);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    loadTweaks();
    loadLayouts();
  }, []);

  const notify = (msg: string, err = false) => {
    setFeedback(msg);
    setIsError(err);
    setTimeout(() => setFeedback(null), 4000);
  };

  const handleToggleClassicMenu = async () => {
    if (!tweaks) return;
    try {
      const next = !tweaks.win11_classic_menu;
      const res = await invoke<string>("set_win11_classic_menu", { enable: next });
      notify(res, false);
      loadTweaks();
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    }
  };

  const handleToggleFileExt = async () => {
    if (!tweaks) return;
    try {
      const next = !tweaks.show_file_extensions;
      const res = await invoke<string>("set_show_file_extensions", { show: next });
      notify(res, false);
      loadTweaks();
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    }
  };

  const handleToggleHiddenFiles = async () => {
    if (!tweaks) return;
    try {
      const next = !tweaks.show_hidden_files;
      const res = await invoke<string>("set_show_hidden_files", { show: next });
      notify(res, false);
      loadTweaks();
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    }
  };

  const handleToggleDriveLetters = async () => {
    if (!tweaks) return;
    try {
      const next = !tweaks.show_drive_letters_first;
      const res = await invoke<string>("set_show_drive_letters_first", { show: next });
      notify(res, false);
      loadTweaks();
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    }
  };

  const handleToggleShortcutArrow = async () => {
    if (!tweaks) return;
    try {
      const next = !tweaks.hide_shortcut_arrow;
      const res = await invoke<string>("set_hide_shortcut_arrow", { hide: next });
      notify(res, false);
      loadTweaks();
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    }
  };

  const handleToggleAnimations = async () => {
    try {
      const next = !animationsDisabled;
      const res = await invoke<string>("set_animation_effects", { disable: next });
      setAnimationsDisabled(next);
      notify(res, false);
    } catch (e: any) {
      notify(`애니메이션 설정 실패: ${e}`, true);
    }
  };

  const handleSearchModeChange = async (mode: number) => {
    try {
      const res = await invoke<string>("set_taskbar_search_mode", { mode });
      notify(res, false);
      loadTweaks();
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    }
  };

  const handleRestartExplorer = async () => {
    try {
      setRestartingExplorer(true);
      const res = await invoke<string>("restart_explorer");
      notify(res, false);
    } catch (e: any) {
      notify(`탐색기 재시작 실패: ${e}`, true);
    } finally {
      setRestartingExplorer(false);
    }
  };

  const handleSaveFolderChange = async () => {
    if (!selectedFolderKey || !newFolderPath.trim()) return;
    try {
      const res = await invoke<string>("set_special_folder_path", {
        keyName: selectedFolderKey,
        newPath: newFolderPath.trim(),
      });
      notify(res, false);
      loadTweaks();
    } catch (e: any) {
      notify(`경로 변경 실패: ${e}`, true);
    }
  };

  // DesktopFixer actions
  const handleBackupDesktop = async () => {
    try {
      const snap = await invoke<DesktopLayoutSnapshot>("backup_desktop_icon_layout", {
        name: backupLayoutName.trim(),
      });
      notify(`바탕화면 아이콘 배치('${snap.name}')가 안전하게 저장되었습니다.`);
      setBackupLayoutName("");
      loadLayouts();
    } catch (e: any) {
      notify(`배치 저장 실패: ${e}`, true);
    }
  };

  const handleRestoreDesktop = async (id: string) => {
    try {
      const res = await invoke<string>("restore_desktop_icon_layout", { id });
      notify(res, false);
    } catch (e: any) {
      notify(`배치 복원 실패: ${e}`, true);
    }
  };

  const tweakCards = [
    {
      title: "Windows 11 클래식 우클릭 메뉴",
      desc: "우클릭 시 '더 많은 옵션 표시(Shift+F10)'를 거치지 않고 전통적인 풀 메뉴를 즉시 엽니다.",
      active: tweaks?.win11_classic_menu ?? false,
      toggle: handleToggleClassicMenu,
      icon: Sparkles,
      color: "text-blue-400",
    },
    {
      title: "알려진 파일 확장자 항상 표시",
      desc: "확장자 변조 악성코드 차단 및 파일 식별을 위해 .exe, .zip 등의 확장자를 항상 노출합니다.",
      active: tweaks?.show_file_extensions ?? false,
      toggle: handleToggleFileExt,
      icon: FileText,
      color: "text-indigo-400",
    },
    {
      title: "숨김 파일 및 폴더 표시",
      desc: "Windows 탐색기에서 시스템 숨김 폴더(AppData, .git 등)를 기본적으로 화면에 표시합니다.",
      active: tweaks?.show_hidden_files ?? false,
      toggle: handleToggleHiddenFiles,
      icon: Eye,
      color: "text-cyan-400",
    },
    {
      title: "내 PC 드라이브 문자 우선 표시",
      desc: "'로컬 디스크 (C:)' 대신 '(C:) 로컬 디스크' 형태로 표시하여 드라이브를 빠르게 식별합니다.",
      active: tweaks?.show_drive_letters_first ?? false,
      toggle: handleToggleDriveLetters,
      icon: HardDrive,
      color: "text-emerald-400",
    },
    {
      title: "바탕화면 바로가기 화살표 아이콘 제거",
      desc: "바탕화면 아이콘 좌측 하단의 지저분한 화살표 오버레이를 제거하여 깔끔하게 정리합니다.",
      active: tweaks?.hide_shortcut_arrow ?? false,
      toggle: handleToggleShortcutArrow,
      icon: CornerUpRight,
      color: "text-amber-400",
    },
    {
      title: "창 최소화/최대화 시각 애니메이션 끄기",
      desc: "불필요한 창 움직임 딜레이를 제거하여 창 전환 및 반응 속도를 즉각적으로 최적화합니다.",
      active: animationsDisabled,
      toggle: handleToggleAnimations,
      icon: Zap,
      color: "text-rose-400",
    },
  ];

  return (
    <div className="space-y-6">
      {/* Action banner */}
      <div className="flex items-center justify-between bg-slate-900/60 p-4 rounded-xl border border-slate-800">
        <div className="flex items-center gap-2">
          <Sliders className="w-5 h-5 text-blue-400" />
          <span className="text-sm font-semibold text-slate-200">
            Windows 11 인터페이스 및 쉘 시스템 트윅
          </span>
        </div>
        <button
          onClick={handleRestartExplorer}
          disabled={restartingExplorer}
          className="flex items-center gap-2 px-3.5 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold border border-slate-700 transition"
        >
          <RotateCw className={`w-3.5 h-3.5 ${restartingExplorer ? "animate-spin" : ""}`} />
          <span>탐색기(explorer.exe) 즉시 재시작</span>
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

      {/* Tweaks Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {tweakCards.map((card, idx) => {
          const Icon = card.icon;
          return (
            <div
              key={idx}
              className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 flex flex-col justify-between hover:border-slate-700 transition shadow-sm"
            >
              <div>
                <div className="flex items-center justify-between mb-3">
                  <div className={`p-2 rounded-xl bg-slate-800/80 ${card.color}`}>
                    <Icon className="w-5 h-5" />
                  </div>
                  <button
                    onClick={card.toggle}
                    className="focus:outline-none cursor-pointer"
                    title={card.active ? "비활성화" : "활성화"}
                  >
                    {card.active ? (
                      <ToggleRight className="w-8 h-8 text-blue-500 transition" />
                    ) : (
                      <ToggleLeft className="w-8 h-8 text-slate-600 transition" />
                    )}
                  </button>
                </div>
                <h4 className="font-bold text-slate-100 text-sm mb-1.5">{card.title}</h4>
                <p className="text-xs text-slate-400 leading-relaxed">{card.desc}</p>
              </div>

              <div className="mt-4 pt-3 border-t border-slate-800/60 flex items-center justify-between text-[11px]">
                <span className="text-slate-500">현재 상태:</span>
                <span className={card.active ? "text-blue-400 font-semibold" : "text-slate-500"}>
                  {card.active ? "적용됨" : "기본값"}
                </span>
              </div>
            </div>
          );
        })}

        {/* Taskbar Search Mode Card */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 flex flex-col justify-between hover:border-slate-700 transition shadow-sm">
          <div>
            <div className="flex items-center gap-2 mb-3">
              <div className="p-2 rounded-xl bg-slate-800/80 text-cyan-400">
                <Search className="w-5 h-5" />
              </div>
              <h4 className="font-bold text-slate-100 text-sm">작업표시줄 검색 모드</h4>
            </div>
            <p className="text-xs text-slate-400 leading-relaxed mb-4">
              작업표시줄 공간 확보를 위해 검색창 표시 형식을 조정합니다.
            </p>

            <div className="space-y-2">
              {[
                { mode: 0, label: "검색 아이콘 숨김" },
                { mode: 1, label: "검색 아이콘만 표시" },
                { mode: 2, label: "검색창 전체 표시" },
              ].map((opt) => (
                <label
                  key={opt.mode}
                  className="flex items-center gap-2.5 text-xs text-slate-300 cursor-pointer p-1.5 rounded-lg hover:bg-slate-800/50"
                >
                  <input
                    type="radio"
                    name="taskbarSearch"
                    checked={tweaks?.taskbar_search_mode === opt.mode}
                    onChange={() => handleSearchModeChange(opt.mode)}
                    className="accent-blue-500 cursor-pointer"
                  />
                  <span>{opt.label}</span>
                </label>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* DesktopFixer: 바탕화면 아이콘 위치 저장 및 복구 카드 */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 shadow-sm">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <LayoutGrid className="w-5 h-5 text-emerald-400" />
            <h3 className="font-semibold text-sm text-slate-200">
              바탕화면 아이콘 위치 저장 & 1클릭 복구 (DesktopFixer 현대화)
            </h3>
          </div>
          <span className="text-[11px] text-slate-400">
            해상도 변경/모니터 연결 시 흩어진 아이콘 원상 복구
          </span>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-12 gap-4 mt-4">
          <div className="lg:col-span-5 bg-slate-950/60 p-4 rounded-xl border border-slate-800/80">
            <label className="block text-xs font-medium text-slate-300 mb-1.5">
              새로운 아이콘 배치 스냅샷 저장
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={backupLayoutName}
                onChange={(e) => setBackupLayoutName(e.target.value)}
                placeholder="예: 4K 듀얼모니터 배치"
                className="flex-1 px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-200 text-xs focus:outline-none focus:border-emerald-500"
              />
              <button
                onClick={handleBackupDesktop}
                className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold shadow-sm transition"
              >
                <Save className="w-3.5 h-3.5" />
                <span>저장</span>
              </button>
            </div>
          </div>

          <div className="lg:col-span-7 bg-slate-950/60 p-4 rounded-xl border border-slate-800/80">
            <label className="block text-xs font-medium text-slate-300 mb-2">
              저장된 배치 목록 ({desktopLayouts.length}개)
            </label>
            <div className="space-y-1.5 max-h-32 overflow-y-auto pr-1">
              {desktopLayouts.length === 0 ? (
                <div className="text-slate-500 text-xs text-center py-2">
                  저장된 배치가 없습니다. 좌측에서 현재 배치를 저장해보세요.
                </div>
              ) : (
                desktopLayouts.map((l) => (
                  <div
                    key={l.id}
                    className="flex items-center justify-between p-2 rounded-lg bg-slate-900 border border-slate-800 text-xs"
                  >
                    <div>
                      <span className="font-semibold text-slate-200">{l.name}</span>
                      <span className="text-[10px] text-slate-500 ml-2 font-mono">
                        ({l.created_at})
                      </span>
                    </div>
                    <button
                      onClick={() => handleRestoreDesktop(l.id)}
                      className="flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-emerald-600 text-slate-300 hover:text-white text-xs font-semibold transition"
                    >
                      <RotateCcw className="w-3 h-3" />
                      <span>복구</span>
                    </button>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Special Folder Relocation Card */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5">
        <div className="flex items-center gap-2 mb-4">
          <FolderOpen className="w-5 h-5 text-indigo-400" />
          <h3 className="font-semibold text-sm text-slate-200">
            사용자 특수 폴더 경로 이동 (User Shell Folders)
          </h3>
        </div>
        <p className="text-xs text-slate-400 mb-4 leading-relaxed">
          C 드라이브 용량 부족을 방지하기 위해 다운로드, 바탕화면, 문서 폴더의 저장 위치를 D 드라이브 등으로 안전하게 변경할 수 있습니다.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1">
              대상 폴더 선택
            </label>
            <select
              value={selectedFolderKey}
              onChange={(e) => {
                const key = e.target.value;
                setSelectedFolderKey(key);
                const item = tweaks?.special_folders.find((f) => f.key_name === key);
                if (item) setNewFolderPath(item.current_path);
              }}
              className="w-full px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 text-xs focus:outline-none focus:border-blue-500"
            >
              {tweaks?.special_folders.map((f) => (
                <option key={f.key_name} value={f.key_name}>
                  {f.display_name} ({f.current_path})
                </option>
              ))}
            </select>
          </div>

          <div className="md:col-span-2">
            <label className="block text-xs font-medium text-slate-400 mb-1">
              새로운 저장 위치 경로
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={newFolderPath}
                onChange={(e) => setNewFolderPath(e.target.value)}
                placeholder="예: D:\Downloads"
                className="flex-1 px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 text-slate-200 text-xs font-mono focus:outline-none focus:border-blue-500"
              />
              <button
                onClick={handleSaveFolderChange}
                className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-sm transition"
              >
                경로 변경 적용
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
