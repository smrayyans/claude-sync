import { useEffect, useState } from "react";
import { GitCommit, ChevronDown, ChevronRight, Server, AlertTriangle } from "lucide-react";
import { api, type Commit } from "../../lib/tauri";
import { formatRelativeTime, cn } from "../../lib/utils";
import ConflictResolver from "./ConflictResolver";

export default function CommitHistory() {
  const [commits, setCommits] = useState<Commit[]>([]);
  const [loading, setLoading] = useState(true);
  const [selected, setSelected] = useState<string | null>(null);
  const [diff, setDiff] = useState<string>("");
  const [conflicts, setConflicts] = useState<string[]>([]);
  const [resolving, setResolving] = useState<string | null>(null);

  useEffect(() => {
    loadHistory();
  }, []);

  const loadHistory = async () => {
    setLoading(true);
    try {
      const data = await api.history.commits(50);
      setCommits(data);
    } catch {
      // Likely no sync repo yet
    } finally {
      setLoading(false);
    }
  };

  const handleSelectCommit = async (hash: string) => {
    if (selected === hash) {
      setSelected(null);
      setDiff("");
      return;
    }
    setSelected(hash);
    try {
      const d = await api.history.diff(hash);
      setDiff(d);
    } catch {
      setDiff("");
    }
  };

  if (resolving) {
    return (
      <ConflictResolver
        filePath={resolving}
        onResolved={() => {
          setResolving(null);
          setConflicts((c) => c.filter((f) => f !== resolving));
        }}
        onCancel={() => setResolving(null)}
      />
    );
  }

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-text">History</h1>
        <p className="text-sm text-text-muted mt-0.5">Sync commit history</p>
      </div>

      {conflicts.length > 0 && (
        <div className="card border-warning/30 bg-warning/5 mb-4">
          <div className="flex items-center gap-2 mb-2">
            <AlertTriangle size={14} className="text-warning" />
            <span className="text-sm font-medium text-warning">
              {conflicts.length} conflict{conflicts.length > 1 ? "s" : ""} detected
            </span>
          </div>
          <div className="space-y-1">
            {conflicts.map((f) => (
              <div key={f} className="flex items-center justify-between text-xs">
                <span className="text-text-muted font-mono">{f}</span>
                <button
                  className="btn-primary text-xs py-0.5 px-2"
                  onClick={() => setResolving(f)}
                >
                  Resolve
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {loading ? (
        <div className="text-sm text-text-muted">Loading history...</div>
      ) : commits.length === 0 ? (
        <div className="text-center py-16">
          <GitCommit size={40} className="text-text-dim mx-auto mb-3" />
          <p className="text-text-muted text-sm">No commit history yet</p>
          <p className="text-text-dim text-xs mt-1">Sync to see commits here</p>
        </div>
      ) : (
        <div className="space-y-1">
          {commits.map((commit) => (
            <div key={commit.hash}>
              <div
                className="card cursor-pointer hover:border-accent/30 transition-colors"
                onClick={() => handleSelectCommit(commit.hash)}
              >
                <div className="flex items-start gap-3">
                  <div className="mt-0.5">
                    {selected === commit.hash ? (
                      <ChevronDown size={14} className="text-text-muted" />
                    ) : (
                      <ChevronRight size={14} className="text-text-muted" />
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="text-sm text-text">{commit.message}</div>
                    <div className="flex items-center gap-3 mt-1 text-xs text-text-dim">
                      <span className="font-mono">{commit.short_hash}</span>
                      {commit.machine_name && (
                        <div className="flex items-center gap-1">
                          <Server size={10} />
                          {commit.machine_name}
                        </div>
                      )}
                      <span>{commit.files_changed} files</span>
                      <span>{formatRelativeTime(commit.timestamp)}</span>
                    </div>
                  </div>
                </div>
              </div>

              {selected === commit.hash && diff && (
                <div className="mt-1 mb-2 bg-surface-2 border border-border rounded-md p-3 font-mono text-xs overflow-x-auto max-h-64 overflow-y-auto">
                  {diff.split("\n").map((line, i) => (
                    <div
                      key={i}
                      className={cn(
                        line.startsWith("+") && !line.startsWith("+++")
                          ? "text-success"
                          : line.startsWith("-") && !line.startsWith("---")
                          ? "text-error"
                          : "text-text-muted"
                      )}
                    >
                      {line || " "}
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
