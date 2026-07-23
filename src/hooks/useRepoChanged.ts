import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { runningInTauri } from "@/lib/api";

const DEBOUNCE_MS = 800;

/** Escuta eventos do watcher (RF-19) e dispara refresh (com debounce). */
export function useRepoChanged(onChange: () => void) {
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;

  useEffect(() => {
    if (!runningInTauri()) return;

    let timer: number | undefined;
    let unlisten: (() => void) | undefined;

    const schedule = () => {
      window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        void onChangeRef.current();
      }, DEBOUNCE_MS);
    };

    void listen("repo-changed", schedule).then((fn) => {
      unlisten = fn;
    });

    return () => {
      window.clearTimeout(timer);
      unlisten?.();
    };
  }, []);
}
