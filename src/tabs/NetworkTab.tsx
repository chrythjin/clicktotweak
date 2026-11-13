import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Wifi,
  ArrowDown,
  ArrowUp,
  Shield,
  ToggleLeft,
  ToggleRight,
  Radio,
} from "lucide-react";
import { NetworkTrafficSummary } from "../types";
import { useFeedback } from "../hooks/useFeedback";
import { FeedbackBanner } from "../components/FeedbackBanner";
import { formatBandwidth } from "../lib/format";

export const NetworkTab: React.FC = () => {
  const [summary, setSummary] = useState<NetworkTrafficSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const { message: feedback, isError, notify } = useFeedback(4000);
  const [flushingDns, setFlushingDns] = useState(false);

  const fetchTraffic = async () => {
    try {
      const res = await invoke<NetworkTrafficSummary>("get_network_summary");
      setSummary(res);
    } catch (e: any) {
      console.error(e);
    }
  };

  useEffect(() => {
    fetchTraffic();
    const timer = setInterval(fetchTraffic, 2000);
    return () => clearInterval(timer);
  }, []);

  const handleToggleAdblock = async () => {
    if (!summary) return;
    try {
      setLoading(true);
      const next = !summary.is_adblock_active;
      const res = await invoke<string>("toggle_adblock", { enable: next });
      notify(res, false);
      fetchTraffic();
    } catch (e: any) {
      notify(`광고 차단 설정 실패: ${e}`, true);
    } finally {
      setLoading(false);
    }
  };

  const handleFlushDns = async () => {
    try {
      setFlushingDns(true);
      const res = await invoke<string>("flush_dns_cache");
      notify(res, false);
    } catch (e: any) {
      notify(`DNS 플러시 실패: ${e}`, true);
    } finally {
      setFlushingDns(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Top Banner */}
      <div className="flex items-center justify-between bg-slate-900/60 p-4 rounded-xl border border-slate-800">
        <div>
          <div className="flex items-center gap-2">
            <Wifi className="w-5 h-5 text-blue-400" />
            <span className="text-sm font-semibold text-slate-200">
              실시간 네트워크 트래픽 및 시스템 전역 광고 필터링
            </span>
          </div>
          <p className="text-xs text-slate-400 mt-1">
            구 CTT의 위험한 브라우저 BHO 대신 시스템 레벨 안전한 루프백 필터링을 적용합니다.
          </p>
        </div>

        <div className="flex items-center gap-2 text-xs text-slate-400 bg-slate-950/60 px-3 py-1.5 rounded-lg border border-slate-800">
          <Radio className="w-3.5 h-3.5 text-emerald-400 animate-pulse" />
          <span>인터페이스: {summary?.active_interface_name || "스캔 중..."}</span>
        </div>
      </div>

      {/* Feedback Banner */}
      <FeedbackBanner message={feedback} isError={isError} />

      {/* Traffic Meter Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Download RX Card */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 shadow-sm">
          <div className="flex items-center justify-between mb-3">
            <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">
              실시간 수신 속도 (Download)
            </span>
            <div className="p-2 rounded-xl bg-blue-500/20 text-blue-400">
              <ArrowDown className="w-5 h-5" />
            </div>
          </div>
          <div className="text-2xl font-bold text-slate-100 font-mono">
            {summary ? formatBandwidth(summary.total_rx_bytes_sec) : "0 B/s"}
          </div>
          <div className="text-xs text-slate-400 mt-1">
            현재 네트워크 스트림 수신 대역폭
          </div>
        </div>

        {/* Upload TX Card */}
        <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 shadow-sm">
          <div className="flex items-center justify-between mb-3">
            <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">
              실시간 송신 속도 (Upload)
            </span>
            <div className="p-2 rounded-xl bg-cyan-500/20 text-cyan-400">
              <ArrowUp className="w-5 h-5" />
            </div>
          </div>
          <div className="text-2xl font-bold text-slate-100 font-mono">
            {summary ? formatBandwidth(summary.total_tx_bytes_sec) : "0 B/s"}
          </div>
          <div className="text-xs text-slate-400 mt-1">
            현재 네트워크 패킷 송신 대역폭
          </div>
        </div>
      </div>

      {/* Ad Blocker Card */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-6">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className="p-3 rounded-2xl bg-indigo-500/20 text-indigo-400">
              <Shield className="w-6 h-6" />
            </div>
            <div>
              <h3 className="font-bold text-base text-slate-100">
                시스템 전역 광고 및 악성 트래커 차단 필터
              </h3>
              <p className="text-xs text-slate-400 mt-1 leading-relaxed max-w-xl">
                웹 브라우저(Edge, Chrome, Whale) 및 Windows 앱 전반에서 로드되는 알려진 주요 악성 광고 서버와 트래커 도메인을 시스템 차원에서 루프백(0.0.0.0)으로 즉시 무력화합니다.
              </p>
            </div>
          </div>

          <button
            onClick={handleToggleAdblock}
            disabled={loading}
            className="focus:outline-none cursor-pointer"
            title={summary?.is_adblock_active ? "광고 차단 끄기" : "광고 차단 켜기"}
          >
            {summary?.is_adblock_active ? (
              <ToggleRight className="w-10 h-10 text-emerald-400 transition" />
            ) : (
              <ToggleLeft className="w-10 h-10 text-slate-600 transition" />
            )}
          </button>
        </div>

        <div className="mt-6 pt-4 border-t border-slate-800/80 flex items-center justify-between text-xs">
          <span className="text-slate-400">
            차단 상태:{" "}
            <strong className={summary?.is_adblock_active ? "text-emerald-400" : "text-slate-500"}>
              {summary?.is_adblock_active ? "활성화됨 (보호 중)" : "비활성화됨"}
            </strong>
          </span>
          <span className="text-slate-500">
            등록된 주요 차단 도메인: {summary?.blocked_domains_count ?? 16}개
          </span>
        </div>
      </div>

      {/* DNS Cache Flush Card */}
      <div className="bg-slate-900/80 border border-slate-800 rounded-2xl p-5 flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h4 className="font-bold text-slate-100 text-sm">Windows DNS 캐시 플러시 & 인터넷 초기화</h4>
          <p className="text-xs text-slate-400 mt-1 leading-relaxed">
            웹사이트 접속 장애, 도메인 연결 지연 또는 DNS 변조 문제를 해결하기 위해 시스템 DNS 캐시를 1클릭으로 완전히 초기화합니다.
          </p>
        </div>
        <button
          onClick={handleFlushDns}
          disabled={flushingDns}
          className="px-4 py-2.5 rounded-xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-md shadow-blue-500/20 shrink-0 transition"
        >
          {flushingDns ? "DNS 초기화 중..." : "DNS 캐시 즉시 플러시"}
        </button>
      </div>
    </div>
  );
};
