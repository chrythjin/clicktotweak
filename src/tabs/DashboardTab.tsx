import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Monitor,
  Cpu,
  Layers,
  Clock,
  HardDrive,
  Shield,
  FileCheck,
  ExternalLink,
} from "lucide-react";
import { SystemSummary, TabKey } from "../types";
import { formatGigabytes, formatUptime } from "../lib/format";
import { useFeedback } from "../hooks/useFeedback";
import { FeedbackBanner } from "../components/FeedbackBanner";

interface DashboardTabProps {
  summary: SystemSummary | null;
  onNavigateTab: (tab: TabKey) => void;
  onRefresh?: () => void;
}

export const DashboardTab: React.FC<DashboardTabProps> = ({ summary, onNavigateTab }) => {
  const [quickToggling, setQuickToggling] = useState(false);
  const { message: feedback, isError, notify } = useFeedback(3000);

  const handleQuickClassicMenu = async () => {
    try {
      setQuickToggling(true);
      const res = await invoke<string>("set_win11_classic_menu", { enable: true });
      notify(res, false);
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    } finally {
      setQuickToggling(false);
    }
  };

  const handleQuickFileExt = async () => {
    try {
      setQuickToggling(true);
      const res = await invoke<string>("set_show_file_extensions", { show: true });
      notify(res, false);
    } catch (e: any) {
      notify(`오류: ${e}`, true);
    } finally {
      setQuickToggling(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Feedback Banner */}
      <FeedbackBanner message={feedback} isError={isError} />

      {/* Row 1: System Specs & Live Load Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Card 1: OS */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 shadow-sm">
          <div className="flex items-center justify-between text-slate-400 mb-3">
            <span className="text-xs font-semibold uppercase tracking-wider">운영체제</span>
            <Monitor className="w-5 h-5 text-blue-400" />
          </div>
          <div className="text-xl font-bold text-slate-100 truncate">
            {summary?.os_name || "Windows"}
          </div>
          <div className="text-xs text-slate-400 mt-1 truncate">
            버전: {summary?.os_version || "11"} ({summary?.hostname})
          </div>
        </div>

        {/* Card 2: CPU */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 shadow-sm">
          <div className="flex items-center justify-between text-slate-400 mb-3">
            <span className="text-xs font-semibold uppercase tracking-wider">프로세서</span>
            <Cpu className="w-5 h-5 text-indigo-400" />
          </div>
          <div className="text-xl font-bold text-slate-100 truncate">
            {summary?.cpu_usage.toFixed(0)}% 사용 중
          </div>
          <div className="text-xs text-slate-400 mt-1 truncate" title={summary?.cpu_name}>
            {summary?.cpu_name || "CPU"} ({summary?.cpu_cores} 코어)
          </div>
        </div>

        {/* Card 3: Memory */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 shadow-sm">
          <div className="flex items-center justify-between text-slate-400 mb-3">
            <span className="text-xs font-semibold uppercase tracking-wider">물리 메모리</span>
            <Layers className="w-5 h-5 text-cyan-400" />
          </div>
          <div className="text-xl font-bold text-slate-100">
            {summary?.memory_percentage.toFixed(0)}% 점유
          </div>
          <div className="text-xs text-slate-400 mt-1">
            {summary ? formatGigabytes(summary.used_memory) : "0 GB"} / {summary ? formatGigabytes(summary.total_memory) : "0 GB"}
          </div>
        </div>

        {/* Card 4: Uptime */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 shadow-sm">
          <div className="flex items-center justify-between text-slate-400 mb-3">
            <span className="text-xs font-semibold uppercase tracking-wider">가동 시간</span>
            <Clock className="w-5 h-5 text-emerald-400" />
          </div>
          <div className="text-xl font-bold text-slate-100">
            {summary ? formatUptime(summary.uptime_seconds) : "계산 중..."}
          </div>
          <div className="text-xs text-slate-400 mt-1">
            {summary?.is_elevated ? "UAC 관리자 승인 활성" : "일반 권한으로 실행됨"}
          </div>
        </div>
      </div>

      {/* Row 2: Storage Drives Grid */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <HardDrive className="w-4 h-4 text-blue-400" />
            <h3 className="font-semibold text-sm text-slate-200">스토리지 드라이브 현황</h3>
          </div>
          <button
            onClick={() => onNavigateTab("cleaner")}
            className="text-xs text-blue-400 hover:text-blue-300 font-medium flex items-center gap-1"
          >
            <span>정크 청소 열기</span>
            <ExternalLink className="w-3 h-3" />
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {summary?.disks.map((disk, idx) => {
            const used = disk.total_space - disk.available_space;
            const pct = disk.total_space > 0 ? (used / disk.total_space) * 100 : 0;
            return (
              <div key={idx} className="bg-slate-950/60 border border-slate-800/80 rounded-xl p-4">
                <div className="flex items-center justify-between mb-2">
                  <span className="font-bold text-slate-100 text-sm">{disk.mount_point}</span>
                  <span className="text-xs text-slate-400">{disk.file_system}</span>
                </div>
                {/* Progress bar */}
                <div className="w-full bg-slate-800 rounded-full h-2 mb-2 overflow-hidden">
                  <div
                    className={`h-2 rounded-full transition-all ${
                      pct > 90 ? "bg-red-500" : pct > 75 ? "bg-amber-500" : "bg-blue-500"
                    }`}
                    style={{ width: `${pct}%` }}
                  />
                </div>
                <div className="flex justify-between text-xs text-slate-400">
                  <span>사용: {formatGigabytes(used)}</span>
                  <span>여유: {formatGigabytes(disk.available_space)} ({disk.total_space > 0 ? (100 - pct).toFixed(0) : 0}%)</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Row 3: Quick Action Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Card 1 */}
        <div className="bg-gradient-to-br from-slate-900 to-slate-900/40 border border-slate-800 rounded-2xl p-5 hover:border-slate-700 transition">
          <div className="w-9 h-9 rounded-xl bg-blue-500/20 text-blue-400 flex items-center justify-center mb-3">
            <Layers className="w-5 h-5" />
          </div>
          <h4 className="font-bold text-slate-100 text-sm">Win11 클래식 우클릭 메뉴</h4>
          <p className="text-xs text-slate-400 mt-1 mb-4 leading-relaxed">
            Windows 11의 중첩된 '더 많은 옵션 표시'를 생략하고 즉시 전체 메뉴를 복원합니다.
          </p>
          <button
            onClick={handleQuickClassicMenu}
            disabled={quickToggling}
            className="w-full py-2 px-3 rounded-lg bg-slate-800 hover:bg-blue-600 text-slate-200 hover:text-white text-xs font-semibold transition"
          >
            클래식 메뉴 즉시 적용
          </button>
        </div>

        {/* Card 2 */}
        <div className="bg-gradient-to-br from-slate-900 to-slate-900/40 border border-slate-800 rounded-2xl p-5 hover:border-slate-700 transition">
          <div className="w-9 h-9 rounded-xl bg-indigo-500/20 text-indigo-400 flex items-center justify-center mb-3">
            <FileCheck className="w-5 h-5" />
          </div>
          <h4 className="font-bold text-slate-100 text-sm">파일 확장자 항상 표시</h4>
          <p className="text-xs text-slate-400 mt-1 mb-4 leading-relaxed">
            보안과 식별을 위해 모든 알려진 파일 확장자(.exe, .txt, .zip)를 항상 표시합니다.
          </p>
          <button
            onClick={handleQuickFileExt}
            disabled={quickToggling}
            className="w-full py-2 px-3 rounded-lg bg-slate-800 hover:bg-indigo-600 text-slate-200 hover:text-white text-xs font-semibold transition"
          >
            확장자 표시 즉시 적용
          </button>
        </div>

        {/* Card 3 */}
        <div className="bg-gradient-to-br from-slate-900 to-slate-900/40 border border-slate-800 rounded-2xl p-5 hover:border-slate-700 transition">
          <div className="w-9 h-9 rounded-xl bg-emerald-500/20 text-emerald-400 flex items-center justify-center mb-3">
            <Shield className="w-5 h-5" />
          </div>
          <h4 className="font-bold text-slate-100 text-sm">원클릭 안전 복원 지점</h4>
          <p className="text-xs text-slate-400 mt-1 mb-4 leading-relaxed">
            설정을 변경하기 전 언제든 이전 상태로 롤백할 수 있는 복원 지점을 생성합니다.
          </p>
          <button
            onClick={() => onNavigateTab("restore")}
            className="w-full py-2 px-3 rounded-lg bg-slate-800 hover:bg-emerald-600 text-slate-200 hover:text-white text-xs font-semibold transition"
          >
            복원 센터 열기
          </button>
        </div>
      </div>
    </div>
  );
};
