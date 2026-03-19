import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useSyncStore } from "../stores/syncStore";

export function useSync() {
  const { setIsSyncing, refreshStatus, refreshPending } = useSyncStore();

  useEffect(() => {
    const unlistens: Promise<() => void>[] = [];

    unlistens.push(
      listen("sync-started", () => {
        setIsSyncing(true);
      })
    );

    unlistens.push(
      listen("sync-completed", () => {
        setIsSyncing(false);
        refreshStatus();
        refreshPending();
      })
    );

    unlistens.push(
      listen("sync-error", () => {
        setIsSyncing(false);
        refreshStatus();
      })
    );

    unlistens.push(
      listen("trigger-sync", () => {
        // Triggered from tray
        const { syncNow } = useSyncStore.getState();
        syncNow().catch(console.error);
      })
    );

    return () => {
      unlistens.forEach((p) => p.then((f) => f()));
    };
  }, []);
}
