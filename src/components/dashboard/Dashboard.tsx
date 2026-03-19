import { useEffect, useState } from "react";
import {
  RefreshCw,
  Server,
  Clock,
  AlertTriangle,
  CheckCircle,
  Download,
  Upload,
  Monitor,
  ArrowDown,
  ArrowUp,
  Bug,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useSyncStore } from "../../stores/syncStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { formatRelativeTime } from "../../lib/utils";
import PendingChanges from "./PendingChanges";
import SyncStatus from "./SyncStatus";

interface PushDiagnostic {
  remote_url: string | null;
  token_found: boolean;
  sync_repo_exists: boolean;
  sync_repo_path: string;
  remote_has_data: boolean;
  head_commit: string | null;
  commits_ahead: number;
  tracked_files_count: number;
  files_to_push: string[];
  error: string | null;
}

export default function Dashboard() {
  const {
    status,
    isSyncing,
    isRefreshing,
    repoStatus,
    lastResult,
    pullLog,
    pullNow,
    pushNow,
    checkRepoStatus,
    refreshStatus,
    refreshPullLog,
  } = useSyncStore();
  const [diag, setDiag] = useState<PushDiagnostic | null>(null);
  const [diagLoading, setDiagLoading] = useState(false);
  const { machineConfig } = useSettingsStore();

  useEffect(() => {
    refreshStatus();
    checkRepoStatus();
    refreshPullLog();
    const interval = setInterval(() => {
      refreshStatus();
    }, 30_000);
    return () => clearInterval(interval);
  }, []);

  const handleRefresh = async () => {
    try { await checkRepoStatus(); } catch (e) { console.error(e); }
  };

  const handleDiagnose = async () => {
    setDiagLoading(true);
    setDiag(null);
    try {
      const result = await invoke<PushDiagnostic>("diagnose_push");
      setDiag(result);
    } catch (e) {
      setDiag({ remote_url: null, token_found: false, sync_repo_exists: false,
        sync_repo_path: "", remote_has_data: false, head_commit: null,
        commits_ahead: 0, tracked_files_count: 0, files_to_push: [],
        error: String(e) });
    } finally {
      setDiagLoading(false);
    }
  };
  const handlePull = async () => {
    try { await pullNow(); await checkRepoStatus(); } catch (e) { console.error(e); }
  };
  const handlePush = async () => {
    try { await pushNow(); await checkRepoStatus(); } catch (e) { console.error(e); }
  };

  const localCount = repoStatus?.local_changes.length ?? 0;
  const remoteCount = repoStatus?.commits_behind ?? 0;

  return (
    <div className="p-6 max-w-4xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-xl font-semibold text-text">Dashboard</h1>
          <p className="text-sm text-text-muted mt-0.5">
            Sync your Claude Code environment across machines
          </p>
        </div>
        {/* Action buttons */}
        <div className="flex items-center gap-2">
          <button
            onClick={handlePull}
            disabled={isSyncing || isRefreshing}
            title="Pull — apply remote changes to this machine"
            className="btn-secondary flex items-center gap-1.5 text-sm"
          >
            <Download size={14} className={isSyncing ? "animate-bounce" : ""} />
            Pull
            {remoteCount > 0 && (
              <span className="badge-blue ml-0.5">{remoteCount}</span>
            )}
          </button>
          <button
            onClick={handlePush}
            disabled={isSyncing || isRefreshing}
            title="Push — commit local changes to remote"
            className="btn-secondary flex items-center gap-1.5 text-sm"
          >
            <Upload size={14} className={isSyncing ? "animate-bounce" : ""} />
            Push
            {localCount > 0 && (
              <span className="badge-yellow ml-0.5">{localCount}</span>
            )}
          </button>
          <button
            onClick={handleRefresh}
            disabled={isRefreshing || isSyncing}
            title="Check what's new on GitHub vs local"
            className="btn-primary flex items-center gap-2"
          >
            <RefreshCw size={15} className={isRefreshing ? "animate-spin" : ""} />
            {isRefreshing ? "Checking..." : "Refresh"}
          </button>
          <button
            onClick={handleDiagnose}
            disabled={diagLoading}
            title="Diagnose push issues"
            className="btn-secondary flex items-center gap-1.5 text-sm"
          >
            <Bug size={14} className={diagLoading ? "animate-pulse" : ""} />
            {diagLoading ? "..." : "Diagnose"}
          </button>
        </div>
      </div>

      {/* Status banners */}
      {repoStatus && !repoStatus.error && (remoteCount > 0 || localCount > 0) && (
        <div className="flex gap-3 mb-4">
          {remoteCount > 0 && (
            <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-info/10 border border-info/20 text-sm">
              <ArrowDown size={13} className="text-info" />
              <span className="text-info font-medium">{remoteCount} new commit{remoteCount !== 1 ? "s" : ""} on GitHub</span>
              <span className="text-text-muted">— hit Pull</span>
            </div>
          )}
          {localCount > 0 && (
            <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-warning/10 border border-warning/20 text-sm">
              <ArrowUp size={13} className="text-warning" />
              <span className="text-warning font-medium">{localCount} local file{localCount !== 1 ? "s" : ""} not on GitHub</span>
              <span className="text-text-muted">— hit Push</span>
            </div>
          )}
        </div>
      )}
      {repoStatus && !repoStatus.error && remoteCount === 0 && localCount === 0 && (
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-success/10 border border-success/20 text-sm mb-4">
          <CheckCircle size={13} className="text-success" />
          <span className="text-success font-medium">Everything is in sync</span>
        </div>
      )}
      {repoStatus?.error && (
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-error/10 border border-error/20 text-sm mb-4">
          <AlertTriangle size={13} className="text-error" />
          <span className="text-error">{repoStatus.error}</span>
        </div>
      )}

      {/* Diagnostic panel */}
      {diag && (
        <div className="card mb-4 border border-accent/20 bg-surface-alt text-xs font-mono">
          <div className="flex items-center gap-2 mb-2">
            <Bug size={13} className="text-accent" />
            <span className="text-sm font-sans font-medium text-text">Push Diagnostic</span>
            <button onClick={() => setDiag(null)} className="ml-auto text-text-dim hover:text-text">✕</button>
          </div>
          <div className="space-y-1 text-text-muted">
            <div><span className="text-text">remote_url:</span> {diag.remote_url ?? "NOT SET"}</div>
            <div><span className="text-text">token_found:</span> <span className={diag.token_found ? "text-success" : "text-error"}>{String(diag.token_found)}</span></div>
            <div><span className="text-text">sync_repo_exists:</span> <span className={diag.sync_repo_exists ? "text-success" : "text-warning"}>{String(diag.sync_repo_exists)}</span> <span className="text-text-dim">({diag.sync_repo_path})</span></div>
            <div><span className="text-text">remote_has_data:</span> {String(diag.remote_has_data)}</div>
            <div><span className="text-text">head_commit:</span> {diag.head_commit ?? "no commits"}</div>
            <div><span className="text-text">commits_ahead:</span> <span className={diag.commits_ahead > 0 ? "text-warning" : ""}>{diag.commits_ahead}</span></div>
            <div><span className="text-text">tracked_files:</span> {diag.tracked_files_count}</div>
            <div><span className="text-text">files_to_push:</span> <span className={diag.files_to_push.length > 0 ? "text-warning" : "text-success"}>{diag.files_to_push.length}</span></div>
            {diag.files_to_push.length > 0 && (
              <div className="ml-4 space-y-0.5">
                {diag.files_to_push.map((f, i) => <div key={i} className="text-warning">+ {f}</div>)}
              </div>
            )}
            {diag.error && <div className="text-error mt-1">error: {diag.error}</div>}
          </div>
        </div>
      )}

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
          <div className="text-xs text-text-muted mb-1">Local Changes</div>
          <div className="flex items-center gap-2">
            {localCount > 0 ? (
              <ArrowUp size={14} className="text-warning" />
            ) : (
              <CheckCircle size={14} className="text-success" />
            )}
            <span className="text-sm text-text">
              {localCount} file{localCount !== 1 ? "s" : ""} to push
            </span>
          </div>
        </div>
      </div>

      {/* Last result */}
      {lastResult && (
        <div
          className={`card mb-6 border ${
            lastResult.success
              ? "border-success/30 bg-success/5"
              : "border-warning/30 bg-warning/5"
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

      <div className="grid grid-cols-2 gap-4 mb-6">
        <SyncStatus />
        <PendingChanges />
      </div>

      {/* Pull log */}
      <div className="card">
        <div className="flex items-center gap-2 mb-3">
          <Monitor size={14} className="text-accent" />
          <span className="text-sm font-medium text-text">Device Pull History</span>
          <span className="text-xs text-text-muted ml-auto">who pulled last</span>
        </div>
        {pullLog.length === 0 ? (
          <p className="text-xs text-text-muted">No pulls recorded yet.</p>
        ) : (
          <div className="space-y-1.5 max-h-48 overflow-y-auto">
            {[...pullLog].reverse().map((entry, i) => (
              <div key={i} className="flex items-center justify-between text-xs">
                <div className="flex items-center gap-2">
                  <Download size={11} className="text-info shrink-0" />
                  <span className="text-text font-medium">{entry.machineName}</span>
                  <span className="text-text-muted font-mono text-[10px]">
                    {entry.machineId.slice(0, 8)}
                  </span>
                </div>
                <span className="text-text-muted">{formatRelativeTime(entry.timestamp)}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
