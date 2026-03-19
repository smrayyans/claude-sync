import { useState } from "react";
import { Plus, X } from "lucide-react";
import { useSettingsStore } from "../../stores/settingsStore";

export default function MachineSettings() {
  const { machineConfig, saveMachineConfig } = useSettingsStore();
  const [name, setName] = useState(machineConfig?.machineName ?? "");
  const [interval, setInterval] = useState(machineConfig?.autoSyncInterval ?? 15);
  const [overrides, setOverrides] = useState<string[]>(
    machineConfig?.machineOverrides ?? []
  );
  const [newOverride, setNewOverride] = useState("");
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const addOverride = () => {
    const trimmed = newOverride.trim();
    if (trimmed && !overrides.includes(trimmed)) {
      setOverrides([...overrides, trimmed]);
      setNewOverride("");
    }
  };

  const removeOverride = (key: string) => {
    setOverrides(overrides.filter((o) => o !== key));
  };

  const handleSave = async () => {
    if (!machineConfig) return;
    setSaving(true);
    try {
      await saveMachineConfig({
        ...machineConfig,
        machineName: name,
        autoSyncInterval: interval,
        machineOverrides: overrides,
      });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm text-text-muted mb-1.5">Machine Name</label>
        <input
          type="text"
          className="input"
          placeholder="My Laptop"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </div>

      <div>
        <label className="block text-sm text-text-muted mb-1.5">
          Auto-sync Interval (minutes)
        </label>
        <input
          type="number"
          className="input"
          min={1}
          max={1440}
          value={interval}
          onChange={(e) => setInterval(Number(e.target.value))}
        />
      </div>

      <div>
        <label className="block text-sm text-text-muted mb-1.5">
          Machine-local Overrides
        </label>
        <p className="text-xs text-text-dim mb-2">
          Keys that are never pushed/pulled — machine-specific preferences (e.g.{" "}
          <code className="text-accent">settings.theme</code>)
        </p>
        <div className="space-y-1.5 mb-2">
          {overrides.map((key) => (
            <div
              key={key}
              className="flex items-center gap-2 bg-surface-2 rounded px-2 py-1.5 text-sm"
            >
              <code className="flex-1 text-text font-mono text-xs">{key}</code>
              <button
                onClick={() => removeOverride(key)}
                className="text-text-dim hover:text-error transition-colors"
              >
                <X size={12} />
              </button>
            </div>
          ))}
        </div>
        <div className="flex gap-2">
          <input
            type="text"
            className="input text-sm"
            placeholder="settings.theme"
            value={newOverride}
            onChange={(e) => setNewOverride(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && addOverride()}
          />
          <button onClick={addOverride} className="btn-secondary text-sm px-3">
            <Plus size={14} />
          </button>
        </div>
      </div>

      {machineConfig?.machineId && (
        <div className="pt-2 border-t border-border">
          <div className="text-xs text-text-dim">
            Machine ID: <code className="font-mono">{machineConfig.machineId}</code>
          </div>
        </div>
      )}

      <button
        onClick={handleSave}
        disabled={saving}
        className="btn-primary text-sm"
      >
        {saving ? "Saving..." : saved ? "Saved!" : "Save Machine Settings"}
      </button>
    </div>
  );
}
