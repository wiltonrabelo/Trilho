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
      const url = remoteUrl?.trim() || undefined;
      try {
        const p = await previewPublishOperation(url ?? null);
        setPreview(p);
        setPending({ kind: "publish", url, remoteUrl: url });
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
  }, []);

  const confirm = useCallback(async () => {
    if (!pending || preview?.blocked) return;
    setLoading(true);
    setError(null);
    try {
      if (pending.kind === "publish") {
        const url = pending.url ?? pending.remoteUrl ?? null;
        await executePublishOperation(url);
      } else {
        await executeWriteOperation(pending);
      }
      setPreview(null);
      setPending(null);
      await onSuccess();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [pending, preview?.blocked, onSuccess]);

  return {
    preview,
    pending,
    loading,
    error,
    request,
    requestPublish,
    confirm,
    cancel,
  };
}
