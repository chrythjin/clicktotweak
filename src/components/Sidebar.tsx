import React from "react";
import {
  LayoutDashboard,
  Zap,
  Sliders,
  Settings2,
  Trash2,
  AppWindow,
  ShieldCheck,
  Wifi,
  PlaySquare,
  History,
} from "lucide-react";
import { TabKey } from "../types";

interface SidebarProps {
  currentTab: TabKey;
  onSelectTab: (tab: TabKey) => void;
  isElevated: boolean;
}

const navItems: { key: TabKey; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { key: "dashboard", label: "대시보드", icon: LayoutDashboard },
  { key: "startup", label: "시작프로그램", icon: Zap },
  { key: "tweaks", label: "시스템 트윅", icon: Sliders },
  { key: "control_panel", label: "제어판 & 시스템 도구", icon: Settings2 },
  { key: "cleaner", label: "정크 & 캐시 청소", icon: Trash2 },
  { key: "window", label: "창 & 프로세스 제어", icon: AppWindow },
  { key: "guard", label: "파일 보호 & 백업", icon: ShieldCheck },
  { key: "network", label: "네트워크 & 차단", icon: Wifi },
  { key: "macro", label: "매크로 플레이어", icon: PlaySquare },
  { key: "restore", label: "복원 센터", icon: History },
];

export const Sidebar: React.FC<SidebarProps> = ({ currentTab, onSelectTab, isElevated }) => {
  return (
    <aside className="w-64 bg-slate-900/90 backdrop-blur border-r border-slate-800 flex flex-col justify-between select-none">
      <div>
        {/* Brand Header */}
        <div className="p-5 border-b border-slate-800/80">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-blue-600 to-cyan-500 flex items-center justify-center shadow-lg shadow-blue-500/20">
              <span className="text-white font-black text-lg">C</span>
            </div>
            <div>
              <div className="font-bold text-slate-100 tracking-tight text-base leading-none flex items-center gap-1.5">
                Click To Tweak
                <span className="text-[10px] px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-400 font-semibold border border-blue-500/30">
                  2026
                </span>
              </div>
              <div className="text-xs text-slate-400 mt-1">Next-Gen Modern Utility</div>
            </div>
          </div>
        </div>

        {/* Navigation Items */}
        <nav className="p-3 space-y-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = currentTab === item.key;
            return (
              <button
                key={item.key}
                onClick={() => onSelectTab(item.key)}
                className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-150 text-left ${
                  isActive
                    ? "bg-blue-600 text-white shadow-md shadow-blue-600/30"
                    : "text-slate-300 hover:bg-slate-800/70 hover:text-white"
                }`}
              >
                <Icon className={`w-4 h-4 shrink-0 ${isActive ? "text-white" : "text-slate-400"}`} />
                <span>{item.label}</span>
              </button>
            );
          })}
        </nav>
      </div>

      {/* Footer Info */}
      <div className="p-4 border-t border-slate-800/80 bg-slate-950/40 text-xs">
        <div className="flex items-center justify-between">
          <span className="text-slate-400">실행 권한:</span>
          {isElevated ? (
            <span className="inline-flex items-center gap-1 text-emerald-400 font-medium">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
              관리자 권한
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 text-amber-400 font-medium">
              <span className="w-1.5 h-1.5 rounded-full bg-amber-400" />
              일반 사용자
            </span>
          )}
        </div>
        <div className="text-[11px] text-slate-500 mt-2 text-center">
          Rust x64 Native Architecture
        </div>
      </div>
    </aside>
  );
};

