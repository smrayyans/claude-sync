import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Bot,
  Brain,
  GitCommit,
  Settings,
  RefreshCw,
  Wifi,
  WifiOff,
} from "lucide-react";
import { cn } from "../../lib/utils";
import { useSyncStore } from "../../stores/syncStore";
import { useSettingsStore } from "../../stores/settingsStore";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/agents", icon: Bot, label: "Agents" },
  { to: "/memory", icon: Brain, label: "Memory" },
  { to: "/history", icon: GitCommit, label: "History" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export default function Sidebar() {
  const { status, isSyncing } = useSyncStore();
  const { machineConfig } = useSettingsStore();

  return (
    <aside className="w-56 bg-surface border-r border-border flex flex-col h-full">
      {/* Logo */}
      <div className="p-4 border-b border-border">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded-md bg-accent flex items-center justify-center">
            <RefreshCw size={14} className="text-white" />
          </div>
          <span className="font-semibold text-text text-sm">claude-sync</span>
        </div>
        <div className="mt-2 text-xs text-text-dim truncate">
          {machineConfig?.machineName ?? "Loading..."}
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 p-2">
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
