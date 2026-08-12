import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";

import {
  executeCloneRemote,
  previewCloneRemote,
  runningInTauri,
} from "@/lib/api";
import { errorMessage, isPreviewRequiredError } from "@/lib/errors";
import type { CloneFormValues, CloneRequestDto, OperationPreviewDto, RepoInfo } from "@/types";

export function useClone(
  onSuccess: (info: RepoInfo, warning: string | null) => Promise<void>,
) {
  const [cloneOpen, setCloneOpen] = useState(false);
  const [preview, setPreview] = useState<OperationPreviewDto | null>(null);
  const [pending, setPending] = useState<CloneRequestDto | null>(null);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const confirmingRef = useRef(false);

  useEffect(() => {
    if (!runningInTauri()) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<{ line: string }>("clone-progress", (event) => {
      if (!disposed) setProgress(event.payload.line);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const openClone = useCallback(() => {
    setError(null);
    setCloneOpen(true);
  }, []);

  const cancelCloneDialog = useCallback(() => {
    if (loading) return;
    setCloneOpen(false);
    setError(null);
  }, [loading]);

  const requestClone = useCallback(
    async (values: CloneFormValues) => {
      setLoading(true);
      setError(null);
      const request: CloneRequestDto = {
        url: values.url,
        parentDir: values.parentDir,
        folderName: values.folderName,
        branch: values.branch,
        depth: values.depth,
      };
      try {
        const p = await previewCloneRemote(request);
        setPreview(p);
        setPending(request);
        if (p.blocked) {
          setError(p.blocked);
        } else {
          setCloneOpen(false);
        }
      } catch (e) {
        setPreview(null);
        setPending(null);
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const cancelPreview = useCallback(() => {
    if (loading) return;
    setPreview(null);
    setPending(null);
    setProgress(null);
    setError(null);
  }, [loading]);

  const confirmClone = useCallback(async () => {
    if (confirmingRef.current) return;
    const auth = preview?.authorization?.trim();
    if (!pending || preview?.blocked || !auth) {
      if (!auth && pending && !preview?.blocked) {
        setError("Confirmação inválida: falta autorização do preview.");
      }
      return;
    }
    confirmingRef.current = true;
    setLoading(true);
    setError(null);
    setProgress("Iniciando clone…");
    try {
      const result = await executeCloneRemote(auth);
      setPreview(null);
      setPending(null);
      setProgress(null);
      await onSuccess(result.repo, result.warning ?? null);
    } catch (e) {
      const msg = errorMessage(e);
      if (pending && isPreviewRequiredError(e)) {
        try {
          const p = await previewCloneRemote(pending);
          setPreview(p);
          setError(`${msg} Atualizei o preview — clique em Confirmar de novo.`);
        } catch (previewErr) {
          setError(
            previewErr instanceof Error ? previewErr.message : String(previewErr),
          );
        }
      } else {
        setError(msg);
      }
    } finally {
      setLoading(false);
      confirmingRef.current = false;
    }
  }, [pending, preview?.blocked, preview?.authorization, onSuccess]);

  return {
    cloneOpen,
    openClone,
    cancelCloneDialog,
    requestClone,
    preview,
    pending,
    loading,
    progress,
    error,
    cancelPreview,
    confirmClone,
  };
}
