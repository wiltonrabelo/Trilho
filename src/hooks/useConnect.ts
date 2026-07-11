import { useCallback, useEffect, useRef, useState } from "react";

import {
  configureGcmHelper,
  enableGithubUseHttpPath,
  getSshPublicKey,
  logoutGithubAccount,
  storeGithubPat,
  testGithubSsh,
  triggerGithubLogin,
} from "@/lib/api";
import type { SshTestResultDto } from "@/types";

export function useConnect(
  activeRemoteUrl?: string | null,
  onSuccess?: () => void,
) {
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sshTest, setSshTest] = useState<SshTestResultDto | null>(null);
  const [copyHint, setCopyHint] = useState<string | null>(null);
  const lastRemoteRef = useRef<string | null>(null);

  useEffect(() => {
    const remote = activeRemoteUrl ?? null;
    if (lastRemoteRef.current !== remote) {
      setSshTest(null);
      lastRemoteRef.current = remote;
    }
  }, [activeRemoteUrl]);

  const cancel = useCallback(() => {
    if (loading) return;
    setOpen(false);
    setError(null);
    setSshTest(null);
    setCopyHint(null);
  }, [loading]);

  const openDialog = useCallback(() => {
    setError(null);
    setCopyHint(null);
    setOpen(true);
  }, []);
  async function loginGcm(remoteUrl?: string | null) {
    setLoading(true);
    setError(null);
    try {
      await triggerGithubLogin(remoteUrl);
      onSuccess?.();
      setOpen(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function savePat(pat: string) {
    setLoading(true);
    setError(null);
    try {
      await storeGithubPat(pat, activeRemoteUrl);
      setCopyHint("Token salvo — acesso ao repositório confirmado.");
      onSuccess?.();
      await new Promise((r) => window.setTimeout(r, 900));
      setOpen(false);
      setCopyHint(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function configureGcm() {
    setLoading(true);
    setError(null);
    try {
      await configureGcmHelper();
      onSuccess?.();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function testSsh() {
    setLoading(true);
    setError(null);
    setSshTest(null);
    try {
      const result = await testGithubSsh();
      setSshTest(result);
      if (result.success) {
        onSuccess?.();
      }
    } catch (e) {
      const message = String(e);
      setSshTest({ success: false, username: null, message });
      setError(message);
    } finally {
      setLoading(false);
    }
  }

  async function copyPublicKey(name: string) {
    setCopyHint(null);
    setError(null);
    try {
      const text = await getSshPublicKey(name);
      await navigator.clipboard.writeText(text);
      setCopyHint(`Chave pública «${name}» copiada — cole em github.com/settings/keys`);
    } catch (e) {
      setError(String(e));
    }
  }

  async function logoutAccount(username: string) {
    setLoading(true);
    setError(null);
    try {
      await logoutGithubAccount(username);
      onSuccess?.();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function enableUseHttpPath() {
    setLoading(true);
    setError(null);
    try {
      await enableGithubUseHttpPath();
      onSuccess?.();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return {
    open,
    loading,
    error,
    sshTest,
    copyHint,
    openDialog,
    cancel,
    loginGcm,
    savePat,
    configureGcm,
    testSsh,
    copyPublicKey,
    logoutAccount,
    enableUseHttpPath,
  };
}
