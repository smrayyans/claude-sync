import { useState } from "react";
import RemoteConfig from "./RemoteConfig";
import MachineSettings from "./MachineSettings";

const tabs = ["Remote", "Machine"] as const;
type Tab = (typeof tabs)[number];

export default function Settings() {
  const [activeTab, setActiveTab] = useState<Tab>("Remote");

  return (
    <div className="p-6 max-w-2xl">
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-text">Settings</h1>
        <p className="text-sm text-text-muted mt-0.5">Configure sync and machine settings</p>
      </div>

      <div className="flex gap-1 mb-6 border-b border-border">
        {tabs.map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 text-sm transition-colors border-b-2 -mb-px ${
              activeTab === tab
                ? "border-accent text-accent"
                : "border-transparent text-text-muted hover:text-text"
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {activeTab === "Remote" && <RemoteConfig />}
      {activeTab === "Machine" && <MachineSettings />}
    </div>
  );
}
