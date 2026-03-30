import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X, FileText, Trash2, Plus, Edit3 } from "lucide-react";
import { useSyncStore } from "../../stores/syncStore";
import { getChangeTypeColor, getChangeTypeSymbol, formatBytes, cn } from "../../lib/utils";

interface FilePreview {
  file_key: string;
  local_path: string;
  local_content: string | null;
  sync_content: string | null;
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

function changeIcon(type: string) {
  if (type === "deleted") return <Trash2 size={11} className="text-error" />;
  if (type === "added") return <Plus size={11} className="text-success" />;
  return <Edit3 size={11} className="text-warning" />;
}

export default function PendingChanges() {
  const { repoStatus } = useSyncStore();
  const changes = repoStatus?.local_changes ?? [];
  const [preview, setPreview] = useState<FilePreview | null>(null);
  const [loadingKey, setLoadingKey] = useState<string | null>(null);

  const openPreview = async (path: string) => {
    if (loadingKey === path) return;
    setLoadingKey(path);
    try {
      const data = await invoke<FilePreview>("get_file_preview", { fileKey: path });
      setPreview(data);
    } catch (e) {
      console.error(e);
    } finally {
      setLoadingKey(null);
    }
  };

  return (
    <>
      <div className="card">
        <h3 className="text-sm font-medium text-text mb-3">
          Local Changes
          {changes.length > 0 && (
            <span className="ml-2 badge-yellow">{changes.length}</span>
          )}
        </h3>
        {changes.length === 0 ? (
          <div className="text-sm text-text-dim text-center py-4">
            {repoStatus ? "Nothing to push" : "Hit Refresh to check"}
          </div>
        ) : (
          <div className="space-y-1 max-h-48 overflow-y-auto">
            {changes.map((change) => (
              <button
                key={change.path}
                onClick={() => openPreview(change.path)}
                disabled={loadingKey === change.path}
                className={cn(
                  "w-full flex items-center gap-2 text-xs font-mono px-1.5 py-1 rounded hover:bg-surface-2 transition-colors text-left",
                  loadingKey === change.path && "opacity-50"
                )}
                title="Click to preview"
              >
                <span className={`font-bold w-3 ${getChangeTypeColor(change.change_type)}`}>
                  {getChangeTypeSymbol(change.change_type)}
                </span>
                {changeIcon(change.change_type)}
                <span className="text-text-muted flex-1 truncate">{formatFileKey(change.path)}</span>
                {change.size_bytes !== undefined && (
                  <span className="text-text-dim">{formatBytes(change.size_bytes)}</span>
                )}
                <span className="text-text-dim opacity-50 text-[10px]">preview</span>
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Preview modal */}
      {preview && (
        <div
          className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 p-4"
          onClick={() => setPreview(null)}
        >
          <div
            className="bg-surface rounded-xl border border-border w-full max-w-2xl max-h-[80vh] flex flex-col shadow-xl"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div className="flex items-start justify-between p-4 border-b border-border flex-shrink-0">
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2 mb-0.5">
                  <FileText size={13} className="text-accent flex-shrink-0" />
                  <span className="text-sm font-medium text-text truncate">
                    {formatFileKey(preview.file_key)}
                  </span>
                </div>
                <div
                  className="text-xs text-text-dim font-mono truncate cursor-pointer hover:text-text-muted"
                  title={preview.local_path}
                  onClick={() => navigator.clipboard?.writeText(preview.local_path)}
                >
                  {preview.local_path}
                </div>
              </div>
              <button onClick={() => setPreview(null)} className="btn-ghost p-1.5 ml-3 flex-shrink-0">
                <X size={14} />
              </button>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              {preview.local_content === null ? (
                <div className="text-sm text-error bg-error/10 border border-error/20 rounded-lg p-4">
                  This file has been deleted locally and will be removed from the remote when pushed.
                  {preview.sync_content && (
                    <div className="mt-3">
                      <div className="text-xs text-text-muted mb-2 font-sans">Last committed version:</div>
                      <pre className="text-xs text-text-dim bg-background rounded p-3 overflow-auto max-h-48 whitespace-pre-wrap">
                        {preview.sync_content}
                      </pre>
                    </div>
                  )}
                </div>
              ) : (
                <>
                  {preview.sync_content && preview.sync_content !== preview.local_content && (
                    <div>
                      <div className="text-xs text-text-muted mb-2 font-sans">Last synced version:</div>
                      <pre className="text-xs text-text-dim bg-background rounded p-3 overflow-auto max-h-48 whitespace-pre-wrap border border-border">
                        {preview.sync_content}
                      </pre>
                    </div>
                  )}
                  <div>
                    <div className="text-xs text-text-muted mb-2 font-sans">
                      {preview.sync_content === null ? "New file (not yet synced):" : "Current local version:"}
                    </div>
                    <pre className="text-xs text-text bg-background rounded p-3 overflow-auto max-h-64 whitespace-pre-wrap border border-success/30">
                      {preview.local_content}
                    </pre>
                  </div>
                </>
              )}
            </div>
          </div>
        </div>
      )}
    </>
  );
}
