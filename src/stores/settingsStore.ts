import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface MachineConfig {
  machineId: string;
  machineName: string;
  remoteUrl?: string;
  autoSyncInterval: number;
  machineOverrides: string[];
  lastSynced?: string;
}

export interface AppSettings {
  claude: Record<string, unknown>;
  machine: MachineConfig;
}

interface SettingsStore {
  settings: AppSettings | null;
  machineConfig: MachineConfig | null;
  loading: boolean;
  loadSettings: () => Promise<void>;
  saveSettings: (settings: AppSettings) => Promise<void>;
  saveMachineConfig: (config: MachineConfig) => Promise<void>;
  setupRemote: (url: string, token: string) => Promise<void>;
  testConnection: () => Promise<boolean>;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: null,
  machineConfig: null,
  loading: false,

  loadSettings: async () => {
    set({ loading: true });
    try {
      const settings = await invoke<AppSettings>("get_app_settings");
      set({ settings, machineConfig: settings.machine, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  saveSettings: async (settings) => {
    await invoke("save_app_settings", { settings });
    set({ settings, machineConfig: settings.machine });
  },

  saveMachineConfig: async (config) => {
    await invoke("save_machine_config", { config });
    set((s) => ({
      machineConfig: config,
      settings: s.settings ? { ...s.settings, machine: config } : null,
    }));
  },

  setupRemote: async (url, token) => {
    await invoke("setup_remote", { url, token });
    await get().loadSettings();
  },

  testConnection: async () => {
    return invoke<boolean>("test_remote_connection");
  },
}));
