import React, { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sidebar } from "./components/Sidebar";
import { Header } from "./components/Header";
import { DashboardTab } from "./tabs/DashboardTab";
import { StartupTab } from "./tabs/StartupTab";
import { TweaksTab } from "./tabs/TweaksTab";
import { ControlPanelTab } from "./tabs/ControlPanelTab";
import { CleanerTab } from "./tabs/CleanerTab";
import { WindowToolTab } from "./tabs/WindowToolTab";
import { BackupGuardTab } from "./tabs/BackupGuardTab";
import { NetworkTab } from "./tabs/NetworkTab";
import { MacroTab } from "./tabs/MacroTab";
import { RestoreTab } from "./tabs/RestoreTab";
import { SystemSummary, TabKey } from "./types";
import "./App.css";

export const App: React.FC = () => {
  const [currentTab, setCurrentTab] = useState<TabKey>("dashboard");
  const [summary, setSummary] = useState<SystemSummary | null>(null);
  const fetchingRef = useRef(false);

  const fetchSummary = async () => {
    if (fetchingRef.current) return;
    fetchingRef.current = true;
    try {
      const data = await invoke<SystemSummary>("get_system_summary");
      setSummary(data);
    } catch (err) {
      console.error("시스템 요약 로드 실패:", err);
    } finally {
      fetchingRef.current = false;
    }
  };

  useEffect(() => {
    fetchSummary();
    const interval = setInterval(fetchSummary, 5000);
    return () => clearInterval(interval);
  }, []);

  const renderActiveTab = () => {
    switch (currentTab) {
      case "dashboard":
        return (
          <DashboardTab
            summary={summary}
            onNavigateTab={setCurrentTab}
            onRefresh={fetchSummary}
          />
        );
      case "startup":
        return <StartupTab />;
      case "tweaks":
        return <TweaksTab />;
      case "control_panel":
        return <ControlPanelTab />;
      case "cleaner":
        return <CleanerTab />;
      case "window":
        return <WindowToolTab />;
      case "guard":
        return <BackupGuardTab />;
      case "network":
        return <NetworkTab />;
      case "macro":
        return <MacroTab />;
      case "restore":
        return <RestoreTab />;
      default:
        return null;
    }
  };

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-slate-950 text-slate-100 font-sans">
      {/* Modern Fluent Sidebar */}
      <Sidebar
        currentTab={currentTab}
        onSelectTab={setCurrentTab}
        isElevated={summary?.is_elevated ?? false}
      />

      {/* Main Content Area */}
      <div className="flex-1 min-w-0 flex flex-col h-full overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900/50 to-slate-950">
        <Header
          currentTab={currentTab}
          summary={summary}
          onRefresh={fetchSummary}
        />

        <main className="flex-1 min-w-0 overflow-y-auto overflow-x-hidden p-6">
          <div className="max-w-6xl mx-auto pb-12">
            {renderActiveTab()}
          </div>
        </main>
      </div>
    </div>
  );
};

export default App;
