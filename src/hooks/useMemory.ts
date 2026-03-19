import { useEffect } from "react";
import { useMemoryStore } from "../stores/memoryStore";

export function useMemory(projectSlug?: string) {
  const { files, loading, loadFiles, getProjectFiles } = useMemoryStore();

  useEffect(() => {
    if (files.length === 0) {
      loadFiles();
    }
  }, []);

  const projectFiles = projectSlug ? getProjectFiles(projectSlug) : files;

  // Get unique project slugs
  const projects = [...new Set(files.map((f) => f.project_slug))];

  return { files: projectFiles, allFiles: files, projects, loading };
}
