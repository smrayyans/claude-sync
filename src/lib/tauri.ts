import { invoke } from "@tauri-apps/api/core";
import type { Agent, Template } from "../stores/agentStore";
import type { MemoryFile } from "../stores/memoryStore";
import type { AppSettings, MachineConfig } from "../stores/settingsStore";
import type { FileChange, SyncResult, SyncStatus } from "../stores/syncStore";

export interface Commit {
  hash: string;
  short_hash: string;
  message: string;
  author: string;
  timestamp: string;
  machine_name: string | null;
  files_changed: number;
}

export type Resolution =
  | { Mine: null }
  | { Theirs: null }
  | { Manual: string };

// Sync
export const api = {
  sync: {
    now: () => invoke<SyncResult>("sync_now"),
    pull: () => invoke<SyncResult>("sync_pull"),
    push: () => invoke<SyncResult>("sync_push"),
    status: () => invoke<SyncStatus>("get_sync_status"),
    pending: () => invoke<FileChange[]>("get_pending_changes"),
  },
  agents: {
    list: () => invoke<Agent[]>("list_agents"),
    get: (name: string) => invoke<Agent>("get_agent", { name }),
    save: (agent: Agent) => invoke<void>("save_agent", { agent }),
    delete: (name: string) => invoke<void>("delete_agent", { name }),
    templates: () => invoke<Template[]>("list_agent_templates"),
    createFromTemplate: (templateName: string) =>
      invoke<Agent>("create_agent_from_template", { templateName }),
  },
  memory: {
    list: () => invoke<MemoryFile[]>("list_memory_files"),
    get: (path: string) => invoke<MemoryFile>("get_memory_file", { path }),
    save: (path: string, content: string) =>
      invoke<void>("save_memory_file", { path, content }),
    delete: (path: string) => invoke<void>("delete_memory_file", { path }),
    project: (projectSlug: string) =>
      invoke<MemoryFile[]>("get_project_memories", { projectSlug }),
  },
  history: {
    commits: (limit: number) => invoke<Commit[]>("get_commit_history", { limit }),
    diff: (hash: string) => invoke<string>("get_commit_diff", { hash }),
    resolve: (file: string, resolution: Resolution) =>
      invoke<void>("resolve_conflict", { file, resolution }),
  },
  settings: {
    get: () => invoke<AppSettings>("get_app_settings"),
    save: (settings: AppSettings) =>
      invoke<void>("save_app_settings", { settings }),
    getMachine: () => invoke<MachineConfig>("get_machine_config"),
    saveMachine: (config: MachineConfig) =>
      invoke<void>("save_machine_config", { config }),
    setupRemote: (url: string, token: string) =>
      invoke<void>("setup_remote", { url, token }),
    testConnection: () => invoke<boolean>("test_remote_connection"),
  },
};
