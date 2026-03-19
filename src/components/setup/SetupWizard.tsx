import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { RefreshCw, CheckCircle, ArrowRight, ArrowLeft, Loader } from "lucide-react";
import { useSettingsStore } from "../../stores/settingsStore";

const STEPS = [
  { title: "Welcome", subtitle: "Set up claude-sync" },
  { title: "Name Machine", subtitle: "Identify this machine" },
  { title: "Connect Remote", subtitle: "Link your Git repository" },
  { title: "Initial Pull", subtitle: "Download existing data" },
  { title: "Done", subtitle: "You're all set!" },
];

export default function SetupWizard() {
  const navigate = useNavigate();
  const { machineConfig, saveMachineConfig, setupRemote, testConnection } =
    useSettingsStore();

  const [step, setStep] = useState(0);
  const [machineName, setMachineName] = useState(machineConfig?.machineName ?? "");
  const [repoUrl, setRepoUrl] = useState("");
  const [token, setToken] = useState("");
  const [testing, setTesting] = useState(false);
  const [testOk, setTestOk] = useState<boolean | null>(null);
  const [saving, setSaving] = useState(false);
  const [pulling, setPulling] = useState(false);

  const handleNext = async () => {
    if (step === 1 && machineName) {
      if (machineConfig) {
        await saveMachineConfig({ ...machineConfig, machineName });
      }
    }
    if (step === 2 && repoUrl && token) {
      setSaving(true);
      try {
        await setupRemote(repoUrl, token);
      } finally {
        setSaving(false);
      }
    }
    if (step === 3) {
      // Trigger initial pull via sync
      setPulling(true);
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("sync_pull");
      } catch {
        // First-time pull may fail if remote is empty
      } finally {
        setPulling(false);
      }
    }
    if (step === 4) {
      navigate("/");
      return;
    }
    setStep((s) => s + 1);
  };

  const handleTest = async () => {
    setTesting(true);
    setTestOk(null);
    try {
      const ok = await testConnection();
      setTestOk(ok);
    } catch {
      setTestOk(false);
    } finally {
      setTesting(false);
    }
  };

  const canNext = () => {
    if (step === 1) return machineName.trim().length > 0;
    if (step === 2) return repoUrl.trim().length > 0 && token.trim().length > 0;
    return true;
  };

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-6">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="flex items-center justify-center gap-2 mb-8">
          <div className="w-10 h-10 rounded-xl bg-accent flex items-center justify-center">
            <RefreshCw size={20} className="text-white" />
          </div>
          <span className="text-xl font-bold text-text">claude-sync</span>
        </div>

        {/* Step indicators */}
        <div className="flex justify-center gap-1.5 mb-8">
          {STEPS.map((_, i) => (
            <div
              key={i}
              className={`h-1.5 rounded-full transition-all ${
                i === step
                  ? "w-6 bg-accent"
                  : i < step
                  ? "w-3 bg-accent/50"
                  : "w-3 bg-surface-3"
              }`}
            />
          ))}
        </div>

        <div className="card">
          <div className="mb-6">
            <h2 className="text-lg font-semibold text-text">{STEPS[step].title}</h2>
            <p className="text-sm text-text-muted">{STEPS[step].subtitle}</p>
          </div>

          {/* Step 0: Welcome */}
          {step === 0 && (
            <div className="space-y-3 text-sm text-text-muted">
              <p>
                claude-sync keeps your Claude Code agents, memory, and settings in sync
                across all your machines using a private Git repository.
              </p>
              <ul className="space-y-2">
                {[
                  "Agents synced across machines",
                  "Skills (slash commands) synced",
                  "Project memory preserved",
                  "Settings with machine-local overrides",
                  "Full conflict resolution UI",
                ].map((item) => (
                  <li key={item} className="flex items-center gap-2">
                    <CheckCircle size={13} className="text-success flex-shrink-0" />
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Step 1: Machine name */}
          {step === 1 && (
            <div>
              <label className="block text-sm text-text-muted mb-1.5">
                What should we call this machine?
              </label>
              <input
                type="text"
                className="input"
                placeholder="Work Laptop, Home PC..."
                value={machineName}
                onChange={(e) => setMachineName(e.target.value)}
                autoFocus
              />
            </div>
          )}

          {/* Step 2: Remote */}
          {step === 2 && (
            <div className="space-y-3">
              <div className="bg-surface-2 rounded-md p-3 text-xs text-text-muted space-y-1.5">
                <p className="font-medium text-text">What repo does this need?</p>
                <p>A <span className="text-accent">new private GitHub repo</span> dedicated to storing your Claude data — <span className="text-warning">not</span> your code repos or dotfiles.</p>
                <p>Create one at <code className="text-accent">github.com/new</code>, name it something like <code className="text-accent">claude-data</code>, set it to <strong>Private</strong>.</p>
              </div>
              <div>
                <label className="block text-sm text-text-muted mb-1.5">
                  GitHub Repository URL
                </label>
                <input
                  type="url"
                  className="input"
                  placeholder="https://github.com/yourusername/claude-data"
                  value={repoUrl}
                  onChange={(e) => setRepoUrl(e.target.value)}
                />
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
                  Go to <code className="text-accent">github.com → Settings → Developer settings → Personal access tokens → Tokens (classic)</code> → generate with <code className="text-accent">repo</code> scope. Stored in OS keychain, never on disk.
                </p>
              </div>
              <button
                onClick={handleTest}
                disabled={!repoUrl || !token || testing}
                className="btn-secondary text-sm flex items-center gap-2"
              >
                {testing ? <Loader size={12} className="animate-spin" /> : null}
                Test Connection
              </button>
              {testOk === true && (
                <p className="text-xs text-success">Connection OK</p>
              )}
              {testOk === false && (
                <p className="text-xs text-error">Connection failed</p>
              )}
            </div>
          )}

          {/* Step 3: Pull */}
          {step === 3 && (
            <div className="text-sm text-text-muted space-y-2">
              <p>We'll do an initial pull from your remote to download any existing data.</p>
              <p className="text-text-dim text-xs">
                If the repo is empty, this will just initialize it.
              </p>
              {pulling && (
                <div className="flex items-center gap-2 text-accent">
                  <Loader size={13} className="animate-spin" />
                  Pulling...
                </div>
              )}
            </div>
          )}

          {/* Step 4: Done */}
          {step === 4 && (
            <div className="text-center space-y-2">
              <CheckCircle size={48} className="text-success mx-auto" />
              <p className="text-sm text-text-muted">
                claude-sync is configured and ready!
              </p>
              <p className="text-xs text-text-dim">
                Your Claude Code environment will now sync automatically.
              </p>
            </div>
          )}

          {/* Navigation */}
          <div className="flex justify-between mt-6 pt-4 border-t border-border">
            <button
              onClick={() => setStep((s) => Math.max(0, s - 1))}
              disabled={step === 0}
              className="btn-ghost text-sm flex items-center gap-1.5 disabled:opacity-30"
            >
              <ArrowLeft size={14} />
              Back
            </button>
            <button
              onClick={handleNext}
              disabled={!canNext() || saving || pulling}
              className="btn-primary text-sm flex items-center gap-1.5"
            >
              {saving || pulling ? (
                <Loader size={13} className="animate-spin" />
              ) : null}
              {step === 4 ? "Open App" : "Next"}
              {step < 4 && <ArrowRight size={14} />}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
