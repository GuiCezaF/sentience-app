import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { INITIAL_SNAPSHOT, type TraySnapshot } from "../types/agent";

export function useAgentSnapshot() {
  const [snapshot, setSnapshot] = useState<TraySnapshot>(INITIAL_SNAPSHOT);

  const refresh = useCallback(async () => {
    try {
      const next = await invoke<TraySnapshot>("snapshot");
      setSnapshot(next);
    } catch {
      // Fora do Tauri (vite isolado) o Extra permanece no snapshot inicial.
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { snapshot, refresh };
}
