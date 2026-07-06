import { listen } from "@tauri-apps/api/event";

import { useEffect } from "react";

import { runningInTauri } from "@/lib/api";



/** Escuta eventos do watcher (RF-19) e dispara refresh. */

export function useRepoChanged(onChange: () => void) {

  useEffect(() => {

    if (!runningInTauri()) return;

    let unlisten: (() => void) | undefined;

    listen("repo-changed", () => onChange()).then((fn) => {

      unlisten = fn;

    });

    return () => {

      unlisten?.();

    };

  }, [onChange]);

}


