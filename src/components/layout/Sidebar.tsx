import { useEffect, useState } from "react";
import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Bot,
  Brain,
  MessageSquare,
  GitCommit,
  Settings,
  RefreshCw,
  Wifi,
  WifiOff,
  ArrowUpCircle,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { cn } from "../../lib/utils";
import { useSyncStore } from "../../stores/syncStore";
import { useSettingsStore } from "../../stores/settingsStore";

interface UpdateInfo {
  current_version: string;
  latest_version: string;
  update_available: boolean;
  release_url: string;
  release_notes: string;
}

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/agents", icon: Bot, label: "Agents" },
  { to: "/memory", icon: Brain, label: "Memory" },
  { to: "/chats", icon: MessageSquare, label: "Chats" },
  { to: "/history", icon: GitCommit, label: "Sync Log" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export default function Sidebar() {
  const { status, isSyncing } = useSyncStore();
  const { machineConfig } = useSettingsStore();
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);

  useEffect(() => {
    // Check for updates once on startup, silently
    invoke<UpdateInfo>("check_for_updates")
      .then((info) => {
        if (info.update_available) setUpdateInfo(info);
      })
      .catch(() => {}); // silently ignore if offline or API fails
  }, []);

  const openRelease = () => {
    if (updateInfo?.release_url) {
      open(updateInfo.release_url).catch(() => {});
    }
  };

  return (
    <aside className="w-56 bg-surface border-r border-border flex flex-col h-full">
      {/* Logo */}
      <div className="p-4 border-b border-border">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded-md bg-accent flex items-center justify-center">
            <RefreshCw size={14} className="text-white" />
          </div>
          <span className="font-semibold text-text text-sm">claude-sync</span>
          <span className="text-[10px] text-text-dim ml-auto">
            v{updateInfo?.current_version ?? "0.1.0"}
          </span>
        </div>
        <div className="mt-2 text-xs text-text-dim truncate">
          {machineConfig?.machineName ?? "Loading..."}
        </div>
      </div>

      {/* Update banner */}
      {updateInfo?.update_available && (
        <button
          onClick={openRelease}
          className="mx-2 mt-2 flex items-center gap-2 px-3 py-2 rounded-md bg-accent/10 border border-accent/30 text-xs text-accent hover:bg-accent/20 transition-colors text-left"
        >
          <ArrowUpCircle size={13} className="shrink-0" />
          <span>
            v{updateInfo.latest_version} available
          </span>
        </button>
      )}

      {/* Nav */}
      <nav className="flex-1 p-2 mt-1">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors mb-0.5",
                isActive
                  ? "bg-accent/15 text-accent"
                  : "text-text-muted hover:bg-surface-2 hover:text-text"
              )
            }
          >
            <Icon size={16} />
            {label}
          </NavLink>
        ))}
      </nav>

      {/* Status footer */}
      <div className="p-3 border-t border-border">
        <div className="flex items-center gap-2 text-xs">
          {isSyncing ? (
            <>
              <RefreshCw size={12} className="text-accent animate-spin" />
              <span className="text-accent">Syncing...</span>
            </>
          ) : status?.is_online ? (
            <>
              <Wifi size={12} className="text-success" />
              <span className="text-success">Online</span>
            </>
          ) : (
            <>
              <WifiOff size={12} className="text-text-dim" />
              <span className="text-text-dim">Offline</span>
            </>
          )}
          {status?.pending_changes !== undefined && status.pending_changes > 0 && (
            <span className="ml-auto badge-yellow">
              {status.pending_changes}
            </span>
          )}
        </div>
      </div>
    </aside>
  );
}
