import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface AgentFrontmatter {
  name?: string;
  description?: string;
  tools?: string[];
  model?: string;
  color?: string;
}

export interface Agent {
  name: string;
  description: string;
  content: string;
  path: string;
  frontmatter: AgentFrontmatter;
}

export interface Template {
  name: string;
  description: string;
  content: string;
}

interface AgentStore {
  agents: Agent[];
  templates: Template[];
  selectedAgent: Agent | null;
  loading: boolean;
  loadAgents: () => Promise<void>;
  loadTemplates: () => Promise<void>;
  selectAgent: (agent: Agent | null) => void;
  saveAgent: (agent: Agent) => Promise<void>;
  deleteAgent: (name: string) => Promise<void>;
  createFromTemplate: (templateName: string) => Promise<Agent>;
}

export const useAgentStore = create<AgentStore>((set, get) => ({
  agents: [],
  templates: [],
  selectedAgent: null,
  loading: false,

  loadAgents: async () => {
    set({ loading: true });
    const agents = await invoke<Agent[]>("list_agents");
    set({ agents, loading: false });
  },

  loadTemplates: async () => {
    const templates = await invoke<Template[]>("list_agent_templates");
    set({ templates });
  },

  selectAgent: (selectedAgent) => set({ selectedAgent }),

  saveAgent: async (agent) => {
    await invoke("save_agent", { agent });
    await get().loadAgents();
  },

  deleteAgent: async (name) => {
    await invoke("delete_agent", { name });
    set((s) => ({
      agents: s.agents.filter((a) => a.name !== name),
      selectedAgent: s.selectedAgent?.name === name ? null : s.selectedAgent,
    }));
  },

  createFromTemplate: async (templateName) => {
    const agent = await invoke<Agent>("create_agent_from_template", { templateName });
    await get().loadAgents();
    return agent;
  },
}));
