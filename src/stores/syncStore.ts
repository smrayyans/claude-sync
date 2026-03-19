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

interface SyncStore {
  status: SyncStatus | null;
  pendingChanges: FileChange[];
  isSyncing: boolean;
  lastResult: SyncResult | null;
  setStatus: (status: SyncStatus) => void;
  setIsSyncing: (val: boolean) => void;
  refreshStatus: () => Promise<void>;
  refreshPending: () => Promise<void>;
  syncNow: () => Promise<SyncResult>;
}

export const useSyncStore = create<SyncStore>((set, get) => ({
  status: null,
  pendingChanges: [],
  isSyncing: false,
  lastResult: null,

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

  syncNow: async () => {
    set({ isSyncing: true });
    try {
      const result = await invoke<SyncResult>("sync_now");
      set({ lastResult: result, isSyncing: false });
      await get().refreshStatus();
      await get().refreshPending();
      return result;
    } catch (err) {
      set({ isSyncing: false });
      throw err;
    }
  },
}));
