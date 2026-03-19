import { useEffect } from "react";
import { BrowserRouter, Route, Routes, Navigate } from "react-router-dom";
import Layout from "./components/layout/Layout";
import Dashboard from "./components/dashboard/Dashboard";
import AgentList from "./components/agents/AgentList";
import MemoryBrowser from "./components/memory/MemoryBrowser";
import ChatHistory from "./components/chats/ChatHistory";
import CommitHistory from "./components/history/CommitHistory";
import Settings from "./components/settings/Settings";
import SetupWizard from "./components/setup/SetupWizard";
import { useSyncStore } from "./stores/syncStore";
import { useSettingsStore } from "./stores/settingsStore";
import { listen } from "@tauri-apps/api/event";

function App() {
  const { setStatus, setIsSyncing } = useSyncStore();
  const { machineConfig, loadSettings } = useSettingsStore();

  useEffect(() => {
    loadSettings();

    // Listen for sync events from backend
    const unlisten1 = listen("sync-started", () => {
      setIsSyncing(true);
    });

    const unlisten2 = listen("sync-completed", (event) => {
      setIsSyncing(false);
    });

    const unlisten3 = listen("sync-error", (event) => {
      setIsSyncing(false);
    });

    return () => {
      unlisten1.then((f) => f());
      unlisten2.then((f) => f());
      unlisten3.then((f) => f());
    };
  }, []);

  // Show setup wizard if no remote configured
  const needsSetup = !machineConfig?.remoteUrl;

  return (
    <BrowserRouter>
      <Routes>
        {needsSetup ? (
          <>
            <Route path="/setup" element={<SetupWizard />} />
            <Route path="*" element={<Navigate to="/setup" replace />} />
          </>
        ) : (
          <Route element={<Layout />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/agents" element={<AgentList />} />
            <Route path="/memory" element={<MemoryBrowser />} />
            <Route path="/chats" element={<ChatHistory />} />
            <Route path="/history" element={<CommitHistory />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Route>
        )}
      </Routes>
    </BrowserRouter>
  );
}

export default App;
