import { describe, expect, it } from "vitest";

import { isAuthError, syncNotices, syncSummary } from "./syncNotices";
import type { CredentialStatusDto, SyncInfoDto } from "@/types";

const semCredencial: CredentialStatusDto = {
  helperConfigured: true,
  gcmAvailable: true,
  helperSummary: null,
  hint: null,
  githubConnected: false,
  githubUsername: null,
  sshKeys: [],
};

const base = {
  credential: semCredencial,
  hasRemote: true,
  isShallow: false,
  needsPublish: false,
  error: null,
};

describe("syncNotices", () => {
  it("fica vazio quando não há nada a avisar", () => {
    expect(syncNotices(base)).toEqual([]);
  });

  it("explica publicação conforme exista remoto", () => {
    const comRemoto = syncNotices({ ...base, needsPublish: true });
    expect(comRemoto[0].message).toContain("sem rastreamento remoto");

    const semRemoto = syncNotices({
      ...base,
      needsPublish: true,
      hasRemote: false,
    });
    expect(semRemoto[0].message).toContain("só local");
  });

  it("trata erro de autenticação como aviso, não como falha", () => {
    const [aviso] = syncNotices({ ...base, error: "Falha de autenticação" });
    expect(aviso.severity).toBe("warning");

    const [erro] = syncNotices({ ...base, error: "objeto corrompido" });
    expect(erro.severity).toBe("error");
  });

  it("acumula avisos independentes", () => {
    const notices = syncNotices({
      ...base,
      isShallow: true,
      needsPublish: true,
      error: "deu ruim",
    });
    expect(notices.map((n) => n.id)).toEqual(["shallow", "publish", "error"]);
  });
});

describe("syncSummary", () => {
  const sync: SyncInfoDto = {
    upstream: "origin/master",
    ahead: 2,
    behind: 1,
    lastFetchAt: null,
  };

  it("mostra upstream com divergência e ausência de sync", () => {
    const linhas = syncSummary({
      sync,
      credential: semCredencial,
      remoteUrl: null,
      sshUsername: null,
    });
    expect(linhas[0]).toBe("origin/master · ↑2 ↓1");
    expect(linhas[1]).toContain("Ainda não sincronizado");
  });

  it("omite divergência quando está em dia", () => {
    const linhas = syncSummary({
      sync: { ...sync, ahead: 0, behind: 0 },
      credential: semCredencial,
      remoteUrl: null,
      sshUsername: null,
    });
    expect(linhas[0]).toBe("origin/master");
  });

  it("identifica a conta conforme o protocolo do remoto", () => {
    const ssh = syncSummary({
      sync,
      credential: semCredencial,
      remoteUrl: "git@github.com:a/b.git",
      sshUsername: "wilton",
    });
    expect(ssh).toContain("GitHub SSH: @wilton");

    const https = syncSummary({
      sync,
      credential: {
        ...semCredencial,
        githubConnected: true,
        githubUsername: "wilton",
      },
      remoteUrl: "https://github.com/a/b.git",
      sshUsername: null,
    });
    expect(https).toContain("GitHub HTTPS: @wilton");
  });
});

describe("isAuthError", () => {
  it("reconhece as variações que o Git devolve", () => {
    expect(isAuthError("could not read Credential")).toBe(true);
    expect(isAuthError("falha de autenticação")).toBe(true);
    expect(isAuthError(null)).toBe(false);
    expect(isAuthError("arquivo não encontrado")).toBe(false);
  });
});
