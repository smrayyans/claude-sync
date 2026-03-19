import { useEffect } from "react";
import { useAgentStore } from "../stores/agentStore";

export function useAgents() {
  const { agents, loading, loadAgents } = useAgentStore();

  useEffect(() => {
    if (agents.length === 0) {
      loadAgents();
    }
  }, []);

  return { agents, loading };
}
