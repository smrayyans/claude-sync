import { useState } from "react";
import { FolderOpen, RotateCcw } from "lucide-react";
import { useSettingsStore } from "../../stores/settingsStore";

interface PathField {
  key: keyof CustomPaths;
  label: string;
  defaultHint: string;
}

interface CustomPaths {
  claudeDir?: string;
  agentsDir?: string;
  skillsDir?: string;
  projectsDir?: string;
}

const FIELDS: PathField[] = [
  { key: "claudeDir",    label: "Claude directory",  defaultHint: "~/.claude" },
  { key: "agentsDir",    label: "Agents directory",   defaultHint: "~/.claude/agents" },
  { key: "skillsDir",    label: "Skills directory",   defaultHint: "~/.claude/skills" },
  { key: "projectsDir",  label: "Projects directory", defaultHint: "~/.claude/projects" },
];

export default function PathSettings() {
  const { machineConfig, saveMachineConfig } = useSettingsStore();
  const existing = (machineConfig as any)?.customPaths ?? {};

  const [paths, setPaths] = useState<CustomPaths>({
    claudeDir:   existing.claudeDir   ?? "",
    agentsDir:   existing.agentsDir   ?? "",
    skillsDir:   existing.skillsDir   ?? "",
    projectsDir: existing.projectsDir ?? "",
  });
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const handleSave = async () => {
    if (!machineConfig) return;
    setSaving(true);
    try {
      // Strip empty strings (use defaults)
      const cleaned: Record<string, string> = {};
      for (const [k, v] of Object.entries(paths)) {
        if (v.trim()) cleaned[k] = v.trim();
      }
      await saveMachineConfig({
        ...machineConfig,
        customPaths: cleaned,
      } as any);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  const reset = (key: keyof CustomPaths) => {
    setPaths((p) => ({ ...p, [key]: "" }));
  };

  return (
    <div className="space-y-5">
      <div>
        <p className="text-sm text-text-muted mb-4">
          Override the default paths claude-sync reads from. Leave blank to use defaults.
          Useful if your Claude installation is in a non-standard location or you want to
          sync a different directory.
        </p>
      </div>

      {FIELDS.map(({ key, label, defaultHint }) => (
        <div key={key}>
          <label className="block text-sm text-text-muted mb-1.5">{label}</label>
          <div className="flex gap-2">
            <div className="relative flex-1">
              <FolderOpen size={13} className="absolute left-3 top-1/2 -translate-y-1/2 text-text-dim" />
              <input
                type="text"
                className="input pl-8 text-sm font-mono"
                placeholder={defaultHint}
                value={paths[key] ?? ""}
                onChange={(e) =>
                  setPaths((p) => ({ ...p, [key]: e.target.value }))
                }
              />
            </div>
            {paths[key] && (
              <button
                onClick={() => reset(key)}
                className="btn-ghost px-2 text-text-dim hover:text-text"
                title="Reset to default"
              >
                <RotateCcw size={13} />
              </button>
            )}
          </div>
          {paths[key] && (
            <p className="text-xs text-accent mt-1 font-mono">→ {paths[key]}</p>
          )}
        </div>
      ))}

      <div className="pt-2 border-t border-border">
        <p className="text-xs text-text-dim mb-3">
          Changes take effect after restarting claude-sync.
        </p>
        <button onClick={handleSave} disabled={saving} className="btn-primary text-sm">
          {saving ? "Saving..." : saved ? "Saved!" : "Save Path Overrides"}
        </button>
      </div>
    </div>
  );
}
