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
  X,
  Trash2,
  Edit3,
  Plus,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useSyncStore, FileChange, SyncResult } from "../../stores/syncStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { formatRelativeTime, formatBytes, cn } from "../../lib/utils";
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

function formatFileKey(key: string): string {
  if (key === "settings.json") return "Settings";
  if (key.startsWith("agents/")) return `Agent: ${key.slice("agents/".length).replace(/\.md$/, "")}`;
  if (key.startsWith("plans/")) return `Plan: ${key.slice("plans/".length).replace(/\.md$/, "")}`;
  if (key.startsWith("plugins/")) return `Plugin: ${key.slice("plugins/".length)}`;
  if (key.startsWith("skills/")) return `Skill: ${key.slice("skills/".length).replace(/\.md$/, "")}`;

  const chatMatch = key.match(/^projects\/([^/]+)\/([0-9a-f-]{36})\.jsonl$/);
  if (chatMatch) {
    const proj = chatMatch[1].replace("_HOME_", "~").replace(/-/g, "/");
    return `Chat [${proj}] ${chatMatch[2].slice(0, 8)}`;
  }

  const memMatch = key.match(/^projects\/([^/]+)\/memory\/(.+)$/);
  if (memMatch) {
    const proj = memMatch[1].replace("_HOME_", "~").replace(/-/g, "/");
    return `Memory [${proj}]: ${memMatch[2]}`;
  }

  return key;
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

  // Selective push dialog state
  const [showPushDialog, setShowPushDialog] = useState(false);
  const [pushSelected, setPushSelected] = useState<Set<string>>(new Set());

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

  // Open selective push dialog, pre-select all pending changes
  const handlePushClick = () => {
    const changes = repoStatus?.local_changes ?? [];
    if (changes.length === 0) {
      // Nothing pending — push anyway (will push any ahead commits)
      pushNow().then(() => checkRepoStatus());
      return;
    }
    setPushSelected(new Set(changes.map((c) => c.path)));
    setShowPushDialog(true);
  };

  const handlePushConfirm = async () => {
    setShowPushDialog(false);
    const selected = Array.from(pushSelected);
    const allPaths = (repoStatus?.local_changes ?? []).map((c) => c.path);
    const isAll = selected.length === allPaths.length;

    try {
      let result: SyncResult;
      if (isAll) {
        result = await pushNow();
      } else {
        const { setIsSyncing } = useSyncStore.getState();
        setIsSyncing(true);
        result = await invoke<SyncResult>("sync_push_selective", { fileKeys: selected });
        setIsSyncing(false);
        useSyncStore.setState({ lastResult: result });
        await refreshStatus();
        await useSyncStore.getState().refreshPending();
      }
      await checkRepoStatus();
    } catch (e) {
      console.error(e);
    }
  };

  const localCount = repoStatus?.local_changes.length ?? 0;
  const remoteCount = repoStatus?.commits_behind ?? 0;

  return (
    <>
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
            onClick={handlePushClick}
            disabled={isSyncing || isRefreshing}
            title="Push — select and commit local changes to remote"
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
          {(lastResult.files_pushed.length > 0 || lastResult.files_pulled.length > 0 || lastResult.conflicts.length > 0) && (
            <div className="text-xs text-text-muted mt-2 space-y-1">
              {lastResult.files_pushed.length > 0 && (
                <div>
                  <span className="text-success font-medium">↑ {lastResult.files_pushed.length} pushed</span>
                  <ul className="ml-3 mt-0.5 space-y-0.5">
                    {lastResult.files_pushed.map((f) => (
                      <li key={f} className="text-text-dim">{formatFileKey(f)}</li>
                    ))}
                  </ul>
                </div>
              )}
              {lastResult.files_pulled.length > 0 && (
                <div>
                  <span className="text-info font-medium">↓ {lastResult.files_pulled.length} pulled</span>
                  <ul className="ml-3 mt-0.5 space-y-0.5">
                    {lastResult.files_pulled.map((f) => (
                      <li key={f} className="text-text-dim">{formatFileKey(f)}</li>
                    ))}
                  </ul>
                </div>
              )}
              {lastResult.conflicts.length > 0 && (
                <div>
                  <span className="text-warning font-medium">⚠ {lastResult.conflicts.length} conflict{lastResult.conflicts.length > 1 ? "s" : ""} (local not overwritten)</span>
                  <ul className="ml-3 mt-0.5 space-y-0.5">
                    {lastResult.conflicts.map((f) => (
                      <li key={f} className="text-text-dim">{formatFileKey(f)}</li>
                    ))}
                  </ul>
                </div>
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

    {/* Selective push dialog */}
    {showPushDialog && (
      <div
        className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4"
        onClick={() => setShowPushDialog(false)}
      >
        <div
          className="bg-surface rounded-xl border border-border w-full max-w-lg shadow-xl flex flex-col max-h-[80vh]"
          onClick={(e) => e.stopPropagation()}
        >
          <div className="flex items-center justify-between p-4 border-b border-border flex-shrink-0">
            <div>
              <h3 className="text-base font-semibold text-text">Push to GitHub</h3>
              <p className="text-xs text-text-muted mt-0.5">
                Select which changes to push. Uncheck items to skip them this time.
              </p>
            </div>
            <button onClick={() => setShowPushDialog(false)} className="btn-ghost p-1.5">
              <X size={14} />
            </button>
          </div>

          <div className="flex-1 overflow-y-auto p-4 space-y-1">
            {/* Select all / none */}
            <div className="flex gap-3 mb-3 text-xs">
              <button
                onClick={() => setPushSelected(new Set((repoStatus?.local_changes ?? []).map((c) => c.path)))}
                className="text-accent hover:underline"
              >
                Select all
              </button>
              <button
                onClick={() => setPushSelected(new Set())}
                className="text-text-muted hover:underline"
              >
                None
              </button>
            </div>

            {(repoStatus?.local_changes ?? []).map((change) => {
              const checked = pushSelected.has(change.path);
              const isDeletion = change.change_type === "deleted";
              return (
                <label
                  key={change.path}
                  className={cn(
                    "flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-surface-2 transition-colors",
                    isDeletion && "border border-error/20 bg-error/5"
                  )}
                >
                  <input
                    type="checkbox"
                    checked={checked}
                    onChange={(e) => {
                      setPushSelected((prev) => {
                        const next = new Set(prev);
                        if (e.target.checked) next.add(change.path);
                        else next.delete(change.path);
                        return next;
                      });
                    }}
                    className="rounded flex-shrink-0"
                  />
                  <div className="flex-shrink-0">
                    {isDeletion
                      ? <Trash2 size={12} className="text-error" />
                      : change.change_type === "added"
                        ? <Plus size={12} className="text-success" />
                        : <Edit3 size={12} className="text-warning" />
                    }
                  </div>
                  <span className={cn("flex-1 text-xs truncate", isDeletion ? "text-error" : "text-text")}>
                    {formatFileKey(change.path)}
                    {isDeletion && <span className="ml-1 text-[10px] text-error/70">(will be deleted from remote)</span>}
                  </span>
                  {change.size_bytes !== undefined && !isDeletion && (
                    <span className="text-xs text-text-dim flex-shrink-0">{formatBytes(change.size_bytes)}</span>
                  )}
                </label>
              );
            })}
          </div>

          <div className="flex gap-2 justify-end p-4 border-t border-border flex-shrink-0">
            <button
              onClick={() => setShowPushDialog(false)}
              className="btn-ghost px-4 py-2 text-sm"
            >
              Cancel
            </button>
            <button
              onClick={handlePushConfirm}
              disabled={pushSelected.size === 0}
              className="px-4 py-2 text-sm rounded-lg bg-accent/20 text-accent hover:bg-accent/30 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              Push {pushSelected.size} file{pushSelected.size !== 1 ? "s" : ""}
            </button>
          </div>
        </div>
      </div>
    )}
    </>
  );
}
