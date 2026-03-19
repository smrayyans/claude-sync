import { useEffect } from "react";
import { ArrowLeft, FileText, Plus } from "lucide-react";
import { useAgentStore } from "../../stores/agentStore";

interface Props {
  onBack: () => void;
}

export default function AgentTemplates({ onBack }: Props) {
  const { templates, loadTemplates, createFromTemplate, selectAgent } = useAgentStore();

  useEffect(() => {
    loadTemplates();
  }, []);

  const handleCreate = async (templateName: string) => {
    const agent = await createFromTemplate(templateName);
    selectAgent(agent);
  };

  return (
    <div className="p-6">
      <div className="flex items-center gap-3 mb-6">
        <button onClick={onBack} className="btn-ghost p-1.5">
          <ArrowLeft size={16} />
        </button>
        <div>
          <h1 className="text-xl font-semibold text-text">Agent Templates</h1>
          <p className="text-sm text-text-muted mt-0.5">Start from a pre-built template</p>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-3">
        {templates.map((template) => (
          <div key={template.name} className="card hover:border-accent/50 transition-colors">
            <div className="flex items-start justify-between mb-2">
              <div className="flex items-center gap-2">
                <FileText size={14} className="text-accent mt-0.5" />
                <span className="text-sm font-medium text-text">{template.name}</span>
              </div>
            </div>
            <p className="text-xs text-text-muted mb-3">{template.description}</p>
            <button
              className="btn-primary text-xs flex items-center gap-1.5"
              onClick={() => handleCreate(template.name)}
            >
              <Plus size={12} />
              Use Template
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
