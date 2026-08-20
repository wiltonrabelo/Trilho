import { describe, expect, it } from "vitest";

import type { AssistantSettingsViewDto } from "@/types";

import {
  defaultModelFor,
  providerLabel,
  providerReady,
  readinessHint,
  readinessLabel,
} from "./assistantProviders";

function settings(
  over: Partial<AssistantSettingsViewDto> = {}
): AssistantSettingsViewDto {
  return {
    enabled: true,
    provider: "ollama",
    model: "llama3.2",
    ollamaBaseUrl: "http://localhost:11434",
    sendMetadata: true,
    sendDiffs: false,
    hasOpenaiKey: false,
    hasAnthropicKey: false,
    ...over,
  };
}

describe("providerLabel / defaultModelFor", () => {
  it("cobre todos os provedores com rótulo e modelo padrão", () => {
    for (const p of [
      "ollama",
      "openAi",
      "anthropic",
      "codexCli",
    ] as const) {
      expect(providerLabel(p)).toBeTruthy();
      expect(defaultModelFor(p)).toBeTruthy();
    }
  });
});

describe("providerReady", () => {
  it("desligado nunca está pronto", () => {
    expect(providerReady(settings({ enabled: false }))).toBe(false);
  });

  it("modelo em branco bloqueia qualquer provedor", () => {
    expect(providerReady(settings({ model: "   " }))).toBe(false);
  });

  it("ollama precisa da URL", () => {
    expect(providerReady(settings())).toBe(true);
    expect(providerReady(settings({ ollamaBaseUrl: "" }))).toBe(false);
  });

  it("openAi e anthropic precisam da chave salva", () => {
    expect(providerReady(settings({ provider: "openAi" }))).toBe(false);
    expect(providerReady(settings({ provider: "openAi", hasOpenaiKey: true }))).toBe(
      true
    );
    expect(providerReady(settings({ provider: "anthropic" }))).toBe(false);
    expect(
      providerReady(settings({ provider: "anthropic", hasAnthropicKey: true }))
    ).toBe(true);
  });

  it("codexCli autentica sozinho: basta o modelo", () => {
    expect(
      providerReady(settings({ provider: "codexCli", model: "gpt-5.4-mini" }))
    ).toBe(true);
  });
});

describe("readinessLabel / readinessHint", () => {
  it("pronto não tem dica", () => {
    expect(readinessLabel(settings())).toBe("Ativo");
    expect(readinessHint(settings())).toBeNull();
  });

  it("desligado", () => {
    expect(readinessLabel(settings({ enabled: false }))).toBe("Desligado");
    expect(readinessHint(settings({ enabled: false }))).toBeNull();
  });

  it("falta de chave é reportada como tal", () => {
    const s = settings({ provider: "openAi" });
    expect(readinessLabel(s)).toBe("Sem chave");
    expect(readinessHint(s)).toContain("OpenAI");
  });

  it("modelo vazio pede o modelo antes de qualquer outra coisa", () => {
    const s = settings({ provider: "openAi", model: "" });
    expect(readinessLabel(s)).toBe("Sem chave");
    expect(readinessHint(s)).toBe("Informe o modelo.");
  });

  it("ollama sem URL fica incompleto", () => {
    const s = settings({ ollamaBaseUrl: "" });
    expect(readinessLabel(s)).toBe("Incompleto");
    expect(readinessHint(s)).toContain("Ollama");
  });
});
