import { useCallback, useRef, useState } from "react";



import {

  executePublishOperation,

  executeWriteOperation,

  previewPublishOperation,

  previewWriteOperation,

} from "@/lib/api";

import { errorMessage, isPreviewRequiredError } from "@/lib/errors";

import type { OperationPreviewDto, WriteRequestDto } from "@/types";



export function useOperations(onSuccess: () => Promise<void>) {

  const [preview, setPreview] = useState<OperationPreviewDto | null>(null);

  const [pending, setPending] = useState<WriteRequestDto | null>(null);

  const [fromAssistant, setFromAssistant] = useState(false);

  const [loading, setLoading] = useState(false);

  const [error, setError] = useState<string | null>(null);

  const [info, setInfo] = useState<string | null>(null);

  const confirmingRef = useRef(false);



  const clearInfo = useCallback(() => setInfo(null), []);



  const request = useCallback(

    async (req: WriteRequestDto, options?: { fromAssistant?: boolean }) => {

      setLoading(true);

      setError(null);

      try {

        const p = await previewWriteOperation(req);

        setPreview(p);

        setPending(req);

        setFromAssistant(Boolean(options?.fromAssistant));

      } catch (e) {

        setPreview(null);

        setPending(null);

        setFromAssistant(false);

        setError(e instanceof Error ? e.message : String(e));

      } finally {

        setLoading(false);

      }

    },

    [],

  );



  const requestPublish = useCallback(

    async (remoteUrl?: string): Promise<OperationPreviewDto | null> => {

      setLoading(true);

      setError(null);

      const url = remoteUrl?.trim() || null;

      try {

        const p = await previewPublishOperation(url);

        setPreview(p);

        setPending({ kind: "publish", url });

        setFromAssistant(false);

        if (p.blocked) {

          setError(p.blocked);

        }

        return p;

      } catch (e) {

        setPreview(null);

        setPending(null);

        setFromAssistant(false);

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

    setFromAssistant(false);

    setError(null);

    setInfo(null);

  }, []);



  const confirm = useCallback(async (): Promise<boolean> => {

    if (confirmingRef.current) return false;

    const auth = preview?.authorization?.trim();

    if (!pending || preview?.blocked || !auth) {

      if (!auth && pending && !preview?.blocked) {

        setError("Confirmação inválida: falta autorização do preview.");

      }

      return false;

    }

    confirmingRef.current = true;

    setLoading(true);

    setError(null);

    try {

      if (pending.kind === "publish") {

        await executePublishOperation(auth);

      } else {

        await executeWriteOperation(auth, { fromAssistant });

      }

      setPreview(null);

      setPending(null);

      setFromAssistant(false);

      await onSuccess();

      return true;

    } catch (e) {

      const msg = errorMessage(e);

      // Token sumiu (restart / clique duplo antigo): renova o preview no diálogo.

      if (pending && isPreviewRequiredError(e)) {

        try {

          const p =

            pending.kind === "publish"

              ? await previewPublishOperation(pending.url)

              : await previewWriteOperation(pending);

          setPreview(p);

          setError(

            `${msg} Atualizei o preview — clique em Confirmar de novo.`,

          );

        } catch (previewErr) {

          setError(

            previewErr instanceof Error ? previewErr.message : String(previewErr),

          );

        }

      } else {

        setError(msg);

      }

      return false;

    } finally {

      confirmingRef.current = false;

      setLoading(false);

    }

  }, [pending, preview?.blocked, preview?.authorization, fromAssistant, onSuccess]);



  /**

   * Preview + execute com token A-02 — para ações já confirmadas na UI

   * (ex.: resolver conflito) sem reabrir o diálogo.

   */

  const executeDirect = useCallback(

    async (

      req: WriteRequestDto,

      options?: { afterSuccess?: () => Promise<void>; fromAssistant?: boolean },

    ) => {

      setLoading(true);

      setError(null);

      try {

        const p = await previewWriteOperation(req);

        if (p.blocked) {

          throw new Error(p.blocked);

        }

        const auth = p.authorization?.trim();

        if (!auth) {

          throw new Error("Confirmação inválida: falta autorização do preview.");

        }

        await executeWriteOperation(auth, {

          fromAssistant: options?.fromAssistant,

        });

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

    fromAssistant,

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


