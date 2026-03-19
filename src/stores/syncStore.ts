import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface FileChange {
  path: string;
  change_type: "added" | "modified" | "deleted";
  size_bytes?: number;
}

export interface SyncStatus {
  pending_changes: number;
  last_synced: string | null;
  machine_name: string;
  is_online: boolean;
  is_syncing: boolean;
  error: string | null;
}

export interface SyncResult {
  success: boolean;
  files_pushed: string[];
  files_pulled: string[];
  conflicts: string[];
  message: string;
}

export interface PullLogEntry {
  machineName: string;
  machineId: string;
  timestamp: string;
}

export interface RepoStatus {
  local_changes: FileChange[];
  commits_behind: number;
  is_online: boolean;
  error: string | null;
}

interface SyncStore {
  status: SyncStatus | null;
  pendingChanges: FileChange[];
  repoStatus: RepoStatus | null;
  isRefreshing: boolean;
  isSyncing: boolean;
  lastResult: SyncResult | null;
  pullLog: PullLogEntry[];
  setStatus: (status: SyncStatus) => void;
  setIsSyncing: (val: boolean) => void;
  refreshStatus: () => Promise<void>;
  refreshPending: () => Promise<void>;
  refreshPullLog: () => Promise<void>;
  checkRepoStatus: () => Promise<void>;
  syncNow: () => Promise<SyncResult>;
  pullNow: () => Promise<SyncResult>;
  pushNow: () => Promise<SyncResult>;
}

async function invokeSync(command: string): Promise<SyncResult> {
  try {
    return await invoke<SyncResult>(command);
  } catch (err) {
    return {
      success: false,
      files_pushed: [],
      files_pulled: [],
      conflicts: [],
      message: String(err),
    };
  }
}

export const useSyncStore = create<SyncStore>((set, get) => ({
  status: null,
  pendingChanges: [],
  repoStatus: null,
  isRefreshing: false,
  isSyncing: false,
  lastResult: null,
  pullLog: [],

  setStatus: (status) => set({ status }),
  setIsSyncing: (isSyncing) => set({ isSyncing }),

  refreshStatus: async () => {
    const status = await invoke<SyncStatus>("get_sync_status");
    set({ status });
  },

  refreshPending: async () => {
    const pendingChanges = await invoke<FileChange[]>("get_pending_changes");
    set({ pendingChanges });
  },

  refreshPullLog: async () => {
    const pullLog = await invoke<PullLogEntry[]>("get_pull_log");
    set({ pullLog });
  },

  checkRepoStatus: async () => {
    set({ isRefreshing: true });
    try {
      const repoStatus = await invoke<RepoStatus>("check_repo_status");
      set({ repoStatus });
    } catch (err) {
      set({
        repoStatus: {
          local_changes: [],
          commits_behind: 0,
          is_online: false,
          error: String(err),
        },
      });
    } finally {
      set({ isRefreshing: false });
    }
  },

  syncNow: async () => {
    set({ isSyncing: true });
    const result = await invokeSync("sync_now");
    set({ lastResult: result, isSyncing: false });
    await get().refreshStatus();
    await get().refreshPending();
    return result;
  },

  pullNow: async () => {
    set({ isSyncing: true });
    const result = await invokeSync("sync_pull");
    set({ lastResult: result, isSyncing: false });
    await get().refreshStatus();
    await get().refreshPending();
    await get().refreshPullLog();
    return result;
  },

  pushNow: async () => {
    set({ isSyncing: true });
    const result = await invokeSync("sync_push");
    set({ lastResult: result, isSyncing: false });
    await get().refreshStatus();
    await get().refreshPending();
    return result;
  },
}));
