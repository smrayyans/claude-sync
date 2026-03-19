import { useState, useEffect } from "react";
import Editor from "@monaco-editor/react";
import { AlertTriangle, Check, X } from "lucide-react";
import { api } from "../../lib/tauri";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  filePath: string;
  onResolved: () => void;
  onCancel: () => void;
}

export default function ConflictResolver({ filePath, onResolved, onCancel }: Props) {
  const [localContent, setLocalContent] = useState("");
  const [remoteContent, setRemoteContent] = useState("");
  const [manualContent, setManualContent] = useState("");
  const [resolving, setResolving] = useState(false);
  const [mode, setMode] = useState<"compare" | "manual">("compare");

  useEffect(() => {
    // Load local content
    invoke<string>("get_memory_file", { path: filePath })
      .then((f: any) => setLocalContent(f.content ?? ""))
      .catch(() => {});
  }, [filePath]);

  const resolve = async (type: "Mine" | "Theirs" | "Manual") => {
    setResolving(true);
    try {
      let resolution;
      if (type === "Manual") {
        resolution = { Manual: manualContent };
      } else {
        resolution = { [type]: null };
      }
      await api.history.resolve(filePath, resolution as any);
      onResolved();
    } finally {
      setResolving(false);
    }
  };

  return (
    <div className="p-6 flex flex-col h-full">
      <div className="flex items-center gap-3 mb-4">
        <AlertTriangle size={18} className="text-warning" />
        <div>
          <h1 className="text-lg font-semibold text-text">Conflict Resolver</h1>
          <p className="text-xs text-text-muted font-mono">{filePath}</p>
        </div>
        <button onClick={onCancel} className="ml-auto btn-ghost p-1.5">
          <X size={16} />
        </button>
      </div>

      <div className="flex gap-2 mb-4">
        <button
          className={`text-sm px-3 py-1.5 rounded-md transition-colors ${
            mode === "compare"
              ? "bg-accent text-white"
              : "bg-surface-2 text-text-muted hover:text-text"
          }`}
          onClick={() => setMode("compare")}
        >
          Compare
        </button>
        <button
          className={`text-sm px-3 py-1.5 rounded-md transition-colors ${
            mode === "manual"
              ? "bg-accent text-white"
              : "bg-surface-2 text-text-muted hover:text-text"
          }`}
          onClick={() => setMode("manual")}
        >
          Manual Edit
        </button>
      </div>

      {mode === "compare" ? (
        <div className="flex-1 grid grid-cols-2 gap-3 min-h-0">
          <div className="flex flex-col">
            <div className="text-xs text-text-muted mb-1.5 flex items-center justify-between">
              <span>Local (Mine)</span>
              <button
                onClick={() => resolve("Mine")}
                disabled={resolving}
                className="btn-primary text-xs py-0.5 px-2 flex items-center gap-1"
              >
                <Check size={10} />
                Use Mine
              </button>
            </div>
            <div className="flex-1 border border-border rounded-md overflow-hidden">
              <Editor
                height="100%"
                defaultLanguage="markdown"
                value={localContent}
                theme="vs-dark"
                options={{ readOnly: true, minimap: { enabled: false }, fontSize: 12 }}
              />
            </div>
          </div>
          <div className="flex flex-col">
            <div className="text-xs text-text-muted mb-1.5 flex items-center justify-between">
              <span>Remote (Theirs)</span>
              <button
                onClick={() => resolve("Theirs")}
                disabled={resolving}
                className="btn-secondary text-xs py-0.5 px-2 flex items-center gap-1"
              >
                <Check size={10} />
                Use Theirs
              </button>
            </div>
            <div className="flex-1 border border-border rounded-md overflow-hidden">
              <Editor
                height="100%"
                defaultLanguage="markdown"
                value={remoteContent || "(Remote content not available — sync to load)"}
                theme="vs-dark"
                options={{ readOnly: true, minimap: { enabled: false }, fontSize: 12 }}
              />
            </div>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex flex-col min-h-0">
          <div className="text-xs text-text-muted mb-1.5">Edit the resolved content:</div>
          <div className="flex-1 border border-border rounded-md overflow-hidden">
            <Editor
              height="100%"
              defaultLanguage="markdown"
              value={manualContent || localContent}
              onChange={(v) => setManualContent(v ?? "")}
              theme="vs-dark"
              options={{ minimap: { enabled: false }, fontSize: 12 }}
            />
          </div>
          <button
            onClick={() => resolve("Manual")}
            disabled={resolving}
            className="btn-primary mt-3 flex items-center gap-2 text-sm self-end"
          >
            <Check size={14} />
            Apply Resolution
          </button>
        </div>
      )}
    </div>
  );
}
