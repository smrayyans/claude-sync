import { useEffect, useState } from "react";
import { Plus, Bot, Trash2, FileText } from "lucide-react";
import { useAgentStore } from "../../stores/agentStore";
import AgentEditor from "./AgentEditor";
import AgentTemplates from "./AgentTemplates";

export default function AgentList() {
  const { agents, selectedAgent, loading, loadAgents, selectAgent, deleteAgent } =
    useAgentStore();
  const [showTemplates, setShowTemplates] = useState(false);

  useEffect(() => {
    loadAgents();
  }, []);

  const handleDelete = async (name: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm(`Delete agent "${name}"?`)) {
      await deleteAgent(name);
    }
  };

  if (selectedAgent) {
    return <AgentEditor onBack={() => selectAgent(null)} />;
  }

  if (showTemplates) {
    return <AgentTemplates onBack={() => setShowTemplates(false)} />;
  }

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-xl font-semibold text-text">Agents</h1>
          <p className="text-sm text-text-muted mt-0.5">
            Manage your Claude Code agents
          </p>
        </div>
        <div className="flex gap-2">
          <button
            className="btn-secondary flex items-center gap-2 text-sm"
            onClick={() => setShowTemplates(true)}
          >
            <FileText size={14} />
            Templates
          </button>
          <button
            className="btn-primary flex items-center gap-2 text-sm"
            onClick={() => {
              selectAgent({
                name: "new-agent",
                description: "",
                content: "---\nname: New Agent\ndescription: Description here\n---\n\nAgent instructions here.",
                path: "",
                frontmatter: {},
              });
            }}
          >
            <Plus size={14} />
            New Agent
          </button>
        </div>
      </div>

      {loading ? (
        <div className="text-sm text-text-muted">Loading agents...</div>
      ) : agents.length === 0 ? (
        <div className="text-center py-16">
          <Bot size={40} className="text-text-dim mx-auto mb-3" />
          <p className="text-text-muted text-sm">No agents yet</p>
          <p className="text-text-dim text-xs mt-1">
            Create an agent or start from a template
          </p>
          <button
            className="btn-primary mt-4 text-sm"
            onClick={() => setShowTemplates(true)}
          >
            Browse Templates
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-2 gap-3">
          {agents.map((agent) => (
            <div
              key={agent.name}
              className="card cursor-pointer hover:border-accent/50 transition-colors group"
              onClick={() => selectAgent(agent)}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-2 min-w-0">
                  <div className="w-8 h-8 rounded-md bg-accent/10 flex items-center justify-center flex-shrink-0">
                    <Bot size={14} className="text-accent" />
                  </div>
                  <div className="min-w-0">
                    <div className="text-sm font-medium text-text truncate">
                      {agent.name}
                    </div>
                    <div className="text-xs text-text-muted truncate mt-0.5">
                      {agent.description}
                    </div>
                  </div>
                </div>
                <button
                  onClick={(e) => handleDelete(agent.name, e)}
                  className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-error/10 hover:text-error transition-all"
                >
                  <Trash2 size={12} />
                </button>
              </div>
              {agent.frontmatter.tools && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {agent.frontmatter.tools.slice(0, 4).map((t) => (
                    <span key={t} className="text-xs bg-surface-2 text-text-dim px-1.5 py-0.5 rounded">
                      {t}
                    </span>
                  ))}
                  {agent.frontmatter.tools.length > 4 && (
                    <span className="text-xs text-text-dim">
                      +{agent.frontmatter.tools.length - 4}
                    </span>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
