import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface MemoryFrontmatter {
  name?: string;
  description?: string;
  type?: string;
}

export interface MemoryFile {
  path: string;
  name: string;
  content: string;
  project_slug: string;
  frontmatter: MemoryFrontmatter;
}

interface MemoryStore {
  files: MemoryFile[];
  selectedFile: MemoryFile | null;
  loading: boolean;
  loadFiles: () => Promise<void>;
  selectFile: (file: MemoryFile | null) => void;
  saveFile: (path: string, content: string) => Promise<void>;
  deleteFile: (path: string) => Promise<void>;
  getProjectFiles: (slug: string) => MemoryFile[];
}

export const useMemoryStore = create<MemoryStore>((set, get) => ({
  files: [],
  selectedFile: null,
  loading: false,

  loadFiles: async () => {
    set({ loading: true });
    const files = await invoke<MemoryFile[]>("list_memory_files");
    set({ files, loading: false });
  },

  selectFile: (selectedFile) => set({ selectedFile }),

  saveFile: async (path, content) => {
    await invoke("save_memory_file", { path, content });
    await get().loadFiles();
    // Refresh selected if it was this file
    const updated = get().files.find((f) => f.path === path);
    if (updated) set({ selectedFile: updated });
  },

  deleteFile: async (path) => {
    await invoke("delete_memory_file", { path });
    set((s) => ({
      files: s.files.filter((f) => f.path !== path),
      selectedFile: s.selectedFile?.path === path ? null : s.selectedFile,
    }));
  },

  getProjectFiles: (slug) => {
    return get().files.filter((f) => f.project_slug === slug);
  },
}));
