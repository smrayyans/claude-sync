import { useEffect, useState } from "react";
import { Brain, FolderOpen, FileText, Trash2 } from "lucide-react";
import { useMemoryStore } from "../../stores/memoryStore";
import MemoryEditor from "./MemoryEditor";

export default function MemoryBrowser() {
  const { files, selectedFile, loading, loadFiles, selectFile, deleteFile } =
    useMemoryStore();

  useEffect(() => {
    loadFiles();
  }, []);

  if (selectedFile) {
    return <MemoryEditor onBack={() => selectFile(null)} />;
  }

  // Group by project slug
  const groups: Record<string, typeof files> = {};
  for (const f of files) {
    if (!groups[f.project_slug]) groups[f.project_slug] = [];
    groups[f.project_slug].push(f);
  }

  const handleDelete = async (path: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm("Delete this memory file?")) {
      await deleteFile(path);
    }
  };

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-text">Memory</h1>
        <p className="text-sm text-text-muted mt-0.5">
          Browse and edit Claude's memory files
        </p>
      </div>

      {loading ? (
        <div className="text-sm text-text-muted">Loading memory files...</div>
      ) : Object.keys(groups).length === 0 ? (
        <div className="text-center py-16">
          <Brain size={40} className="text-text-dim mx-auto mb-3" />
          <p className="text-text-muted text-sm">No memory files found</p>
          <p className="text-text-dim text-xs mt-1">
            Memory files appear when Claude stores project-specific context
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {Object.entries(groups).map(([slug, projectFiles]) => (
            <div key={slug} className="card">
              <div className="flex items-center gap-2 mb-3">
                <FolderOpen size={14} className="text-accent" />
                <span className="text-sm font-medium text-text">{slug}</span>
                <span className="text-xs text-text-dim">
                  ({projectFiles.length} files)
                </span>
              </div>
              <div className="space-y-1">
                {projectFiles.map((file) => (
                  <div
                    key={file.path}
                    className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-surface-2 cursor-pointer group"
                    onClick={() => selectFile(file)}
                  >
                    <FileText size={12} className="text-text-dim flex-shrink-0" />
                    <div className="flex-1 min-w-0">
                      <span className="text-sm text-text">{file.name}</span>
                      {file.frontmatter.description && (
                        <span className="text-xs text-text-dim ml-2 truncate">
                          — {file.frontmatter.description}
                        </span>
                      )}
                    </div>
                    {file.frontmatter.type && (
                      <span className="text-xs bg-surface-3 text-text-dim px-1.5 rounded">
                        {file.frontmatter.type}
                      </span>
                    )}
                    <button
                      onClick={(e) => handleDelete(file.path, e)}
                      className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-error/10 hover:text-error transition-all"
                    >
                      <Trash2 size={11} />
                    </button>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
