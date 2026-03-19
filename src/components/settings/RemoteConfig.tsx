import { useState } from "react";
import { CheckCircle, XCircle, Loader } from "lucide-react";
import { useSettingsStore } from "../../stores/settingsStore";

export default function RemoteConfig() {
  const { machineConfig, setupRemote, testConnection } = useSettingsStore();
  const [url, setUrl] = useState(machineConfig?.remoteUrl ?? "");
  const [token, setToken] = useState("");
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<boolean | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  const handleTest = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const ok = await testConnection();
      setTestResult(ok);
    } catch {
      setTestResult(false);
    } finally {
      setTesting(false);
    }
  };

  const handleSave = async () => {
    if (!url || !token) return;
    setSaving(true);
    try {
      await setupRemote(url, token);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm text-text-muted mb-1.5">GitHub Repository URL</label>
        <input
          type="url"
          className="input"
          placeholder="https://github.com/username/my-claude-data"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
        />
        <p className="text-xs text-text-dim mt-1">
          Private repo recommended. Only you should have access.
        </p>
      </div>

      <div>
        <label className="block text-sm text-text-muted mb-1.5">
          Personal Access Token (PAT)
        </label>
        <input
          type="password"
          className="input"
          placeholder="ghp_xxxxxxxxxxxx"
          value={token}
          onChange={(e) => setToken(e.target.value)}
        />
        <p className="text-xs text-text-dim mt-1">
          Requires <code className="text-accent">repo</code> scope. Stored securely in OS keychain.
        </p>
      </div>

      <div className="flex items-center gap-3">
        <button
          onClick={handleTest}
          disabled={!url || !token || testing}
          className="btn-secondary text-sm flex items-center gap-2"
        >
          {testing ? (
            <Loader size={13} className="animate-spin" />
          ) : testResult === true ? (
            <CheckCircle size={13} className="text-success" />
          ) : testResult === false ? (
            <XCircle size={13} className="text-error" />
          ) : null}
          Test Connection
        </button>

        {testResult === true && (
          <span className="text-xs text-success">Connection successful</span>
        )}
        {testResult === false && (
          <span className="text-xs text-error">Connection failed — check URL and token</span>
        )}
      </div>

      <div className="pt-2 border-t border-border">
        <button
          onClick={handleSave}
          disabled={!url || !token || saving}
          className="btn-primary text-sm"
        >
          {saving ? "Saving..." : saved ? "Saved!" : "Save Remote Configuration"}
        </button>
      </div>
    </div>
  );
}
