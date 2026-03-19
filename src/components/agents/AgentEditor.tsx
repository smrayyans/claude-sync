import { useState } from "react";
import Editor from "@monaco-editor/react";
import { ArrowLeft, Save } from "lucide-react";
import { useAgentStore } from "../../stores/agentStore";

interface Props {
  onBack: () => void;
}

export default function AgentEditor({ onBack }: Props) {
  const { selectedAgent, saveAgent } = useAgentStore();
  const [content, setContent] = useState(selectedAgent?.content ?? "");
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  if (!selectedAgent) return null;

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveAgent({ ...selectedAgent, content });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-3 p-4 border-b border-border bg-surface">
        <button onClick={onBack} className="btn-ghost p-1.5">
          <ArrowLeft size={16} />
        </button>
        <div className="flex-1 min-w-0">
          <div className="text-sm font-medium text-text">{selectedAgent.name}</div>
          <div className="text-xs text-text-muted truncate">{selectedAgent.path || "New file"}</div>
        </div>
        <button
          onClick={handleSave}
          disabled={saving}
          className="btn-primary flex items-center gap-2 text-sm"
        >
          <Save size={14} />
          {saving ? "Saving..." : saved ? "Saved!" : "Save"}
        </button>
      </div>
      <div className="flex-1">
        <Editor
          height="100%"
          defaultLanguage="markdown"
          value={content}
          onChange={(v) => setContent(v ?? "")}
          theme="vs-dark"
          options={{
            fontSize: 13,
            fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
            lineNumbers: "on",
            minimap: { enabled: false },
            wordWrap: "on",
            scrollBeyondLastLine: false,
            padding: { top: 16 },
          }}
        />
      </div>
    </div>
  );
}
