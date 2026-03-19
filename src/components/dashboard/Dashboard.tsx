import { useEffect } from "react";
import { RefreshCw, Server, Clock, AlertTriangle, CheckCircle } from "lucide-react";
import { useSyncStore } from "../../stores/syncStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { formatRelativeTime } from "../../lib/utils";
import PendingChanges from "./PendingChanges";
import SyncStatus from "./SyncStatus";

export default function Dashboard() {
  const { status, isSyncing, lastResult, syncNow, refreshStatus, refreshPending } =
    useSyncStore();
  const { machineConfig } = useSettingsStore();

  useEffect(() => {
    refreshStatus();
    refreshPending();
    const interval = setInterval(() => {
      refreshStatus();
      refreshPending();
    }, 30_000);
    return () => clearInterval(interval);
  }, []);

  const handleSync = async () => {
    try {
      await syncNow();
    } catch (e) {
      console.error("Sync failed:", e);
    }
  };

  return (
    <div className="p-6 max-w-4xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-xl font-semibold text-text">Dashboard</h1>
          <p className="text-sm text-text-muted mt-0.5">
            Sync your Claude Code environment across machines
          </p>
        </div>
        <button
          onClick={handleSync}
          disabled={isSyncing}
          className="btn-primary flex items-center gap-2"
        >
          <RefreshCw size={15} className={isSyncing ? "animate-spin" : ""} />
          {isSyncing ? "Syncing..." : "Sync Now"}
        </button>
      </div>

      {/* Status cards */}
      <div className="grid grid-cols-3 gap-4 mb-6">
        <div className="card">
          <div className="text-xs text-text-muted mb-1">Machine</div>
          <div className="flex items-center gap-2">
            <Server size={14} className="text-accent" />
            <span className="text-sm font-medium text-text truncate">
              {machineConfig?.machineName ?? "Loading..."}
            </span>
          </div>
        </div>
        <div className="card">
          <div className="text-xs text-text-muted mb-1">Last Synced</div>
          <div className="flex items-center gap-2">
            <Clock size={14} className="text-text-muted" />
            <span className="text-sm text-text">
              {formatRelativeTime(status?.last_synced)}
            </span>
          </div>
        </div>
        <div className="card">
          <div className="text-xs text-text-muted mb-1">Pending Changes</div>
          <div className="flex items-center gap-2">
            {status?.pending_changes ? (
              <AlertTriangle size={14} className="text-warning" />
            ) : (
              <CheckCircle size={14} className="text-success" />
            )}
            <span className="text-sm text-text">
              {status?.pending_changes ?? 0} files
            </span>
          </div>
        </div>
      </div>

      {/* Last result */}
      {lastResult && (
        <div
          className={`card mb-6 border ${
            lastResult.success ? "border-success/30 bg-success/5" : "border-warning/30 bg-warning/5"
          }`}
        >
          <div className="flex items-center gap-2 mb-1">
            {lastResult.success ? (
              <CheckCircle size={14} className="text-success" />
            ) : (
              <AlertTriangle size={14} className="text-warning" />
            )}
            <span className="text-sm font-medium text-text">{lastResult.message}</span>
          </div>
          {(lastResult.files_pushed.length > 0 || lastResult.files_pulled.length > 0) && (
            <div className="text-xs text-text-muted mt-1">
              {lastResult.files_pushed.length > 0 && (
                <span className="text-success mr-3">
                  ↑ {lastResult.files_pushed.length} pushed
                </span>
              )}
              {lastResult.files_pulled.length > 0 && (
                <span className="text-info">
                  ↓ {lastResult.files_pulled.length} pulled
                </span>
              )}
            </div>
          )}
        </div>
      )}

      <div className="grid grid-cols-2 gap-4">
        <SyncStatus />
        <PendingChanges />
      </div>
    </div>
  );
}
