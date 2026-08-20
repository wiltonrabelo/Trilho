import type { AssistantSettingsViewDto, LlmProviderKindDto } from "@/types";

/** Nome do provedor como aparece na UI. */
export function providerLabel(provider: LlmProviderKindDto): string {
  switch (provider) {
    case "ollama":
      return "Ollama";
    case "openAi":
      return "OpenAI";
    case "anthropic":
      return "Anthropic";
    case "codexCli":
      return "Codex CLI";
  }
}

/** Modelo sugerido ao trocar de provedor. */
export function defaultModelFor(provider: LlmProviderKindDto): string {
  switch (provider) {
    case "ollama":
      return "llama3.2";
    case "openAi":
      return "gpt-4o-mini";
    case "anthropic":
      return "claude-3-5-haiku-latest";
    case "codexCli":
      return "gpt-5.4-mini";
  }
}

/** Opt-in ligado E credenciais/modelo mínimos do provedor atual. */
export function providerReady(s: AssistantSettingsViewDto): boolean {
  if (!s.enabled) return false;
  if (!s.model.trim()) return false;
  switch (s.provider) {
    case "ollama":
      return Boolean(s.ollamaBaseUrl.trim());
    case "openAi":
      return s.hasOpenaiKey;
    case "anthropic":
      return s.hasAnthropicKey;
    case "codexCli":
      // Auth fica no CLI do usuário; o Trilho só precisa do modelo.
      return true;
  }
}

/** Estado do assistente em uma palavra, para o cabeçalho do painel. */
export function readinessLabel(s: AssistantSettingsViewDto): string {
  if (!s.enabled) return "Desligado";
  if (providerReady(s)) return "Ativo";
  if (
    (s.provider === "openAi" && !s.hasOpenaiKey) ||
    (s.provider === "anthropic" && !s.hasAnthropicKey)
  ) {
    return "Sem chave";
  }
  return "Incompleto";
}

/** O que falta para o assistente ficar utilizável, ou `null` se já está. */
export function readinessHint(s: AssistantSettingsViewDto): string | null {
  if (!s.enabled || providerReady(s)) return null;
  if (!s.model.trim()) return "Informe o modelo.";
  if (s.provider === "openAi" && !s.hasOpenaiKey) {
    return "Salve a API key da OpenAI para usar o assistente.";
  }
  if (s.provider === "anthropic" && !s.hasAnthropicKey) {
    return "Salve a API key da Anthropic para usar o assistente.";
  }
  if (s.provider === "ollama" && !s.ollamaBaseUrl.trim()) {
    return "Informe a URL do Ollama.";
  }
  return "Complete a configuração do provedor.";
}
