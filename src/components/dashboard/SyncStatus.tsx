import { Wifi, WifiOff, RefreshCw } from "lucide-react";
import { useSyncStore } from "../../stores/syncStore";
import { useSettingsStore } from "../../stores/settingsStore";

export default function SyncStatus() {
  const { status, isSyncing } = useSyncStore();
  const { machineConfig } = useSettingsStore();

  return (
    <div className="card">
      <h3 className="text-sm font-medium text-text mb-3">Sync Status</h3>
      <div className="space-y-2">
        <div className="flex items-center justify-between text-sm">
          <span className="text-text-muted">Connection</span>
          <div className="flex items-center gap-1.5">
            {status?.is_online ? (
              <>
                <Wifi size={12} className="text-success" />
                <span className="text-success">Online</span>
              </>
            ) : (
              <>
                <WifiOff size={12} className="text-text-dim" />
                <span className="text-text-dim">Offline</span>
              </>
            )}
          </div>
        </div>
        <div className="flex items-center justify-between text-sm">
          <span className="text-text-muted">Auto-sync</span>
          <span className="text-text">
            Every {machineConfig?.autoSyncInterval ?? 15}m
          </span>
        </div>
        <div className="flex items-center justify-between text-sm">
          <span className="text-text-muted">Remote</span>
          <span className="text-text text-xs truncate max-w-32">
            {machineConfig?.remoteUrl
              ? new URL(machineConfig.remoteUrl).pathname.slice(1)
              : "Not set"}
          </span>
        </div>
        {isSyncing && (
          <div className="flex items-center gap-2 text-sm text-accent pt-1">
            <RefreshCw size={12} className="animate-spin" />
            <span>Syncing...</span>
          </div>
        )}
        {status?.error && (
          <div className="text-xs text-error pt-1">{status.error}</div>
        )}
      </div>
    </div>
  );
}
