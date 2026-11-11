import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Cpu, HardDrive, RefreshCw, Sparkles, CheckCircle2 } from "lucide-react";
import { SystemSummary, TabKey } from "../types";

interface HeaderProps {
  currentTab: TabKey;
  summary: SystemSummary | null;
  onRefresh: () => void;
}

const tabTitles: Record<TabKey, { title: string; subtitle: string }> = {
  dashboard: { title: "대시보드 개요", subtitle: "실시간 하드웨어 상태 및 원클릭 시스템 부스트" },
  startup: { title: "시작프로그램 관리", subtitle: "부팅 시 자동 실행되는 앱 및 서비스 제어" },
  tweaks: { title: "시스템 & 탐색기 트윅", subtitle: "Windows 11 인터페이스 및 사용 환경 맞춤 최적화" },
  control_panel: { title: "제어판 & 시스템 도구", subtitle: "바탕화면 시스템 아이콘, 마우스/키보드 반응 속도 및 Windows 관리 콘솔" },
  cleaner: { title: "정크 & 캐시 청소", subtitle: "임시 파일, 충돌 덤프, 웹 브라우저 잔여 데이터 안전 정리" },
  window: { title: "창 & 프로세스 제어", subtitle: "투명도 조절, 항상 위 고정, 프로세스 일시중지 및 메모리 정리" },
  guard: { title: "파일 보호 & 백업 가드", subtitle: "중요 파일 무중단 스냅샷 및 즉시 복원 센터" },
  network: { title: "네트워크 & 트래픽 모니터", subtitle: "실시간 전송 대역폭 측정 및 시스템 전역 광고 차단" },
  macro: { title: "매크로 플레이어", subtitle: "반복적인 마우스/키보드 작업 자동화 매크로" },
  restore: { title: "복원 센터", subtitle: "트윅 변경 이력 스냅샷 열람 및 1클릭 시스템 롤백" },
};

export const Header: React.FC<HeaderProps> = ({ currentTab, summary, onRefresh }) => {
  const [optimizing, setOptimizing] = useState(false);
  const [notifyMsg, setNotifyMsg] = useState<string | null>(null);

  const info = tabTitles[currentTab];

  const handleQuickOptimize = async () => {
    try {
      setOptimizing(true);
      const res = await invoke<string>("optimize_memory");
      setNotifyMsg(res);
      onRefresh();
      setTimeout(() => setNotifyMsg(null), 4000);
    } catch (err: any) {
      setNotifyMsg(`최적화 실패: ${err}`);
      setTimeout(() => setNotifyMsg(null), 4000);
    } finally {
      setOptimizing(false);
    }
  };

  const ramUsedGb = summary ? (summary.used_memory / 1024 / 1024 / 1024).toFixed(1) : "0";
  const ramTotalGb = summary ? (summary.total_memory / 1024 / 1024 / 1024).toFixed(1) : "0";

  return (
    <header className="h-20 bg-slate-900/60 backdrop-blur border-b border-slate-800 px-6 flex items-center justify-between shrink-0">
      <div>
        <h1 className="text-xl font-bold text-slate-100 flex items-center gap-2">
          {info.title}
        </h1>
        <p className="text-xs text-slate-400 mt-0.5">{info.subtitle}</p>
      </div>

      <div className="flex items-center gap-4">
        {/* Toast / Notification pill */}
        {notifyMsg && (
          <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-xs animate-fade-in">
            <CheckCircle2 className="w-3.5 h-3.5" />
            <span>{notifyMsg}</span>
          </div>
        )}

        {/* Live Hardware Gauges */}
        {summary && (
          <div className="hidden lg:flex items-center gap-3 bg-slate-950/60 px-3.5 py-1.5 rounded-xl border border-slate-800 text-xs">
            <div className="flex items-center gap-1.5">
              <Cpu className="w-3.5 h-3.5 text-blue-400" />
              <span className="text-slate-400">CPU</span>
              <span className="font-semibold text-slate-200">
                {summary.cpu_usage.toFixed(0)}%
              </span>
            </div>
            <div className="w-px h-3 bg-slate-800" />
            <div className="flex items-center gap-1.5">
              <HardDrive className="w-3.5 h-3.5 text-cyan-400" />
              <span className="text-slate-400">RAM</span>
              <span className="font-semibold text-slate-200">
                {ramUsedGb}/{ramTotalGb} GB ({summary.memory_percentage.toFixed(0)}%)
              </span>
            </div>
          </div>
        )}

        {/* Quick Optimize Action */}
        <button
          onClick={handleQuickOptimize}
          disabled={optimizing}
          className="flex items-center gap-2 px-3.5 py-2 rounded-lg bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 text-white text-xs font-semibold shadow-md shadow-blue-500/20 transition-all disabled:opacity-50"
        >
          <Sparkles className={`w-3.5 h-3.5 ${optimizing ? "animate-spin" : ""}`} />
          <span>{optimizing ? "최적화 중..." : "원클릭 메모리 부스트"}</span>
        </button>

        {/* Refresh button */}
        <button
          onClick={onRefresh}
          className="p-2 rounded-lg bg-slate-800/80 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors"
          title="새로고침"
        >
          <RefreshCw className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
