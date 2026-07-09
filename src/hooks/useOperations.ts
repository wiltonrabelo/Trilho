import { useCallback, useState } from "react";

import {
  executePublishOperation,
  executeWriteOperation,
  previewPublishOperation,
  previewWriteOperation,
} from "@/lib/api";
import type { OperationPreviewDto, WriteRequestDto } from "@/types";

export function useOperations(onSuccess: () => Promise<void>) {
  const [preview, setPreview] = useState<OperationPreviewDto | null>(null);
  const [pending, setPending] = useState<WriteRequestDto | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);

  const clearInfo = useCallback(() => setInfo(null), []);

  const request = useCallback(async (req: WriteRequestDto) => {
    setLoading(true);
    setError(null);
    try {
      const p = await previewWriteOperation(req);
      setPreview(p);
      setPending(req);
    } catch (e) {
      setPreview(null);
      setPending(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const requestPublish = useCallback(
    async (remoteUrl?: string): Promise<OperationPreviewDto | null> => {
      setLoading(true);
      setError(null);
      const url = remoteUrl?.trim() || null;
      try {
        const p = await previewPublishOperation(url);
        setPreview(p);
        setPending({ kind: "publish", url });
        if (p.blocked) {
          setError(p.blocked);
        }
        return p;
      } catch (e) {
        setPreview(null);
        setPending(null);
        setError(e instanceof Error ? e.message : String(e));
        return null;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const cancel = useCallback(() => {
    setPreview(null);
    setPending(null);
    setError(null);
    setInfo(null);
  }, []);

  const confirm = useCallback(async (): Promise<boolean> => {
    if (!pending || preview?.blocked) return false;
    setLoading(true);
    setError(null);
    try {
      if (pending.kind === "publish") {
        await executePublishOperation(pending.url);
      } else {
        await executeWriteOperation(pending);
      }
      setPreview(null);
      setPending(null);
      await onSuccess();
      return true;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return false;
    } finally {
      setLoading(false);
    }
  }, [pending, preview?.blocked, onSuccess]);

  /** Executa sem diálogo de preview — para ações já confirmadas na UI (ex.: resolver conflito). */
  const executeDirect = useCallback(
    async (
      req: WriteRequestDto,
      options?: { afterSuccess?: () => Promise<void> },
    ) => {
      setLoading(true);
      setError(null);
      try {
        await executeWriteOperation(req);
        if (options?.afterSuccess) {
          await options.afterSuccess();
        } else {
          await onSuccess();
        }
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
      }
    },
    [onSuccess],
  );

  return {
    preview,
    pending,
    loading,
    error,
    info,
    request,
    requestPublish,
    confirm,
    cancel,
    executeDirect,
    setInfo,
    clearInfo,
    setError,
  };
}
