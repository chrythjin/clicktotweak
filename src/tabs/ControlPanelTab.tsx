import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Sliders,
  Monitor,
  MousePointer,
  Keyboard,
  BatteryCharging,
  Layout,
  ExternalLink,
  HardDrive,
  Cpu,
  Sparkles,
  ToggleLeft,
  ToggleRight,
  Shield,
  Folder,
  Network,
  Trash,
  Clock,
  Power,
  RotateCcw,
  Moon,
  AlignLeft,
  AlignCenter,
} from "lucide-react";
import {
  DesktopSystemIcons,
  InputResponseSettings,
  SystemPowerSettings,
  TaskbarSettings,
  AdvancedWin11Tweaks,
} from "../types";
import { useFeedback } from "../hooks/useFeedback";
import { FeedbackBanner } from "../components/FeedbackBanner";

export const ControlPanelTab: React.FC = () => {
  const [icons, setIcons] = useState<DesktopSystemIcons | null>(null);
  const [inputSettings, setInputSettings] = useState<InputResponseSettings | null>(null);
  const [powerSettings, setPowerSettings] = useState<SystemPowerSettings | null>(null);
  const [taskbarSettings, setTaskbarSettings] = useState<TaskbarSettings | null>(null);
  const [win11Tweaks, setWin11Tweaks] = useState<AdvancedWin11Tweaks | null>(null);

  // 예약 종료 상태
  const [shutdownAction, setShutdownAction] = useState<string>("shutdown");
  const [shutdownMinutes, setShutdownMinutes] = useState<number>(60);
  const [customMinutes, setCustomMinutes] = useState<string>("");
  const [shutdownScheduled, setShutdownScheduled] = useState<string | null>(null);

  const { message: feedback, isError, notify } = useFeedback(4500);

  const loadAll = async () => {
    try {
      const [ic, inp, pwr, tb, w11] = await Promise.all([
        invoke<DesktopSystemIcons>("get_desktop_system_icons"),
        invoke<InputResponseSettings>("get_input_response_settings"),
        invoke<SystemPowerSettings>("get_system_power_settings"),
        invoke<TaskbarSettings>("get_taskbar_settings"),
        invoke<AdvancedWin11Tweaks>("get_advanced_win11_tweaks"),
      ]);
      setIcons(ic);
      setInputSettings(inp);
      setPowerSettings(pwr);
      setTaskbarSettings(tb);
      setWin11Tweaks(w11);
    } catch (e: any) {
      notify(`설정 로드 실패: ${e}`, true);
    }
  };

  useEffect(() => {
    loadAll();
  }, []);

  // 1. 바탕화면 아이콘 토글
  const handleToggleIcon = async (iconType: string, currentShow: boolean) => {
    try {
      const nextShow = !currentShow;
      const res = await invoke<string>("set_desktop_system_icon", {
        iconType,
        show: nextShow,
      });
      notify(res, false);
      const refreshed = await invoke<DesktopSystemIcons>("get_desktop_system_icons");
      setIcons(refreshed);
    } catch (e: any) {
      notify(`아이콘 변경 실패: ${e}`, true);
    }
  };

  // 2. 키보드 & 마우스 반응속도 저장
  const handleSaveInputSettings = async () => {
    if (!inputSettings) return;
    try {
      const res = await invoke<string>("set_input_response_settings", {
        settings: inputSettings,
      });
      notify(res, false);
    } catch (e: any) {
      notify(`저장 실패: ${e}`, true);
    }
  };

  // 3. 최대 절전 모드 토글
  const handleToggleHibernate = async () => {
    if (!powerSettings) return;
    try {
      const next = !powerSettings.hibernate_enabled;
      const res = await invoke<string>("set_hibernate_enabled", { enable: next });
      notify(res, false);
      const pwr = await invoke<SystemPowerSettings>("get_system_power_settings");
      setPowerSettings(pwr);
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    }
  };

  // 4. 최고의 성능 잠금 해제
  const handleUnlockUltimatePower = async () => {
    try {
      const res = await invoke<string>("unlock_ultimate_performance");
      notify(res, false);
      const pwr = await invoke<SystemPowerSettings>("get_system_power_settings");
      setPowerSettings(pwr);
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    }
  };

  // 5. 작업표시줄 설정 변경 (검색상자, 위젯, 작업보기)
  const handleChangeTaskbar = async (key: string, val: number) => {
    try {
      const res = await invoke<string>("set_taskbar_setting", { key, value: val });
      notify(res, false);
      const tb = await invoke<TaskbarSettings>("get_taskbar_settings");
      setTaskbarSettings(tb);
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    }
  };

  // 6. Windows 11 고급 트윅 토글
  const handleToggleWin11Tweak = async (key: string, nextVal: boolean) => {
    try {
      const res = await invoke<string>("set_advanced_win11_tweak", {
        key,
        enable: nextVal,
      });
      notify(res, false);
      const refreshed = await invoke<AdvancedWin11Tweaks>("get_advanced_win11_tweaks");
      setWin11Tweaks(refreshed);
    } catch (e: any) {
      notify(`트윅 적용 오류: ${e}`, true);
    }
  };

  // 7. 제어판 애플릿 실행
  const handleLaunchApplet = async (appletId: string) => {
    try {
      const res = await invoke<string>("launch_control_panel_applet", { applet: appletId });
      notify(res, false);
    } catch (e: any) {
      notify(`실행 오류: ${e}`, true);
    }
  };

  // 8. 예약 종료 스케줄 실행
  const handleScheduleShutdown = async () => {
    const mins = customMinutes ? parseInt(customMinutes, 10) : shutdownMinutes;
    if (isNaN(mins) || mins <= 0 || mins > 5_256_000) {
      notify("예약 시간은 1분에서 5,256,000분(10년) 사이로 지정해주세요.", true);
      return;
    }
    try {
      const res = await invoke<string>("schedule_shutdown", {
        seconds: mins * 60,
        action: shutdownAction,
      });
      setShutdownScheduled(`${mins}분 후 ${shutdownAction === "shutdown" ? "종료" : shutdownAction === "restart" ? "다시 시작" : "최대 절전"}`);
      notify(res, false);
    } catch (e: any) {
      notify(`예약 실패: ${e}`, true);
    }
  };

  // 9. 예약 종료 취소
  const handleCancelShutdown = async () => {
    try {
      const res = await invoke<string>("cancel_shutdown");
      setShutdownScheduled(null);
      notify(res, false);
    } catch (e: any) {
      notify(`취소 실패: ${e}`, true);
    }
  };

  const applets = [
    { id: "control", name: "클래식 제어판", desc: "전체 제어판 애플릿", icon: Sliders },
    { id: "godmode", name: "갓모드 (GodMode)", desc: "200개 이상 통합 제어판", icon: Sparkles, color: "text-amber-400" },
    { id: "sysdm", name: "시스템 속성", desc: "환경변수, 고급 시스템 설정", icon: Cpu },
    { id: "appwiz", name: "프로그램 및 기능", desc: "설치된 앱 제거 및 변경", icon: Layout },
    { id: "ncpa", name: "네트워크 어댑터", desc: "이더넷 / Wi-Fi 속성", icon: Network },
    { id: "devmgmt", name: "장치 관리자", desc: "하드웨어 드라이버 관리", icon: Monitor },
    { id: "diskmgmt", name: "디스크 관리", desc: "파티션 및 볼륨 할당", icon: HardDrive },
    { id: "services", name: "서비스 관리자", desc: "백그라운드 서비스 시작/중지", icon: Sliders },
    { id: "compmgmt", name: "컴퓨터 관리", desc: "이벤트 뷰어 및 성능 모니터", icon: Cpu },
    { id: "gpedit", name: "로컬 그룹 정책", desc: "시스템 보안 및 관리 정책", icon: Shield },
    { id: "regedit", name: "레지스트리 편집기", desc: "시스템 구성 데이터베이스", icon: Sliders },
    { id: "powercfg", name: "전원 옵션", desc: "절전 및 고성능 플랜", icon: BatteryCharging },
  ];

  return (
    <div className="space-y-6">
      {/* Toast Feedback */}
      <FeedbackBanner message={feedback} isError={isError} />

      {/* Header Banner */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-6 shadow-sm">
        <div className="flex items-center gap-3 mb-2">
          <div className="w-10 h-10 rounded-xl bg-blue-600/20 text-blue-400 flex items-center justify-center">
            <Sliders className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-lg font-bold text-slate-100">제어판 & 시스템 도구 센터</h2>
            <p className="text-xs text-slate-400">
              클릭투트윅 클래식 제어판 설정, 바탕화면 시스템 아이콘, 마우스/키보드 반응 속도, Windows 11 고급 트윅 및 자동 예약 종료를 통합 제어합니다.
            </p>
          </div>
        </div>
      </div>

      {/* Row 1: 바탕화면 시스템 아이콘 & 마우스/키보드 속도 */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Card 1: 바탕화면 시스템 아이콘 */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-2 mb-4">
              <Monitor className="w-4 h-4 text-blue-400" />
              <h3 className="font-bold text-sm text-slate-100">바탕화면 시스템 아이콘 표시</h3>
            </div>
            <p className="text-xs text-slate-400 mb-4 leading-relaxed">
              Windows 11에서 숨겨진 클래식 시스템 아이콘(내 PC, 휴지통, 제어판 등)을 바탕화면에 꺼내거나 숨깁니다.
            </p>

            <div className="space-y-2.5">
              {[
                { key: "this_pc", label: "내 PC (컴퓨터)", icon: Monitor, show: icons?.this_pc ?? false },
                { key: "recycle_bin", label: "휴지통", icon: Trash, show: icons?.recycle_bin ?? true },
                { key: "control_panel", label: "제어판", icon: Sliders, show: icons?.control_panel ?? false },
                { key: "user_files", label: "사용자 파일 (문서)", icon: Folder, show: icons?.user_files ?? false },
                { key: "network", label: "네트워크", icon: Network, show: icons?.network ?? false },
              ].map((item) => {
                const Icon = item.icon;
                return (
                  <div
                    key={item.key}
                    onClick={() => handleToggleIcon(item.key, item.show)}
                    className="flex items-center justify-between p-3 rounded-xl bg-slate-950/60 border border-slate-800 hover:border-slate-700 cursor-pointer transition select-none"
                  >
                    <div className="flex items-center gap-3">
                      <Icon className="w-4 h-4 text-slate-400" />
                      <span className="text-xs font-semibold text-slate-200">{item.label}</span>
                    </div>
                    {item.show ? (
                      <span className="flex items-center gap-1.5 text-xs font-bold text-blue-400">
                        <ToggleRight className="w-5 h-5 text-blue-500" />
                        <span>표시됨</span>
                      </span>
                    ) : (
                      <span className="flex items-center gap-1.5 text-xs text-slate-500">
                        <ToggleLeft className="w-5 h-5 text-slate-600" />
                        <span>숨김</span>
                      </span>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        </div>

        {/* Card 2: 마우스 & 키보드 반응 속도 */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-2 mb-4">
              <Keyboard className="w-4 h-4 text-indigo-400" />
              <h3 className="font-bold text-sm text-slate-100">마우스 & 키보드 반응 속도 제어판</h3>
            </div>
            <p className="text-xs text-slate-400 mb-4 leading-relaxed">
              입력 지연(Input Lag)을 최소화하고 메뉴 팝업 및 키보드 반복 반응 속도를 초고속으로 설정합니다.
            </p>

            <div className="space-y-4">
              {/* Keyboard Delay */}
              <div className="bg-slate-950/60 border border-slate-800 rounded-xl p-3.5">
                <div className="flex justify-between items-center mb-1.5">
                  <span className="text-xs font-semibold text-slate-200">키보드 반복 지연 시간</span>
                  <span className="text-xs font-bold text-indigo-400">
                    {inputSettings?.keyboard_delay === 0 ? "0 (최단/초고속)" : `${inputSettings?.keyboard_delay}단계`}
                  </span>
                </div>
                <input
                  type="range"
                  min="0"
                  max="3"
                  value={inputSettings?.keyboard_delay ?? 1}
                  onChange={(e) =>
                    setInputSettings((prev) =>
                      prev ? { ...prev, keyboard_delay: parseInt(e.target.value) } : null
                    )
                  }
                  className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-indigo-500"
                />
              </div>

              {/* Keyboard Repeat Speed */}
              <div className="bg-slate-950/60 border border-slate-800 rounded-xl p-3.5">
                <div className="flex justify-between items-center mb-1.5">
                  <span className="text-xs font-semibold text-slate-200">키보드 반복 속도</span>
                  <span className="text-xs font-bold text-indigo-400">
                    {inputSettings?.keyboard_speed === 31
                      ? "31 (초고속)"
                      : `${inputSettings?.keyboard_speed ?? 31}단계`}
                  </span>
                </div>
                <input
                  type="range"
                  min="0"
                  max="31"
                  value={inputSettings?.keyboard_speed ?? 31}
                  onChange={(e) =>
                    setInputSettings((prev) =>
                      prev ? { ...prev, keyboard_speed: parseInt(e.target.value) } : null
                    )
                  }
                  className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-indigo-500"
                />
              </div>

              {/* Menu Show Delay */}
              <div className="bg-slate-950/60 border border-slate-800 rounded-xl p-3.5">
                <div className="flex justify-between items-center mb-1.5">
                  <span className="text-xs font-semibold text-slate-200">메뉴 팝업 지연 시간 (MenuShowDelay)</span>
                  <span className="text-xs font-bold text-indigo-400">
                    {inputSettings?.menu_show_delay ?? 400} ms
                  </span>
                </div>
                <input
                  type="range"
                  min="0"
                  max="400"
                  step="10"
                  value={inputSettings?.menu_show_delay ?? 400}
                  onChange={(e) =>
                    setInputSettings((prev) =>
                      prev ? { ...prev, menu_show_delay: parseInt(e.target.value) } : null
                    )
                  }
                  className="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-indigo-500"
                />
              </div>

              {/* Mouse Acceleration Toggle */}
              <div
                onClick={() =>
                  setInputSettings((prev) =>
                    prev ? { ...prev, mouse_speed_acceleration: !prev.mouse_speed_acceleration } : null
                  )
                }
                className="flex items-center justify-between p-3.5 rounded-xl bg-slate-950/60 border border-slate-800 hover:border-slate-700 cursor-pointer transition select-none"
              >
                <div className="flex items-center gap-2.5">
                  <MousePointer className="w-4 h-4 text-slate-400" />
                  <div>
                    <div className="text-xs font-semibold text-slate-200">마우스 포인터 정확도 향상 (가속도)</div>
                    <div className="text-[11px] text-slate-500">1:1 정밀 센서 매핑을 원하시면 끄는 것을 권장합니다.</div>
                  </div>
                </div>
                {inputSettings?.mouse_speed_acceleration ? (
                  <span className="text-xs font-bold text-blue-400 flex items-center gap-1">
                    <ToggleRight className="w-5 h-5 text-blue-500" /> 켜짐
                  </span>
                ) : (
                  <span className="text-xs font-bold text-emerald-400 flex items-center gap-1">
                    <ToggleLeft className="w-5 h-5 text-slate-600" /> 꺼짐 (1:1)
                  </span>
                )}
              </div>
            </div>
          </div>

          <button
            onClick={handleSaveInputSettings}
            className="mt-4 w-full py-2.5 px-4 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs shadow-md shadow-indigo-600/20 transition"
          >
            입력 반응 속도 설정 적용하기
          </button>
        </div>
      </div>

      {/* Row 2: Windows 11 맞춤 고급 트윅 & 컴퓨터 자동 예약 종료 타이머 */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Card 3: Windows 11 고급 환경 트윅 */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-2 mb-4">
              <Sparkles className="w-4 h-4 text-amber-400" />
              <h3 className="font-bold text-sm text-slate-100">Windows 11 필수 맞춤 트윅</h3>
            </div>
            <p className="text-xs text-slate-400 mb-4 leading-relaxed">
              Windows 11의 번거로운 강제 설정(작업표시줄 중앙 정렬, 시작 메뉴 Bing 검색 광고, 강제 재부팅)을 원클릭으로 튜닝합니다.
            </p>

            <div className="space-y-3">
              {/* Taskbar Alignment */}
              <div className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800 flex items-center justify-between">
                <div>
                  <div className="text-xs font-semibold text-slate-200">작업 표시줄 아이콘 정렬</div>
                  <div className="text-[11px] text-slate-400">
                    클래식 왼쪽 정렬 vs Windows 11 가운데 정렬
                  </div>
                </div>
                <div className="flex items-center gap-1 bg-slate-900 p-1 rounded-lg border border-slate-700">
                  <button
                    onClick={() => handleToggleWin11Tweak("taskbar_alignment_center", false)}
                    className={`px-2.5 py-1 rounded text-xs font-semibold flex items-center gap-1 transition ${
                      !win11Tweaks?.taskbar_alignment_center
                        ? "bg-blue-600 text-white"
                        : "text-slate-400 hover:text-white"
                    }`}
                  >
                    <AlignLeft className="w-3.5 h-3.5" />
                    왼쪽
                  </button>
                  <button
                    onClick={() => handleToggleWin11Tweak("taskbar_alignment_center", true)}
                    className={`px-2.5 py-1 rounded text-xs font-semibold flex items-center gap-1 transition ${
                      win11Tweaks?.taskbar_alignment_center
                        ? "bg-blue-600 text-white"
                        : "text-slate-400 hover:text-white"
                    }`}
                  >
                    <AlignCenter className="w-3.5 h-3.5" />
                    가운데
                  </button>
                </div>
              </div>

              {/* Bing Search Disable */}
              <div
                onClick={() =>
                  handleToggleWin11Tweak(
                    "disable_bing_search",
                    !win11Tweaks?.disable_bing_search
                  )
                }
                className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800 hover:border-slate-700 cursor-pointer flex items-center justify-between transition select-none"
              >
                <div>
                  <div className="text-xs font-semibold text-slate-200">
                    시작 메뉴 Bing 웹 검색 결과 끄기
                  </div>
                  <div className="text-[11px] text-slate-400">
                    로컬 앱/파일만 즉각 검색되도록 속도를 극대화합니다.
                  </div>
                </div>
                {win11Tweaks?.disable_bing_search ? (
                  <span className="text-xs font-bold text-emerald-400 flex items-center gap-1">
                    <ToggleRight className="w-5 h-5 text-emerald-500" /> 차단됨
                  </span>
                ) : (
                  <span className="text-xs font-bold text-slate-500 flex items-center gap-1">
                    <ToggleLeft className="w-5 h-5 text-slate-600" /> 활성 (웹검색)
                  </span>
                )}
              </div>

              {/* Auto Reboot Disable */}
              <div
                onClick={() =>
                  handleToggleWin11Tweak(
                    "prevent_auto_reboot",
                    !win11Tweaks?.prevent_auto_reboot
                  )
                }
                className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800 hover:border-slate-700 cursor-pointer flex items-center justify-between transition select-none"
              >
                <div>
                  <div className="text-xs font-semibold text-slate-200">
                    업데이트 후 강제 자동 재부팅 방지
                  </div>
                  <div className="text-[11px] text-slate-400">
                    작업 중인 문서가 날아가지 않도록 자동 재부팅을 차단합니다.
                  </div>
                </div>
                {win11Tweaks?.prevent_auto_reboot ? (
                  <span className="text-xs font-bold text-emerald-400 flex items-center gap-1">
                    <ToggleRight className="w-5 h-5 text-emerald-500" /> 방지 중
                  </span>
                ) : (
                  <span className="text-xs font-bold text-slate-500 flex items-center gap-1">
                    <ToggleLeft className="w-5 h-5 text-slate-600" /> 기본값
                  </span>
                )}
              </div>

              {/* Take Ownership Menu */}
              <div
                onClick={() =>
                  handleToggleWin11Tweak(
                    "take_ownership_menu",
                    !win11Tweaks?.take_ownership_menu
                  )
                }
                className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800 hover:border-slate-700 cursor-pointer flex items-center justify-between transition select-none"
              >
                <div>
                  <div className="text-xs font-semibold text-slate-200">
                    우클릭 메뉴 '관리자 소유권 가져오기'
                  </div>
                  <div className="text-[11px] text-slate-400">
                    삭제되지 않는 시스템 잠금 파일 권한을 즉시 획득합니다.
                  </div>
                </div>
                {win11Tweaks?.take_ownership_menu ? (
                  <span className="text-xs font-bold text-blue-400 flex items-center gap-1">
                    <ToggleRight className="w-5 h-5 text-blue-500" /> 등록됨
                  </span>
                ) : (
                  <span className="text-xs font-bold text-slate-500 flex items-center gap-1">
                    <ToggleLeft className="w-5 h-5 text-slate-600" /> 미등록
                  </span>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Card 4: 컴퓨터 자동 예약 종료 타이머 */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-2 mb-4">
              <Clock className="w-4 h-4 text-emerald-400" />
              <h3 className="font-bold text-sm text-slate-100">컴퓨터 자동 예약 종료 타이머</h3>
            </div>
            <p className="text-xs text-slate-400 mb-4 leading-relaxed">
              대용량 다운로드나 작업 완료 후 시스템이 지정된 시간에 자동으로 안전하게 꺼지도록 예약합니다.
            </p>

            {shutdownScheduled && (
              <div className="mb-4 p-3 rounded-xl bg-amber-950/50 border border-amber-500/40 text-amber-200 text-xs flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Clock className="w-4 h-4 text-amber-400 animate-pulse" />
                  <span className="font-bold">현재 예약 동작 중: {shutdownScheduled}</span>
                </div>
                <button
                  onClick={handleCancelShutdown}
                  className="px-2.5 py-1 rounded-lg bg-red-600/80 hover:bg-red-500 text-white text-[11px] font-bold transition"
                >
                  예약 즉시 취소
                </button>
              </div>
            )}

            <div className="space-y-3.5">
              {/* 동작 선택 */}
              <div>
                <label className="text-[11px] font-bold text-slate-300 block mb-1.5">실행 동작</label>
                <div className="grid grid-cols-3 gap-2">
                  {[
                    { id: "shutdown", label: "시스템 종료", icon: Power },
                    { id: "restart", label: "다시 시작", icon: RotateCcw },
                    { id: "hibernate", label: "최대 절전", icon: Moon },
                  ].map((act) => {
                    const Icon = act.icon;
                    return (
                      <button
                        key={act.id}
                        onClick={() => setShutdownAction(act.id)}
                        className={`py-2 px-3 rounded-xl border text-xs font-semibold flex items-center justify-center gap-1.5 transition ${
                          shutdownAction === act.id
                            ? "bg-emerald-950/40 border-emerald-500 text-emerald-300"
                            : "bg-slate-950/60 border-slate-800 text-slate-400 hover:border-slate-700"
                        }`}
                      >
                        <Icon className="w-3.5 h-3.5" />
                        {act.label}
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* 시간 선택 */}
              <div>
                <label className="text-[11px] font-bold text-slate-300 block mb-1.5">종료 시간</label>
                <div className="grid grid-cols-4 gap-2 mb-2">
                  {[15, 30, 60, 120].map((mins) => (
                    <button
                      key={mins}
                      onClick={() => {
                        setShutdownMinutes(mins);
                        setCustomMinutes("");
                      }}
                      className={`py-2 px-2 rounded-xl border text-xs font-semibold transition ${
                        shutdownMinutes === mins && !customMinutes
                          ? "bg-blue-950/40 border-blue-500 text-blue-300"
                          : "bg-slate-950/60 border-slate-800 text-slate-400 hover:border-slate-700"
                      }`}
                    >
                      {mins < 60 ? `${mins}분 후` : `${mins / 60}시간 후`}
                    </button>
                  ))}
                </div>

                <div className="flex items-center gap-2">
                  <input
                    type="number"
                    min="1"
                    placeholder="직접 입력 (분 단위)"
                    value={customMinutes}
                    onChange={(e) => setCustomMinutes(e.target.value)}
                    className="flex-1 bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-slate-200 outline-none focus:border-blue-500 transition"
                  />
                  <span className="text-xs text-slate-400">분 후</span>
                </div>
              </div>
            </div>
          </div>

          <div className="flex gap-2 mt-4">
            <button
              onClick={handleScheduleShutdown}
              className="flex-1 py-2.5 px-4 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs shadow-md shadow-emerald-600/20 transition flex items-center justify-center gap-1.5"
            >
              <Clock className="w-3.5 h-3.5" />
              예약 실행
            </button>
            {shutdownScheduled && (
              <button
                onClick={handleCancelShutdown}
                className="py-2.5 px-4 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 font-bold text-xs transition"
              >
                취소
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Row 3: 전원/용량 트윅 & 작업 표시줄 트윅 */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Card 5: 전원 및 드라이브 용량 확보 */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5">
          <div className="flex items-center gap-2 mb-4">
            <BatteryCharging className="w-4 h-4 text-emerald-400" />
            <h3 className="font-bold text-sm text-slate-100">전원 옵션 및 드라이브 용량 제어</h3>
          </div>

          <div className="space-y-3">
            {/* Hibernate */}
            <div className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800 flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-slate-200">최대 절전 모드 (hiberfil.sys)</div>
                <div className="text-[11px] text-slate-400">
                  해제 시 RAM 크기만큼 C: 드라이브(수십 GB) 용량을 즉시 확보합니다.
                </div>
              </div>
              <button
                onClick={handleToggleHibernate}
                className={`py-1.5 px-3 rounded-lg text-xs font-semibold transition ${
                  powerSettings?.hibernate_enabled
                    ? "bg-amber-600/30 text-amber-300 border border-amber-500/30 hover:bg-amber-600/50"
                    : "bg-emerald-600/30 text-emerald-300 border border-emerald-500/30 hover:bg-emerald-600/50"
                }`}
              >
                {powerSettings?.hibernate_enabled ? "끄기 (용량 확보)" : "켜기"}
              </button>
            </div>

            {/* Ultimate Performance */}
            <div className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800 flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-slate-200">최고의 성능 (Ultimate Performance) 전원 플랜</div>
                <div className="text-[11px] text-slate-400">
                  마이크로 레이턴시를 제거하는 최고 성능 전원 옵션을 제어판에 잠금 해제합니다.
                </div>
              </div>
              <button
                onClick={handleUnlockUltimatePower}
                disabled={powerSettings?.ultimate_performance_unlocked}
                className={`py-1.5 px-3 rounded-lg text-xs font-semibold shadow-sm transition ${
                  powerSettings?.ultimate_performance_unlocked
                    ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 cursor-default"
                    : "bg-blue-600 hover:bg-blue-500 text-white"
                }`}
              >
                {powerSettings?.ultimate_performance_unlocked ? "잠금 해제됨" : "플랜 활성화"}
              </button>
            </div>
          </div>
        </div>

        {/* Card 6: 작업 표시줄 요소 제어판 */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5">
          <div className="flex items-center gap-2 mb-4">
            <Layout className="w-4 h-4 text-cyan-400" />
            <h3 className="font-bold text-sm text-slate-100">작업 표시줄 부가 요소 제어판</h3>
          </div>

          <div className="space-y-3">
            {/* Searchbox Mode */}
            <div className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800 flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-slate-200">검색 상자 스타일</div>
                <div className="text-[11px] text-slate-400">작업 표시줄 검색 공간을 최적화합니다.</div>
              </div>
              <select
                value={taskbarSettings?.searchbox_mode ?? 1}
                onChange={(e) => handleChangeTaskbar("searchbox_mode", parseInt(e.target.value))}
                className="bg-slate-900 border border-slate-700 text-slate-200 text-xs rounded-lg px-2.5 py-1.5 outline-none"
              >
                <option value={0}>숨김</option>
                <option value={1}>검색 아이콘만</option>
                <option value={2}>전체 검색 상자</option>
                <option value={3}>아이콘 및 레이블</option>
              </select>
            </div>

            {/* Widgets */}
            <div className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800 flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-slate-200">위젯 (날씨/뉴스) 버튼</div>
                <div className="text-[11px] text-slate-400">작업 표시줄 좌측 위젯 아이콘을 끕니다.</div>
              </div>
              <button
                onClick={() =>
                  handleChangeTaskbar("show_widgets", taskbarSettings?.show_widgets ? 0 : 1)
                }
                className={`py-1.5 px-3 rounded-lg text-xs font-semibold transition ${
                  taskbarSettings?.show_widgets
                    ? "bg-slate-800 text-slate-300 hover:bg-slate-700"
                    : "bg-blue-600/30 text-blue-300 border border-blue-500/30"
                }`}
              >
                {taskbarSettings?.show_widgets ? "표시 중" : "숨김 (깔끔)"}
              </button>
            </div>

            {/* Task View */}
            <div className="p-3.5 rounded-xl bg-slate-950/60 border border-slate-800 flex items-center justify-between">
              <div>
                <div className="text-xs font-semibold text-slate-200">작업 보기 (Task View) 버튼</div>
                <div className="text-[11px] text-slate-400">가상 데스크톱 전환 버튼 표시 여부</div>
              </div>
              <button
                onClick={() =>
                  handleChangeTaskbar("show_task_view", taskbarSettings?.show_task_view ? 0 : 1)
                }
                className={`py-1.5 px-3 rounded-lg text-xs font-semibold transition ${
                  taskbarSettings?.show_task_view
                    ? "bg-slate-800 text-slate-300 hover:bg-slate-700"
                    : "bg-blue-600/30 text-blue-300 border border-blue-500/30"
                }`}
              >
                {taskbarSettings?.show_task_view ? "표시 중" : "숨김"}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Row 4: Windows 핵심 제어판 애플릿 런처 그리드 */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Sliders className="w-4 h-4 text-blue-400" />
            <h3 className="font-bold text-sm text-slate-100">Windows 핵심 제어판 & 관리 콘솔 런처</h3>
          </div>
          <span className="text-[11px] text-slate-400">원클릭으로 윈도우 기본 제어판 즉시 실행</span>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
          {applets.map((app) => {
            const Icon = app.icon;
            return (
              <button
                key={app.id}
                onClick={() => handleLaunchApplet(app.id)}
                className="flex items-start gap-3 p-3.5 rounded-xl bg-slate-950/60 border border-slate-800/80 hover:border-blue-500/50 hover:bg-slate-800/50 transition text-left group"
              >
                <div className="w-8 h-8 rounded-lg bg-slate-900 border border-slate-800 flex items-center justify-center shrink-0 group-hover:border-blue-500/40">
                  <Icon className={`w-4 h-4 ${app.color || "text-slate-300 group-hover:text-blue-400"}`} />
                </div>
                <div className="min-w-0">
                  <div className="text-xs font-bold text-slate-200 group-hover:text-white flex items-center gap-1 truncate">
                    <span>{app.name}</span>
                    <ExternalLink className="w-2.5 h-2.5 text-slate-500 opacity-0 group-hover:opacity-100 transition" />
                  </div>
                  <div className="text-[10px] text-slate-500 truncate mt-0.5">{app.desc}</div>
                </div>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
};
