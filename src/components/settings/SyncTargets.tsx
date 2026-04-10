import { useState } from "react";
import { Plus, Trash2, ChevronDown, ChevronUp } from "lucide-react";
import { useSettingsStore, ExtraSyncTarget } from "../../stores/settingsStore";
import { cn } from "../../lib/utils";

const OBSIDIAN_DEFAULTS: ExtraSyncTarget = {
  name: "obsidian",
  local_path: "~/Documents/Obsidian",
  enabled: true,
  exclude_patterns: [
    ".obsidian/workspace.json",
    ".obsidian/workspace-mobile.json",
    ".obsidian/cache",
    ".trash/",
  ],
};

function TargetCard({
  target,
  onChange,
  onRemove,
}: {
  target: ExtraSyncTarget;
  onChange: (t: ExtraSyncTarget) => void;
  onRemove: () => void;
}) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="border border-border rounded-lg overflow-hidden">
      {/* Header row */}
      <div className="flex items-center gap-3 px-3 py-2.5 bg-surface-2">
        <button
          onClick={() => setExpanded((e) => !e)}
          className="flex items-center gap-2 flex-1 text-left min-w-0"
        >
          {expanded ? (
            <ChevronUp size={13} className="text-text-dim flex-shrink-0" />
          ) : (
            <ChevronDown size={13} className="text-text-dim flex-shrink-0" />
          )}
          <span className="text-sm font-medium text-text truncate">{target.name || "Unnamed"}</span>
          <span className="text-xs text-text-dim font-mono truncate">{target.local_path}</span>
        </button>

        {/* Enabled toggle */}
        <label className="flex items-center gap-1.5 flex-shrink-0 cursor-pointer">
          <input
            type="checkbox"
            checked={target.enabled}
            onChange={(e) => onChange({ ...target, enabled: e.target.checked })}
            className="w-3.5 h-3.5 accent-accent"
          />
          <span className="text-xs text-text-muted">Enabled</span>
        </label>

        <button
          onClick={onRemove}
          className="btn-ghost p-1 text-text-dim hover:text-error flex-shrink-0"
          title="Remove target"
        >
          <Trash2 size={13} />
        </button>
      </div>

      {/* Expanded settings */}
      {expanded && (
        <div className="px-3 py-3 space-y-3 border-t border-border">
          <div>
            <label className="block text-xs text-text-muted mb-1">Name (used as repo prefix)</label>
            <input
              type="text"
              className="input text-sm font-mono w-full"
              placeholder="obsidian"
              value={target.name}
              onChange={(e) => onChange({ ...target, name: e.target.value.replace(/[^a-z0-9_-]/gi, "-").toLowerCase() })}
            />
          </div>

          <div>
            <label className="block text-xs text-text-muted mb-1">Local path</label>
            <input
              type="text"
              className="input text-sm font-mono w-full"
              placeholder="~/Documents/Obsidian"
              value={target.local_path}
              onChange={(e) => onChange({ ...target, local_path: e.target.value })}
            />
          </div>

          <div>
            <label className="block text-xs text-text-muted mb-1">
              Exclude patterns (one per line, relative to vault root)
            </label>
            <textarea
              className="input text-xs font-mono w-full resize-none"
              rows={4}
              value={target.exclude_patterns.join("\n")}
              onChange={(e) =>
                onChange({
                  ...target,
                  exclude_patterns: e.target.value.split("\n").map((l) => l.trim()).filter(Boolean),
                })
              }
            />
          </div>
        </div>
      )}
    </div>
  );
}

export default function SyncTargets() {
  const { machineConfig, saveMachineConfig } = useSettingsStore();
  const [targets, setTargets] = useState<ExtraSyncTarget[]>(
    machineConfig?.extraSyncTargets ?? []
  );
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const update = (i: number, t: ExtraSyncTarget) =>
    setTargets((ts) => ts.map((x, idx) => (idx === i ? t : x)));

  const remove = (i: number) =>
    setTargets((ts) => ts.filter((_, idx) => idx !== i));

  const addObsidian = () => {
    const alreadyExists = targets.some((t) => t.name === "obsidian");
    if (alreadyExists) return;
    setTargets((ts) => [...ts, { ...OBSIDIAN_DEFAULTS }]);
  };

  const addCustom = () => {
    setTargets((ts) => [
      ...ts,
      { name: "custom", local_path: "~/", enabled: true, exclude_patterns: [] },
    ]);
  };

  const handleSave = async () => {
    if (!machineConfig) return;
    setSaving(true);
    try {
      await saveMachineConfig({ ...machineConfig, extraSyncTargets: targets });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  const obsidianExists = targets.some((t) => t.name === "obsidian");

  return (
    <div className="space-y-5">
      <div>
        <p className="text-sm text-text-muted mb-4">
          Add additional directories to sync alongside <code className="text-xs bg-surface-2 px-1 py-0.5 rounded">~/.claude</code>.
          Each target is stored under <code className="text-xs bg-surface-2 px-1 py-0.5 rounded">vaults/&lt;name&gt;/</code> in the git repo.
          Each machine stores its own local path — configure the correct path on each device after pulling.
        </p>

        <div className="flex gap-2 mb-4">
          <button
            onClick={addObsidian}
            disabled={obsidianExists}
            className={cn(
              "btn-primary text-sm",
              obsidianExists && "opacity-50 cursor-not-allowed"
            )}
          >
            <Plus size={13} className="inline mr-1" />
            Add Obsidian Vault
          </button>
          <button onClick={addCustom} className="btn-secondary text-sm">
            <Plus size={13} className="inline mr-1" />
            Add Custom Target
          </button>
        </div>
      </div>

      {targets.length === 0 ? (
        <div className="text-sm text-text-dim text-center py-6 border border-dashed border-border rounded-lg">
          No extra sync targets configured
        </div>
      ) : (
        <div className="space-y-2">
          {targets.map((t, i) => (
            <TargetCard
              key={i}
              target={t}
              onChange={(updated) => update(i, updated)}
              onRemove={() => remove(i)}
            />
          ))}
        </div>
      )}

      <div className="pt-2 border-t border-border">
        <button
          onClick={handleSave}
          disabled={saving}
          className="btn-primary text-sm"
        >
          {saving ? "Saving..." : saved ? "Saved!" : "Save Sync Targets"}
        </button>
      </div>
    </div>
  );
}
