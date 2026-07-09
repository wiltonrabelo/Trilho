import { Bot, Send, Settings2 } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import {
  chatAssistant,
  clearLlmApiKey,
  getAssistantSettings,
  setAssistantSettings,
  setLlmApiKey,
  testLlmConnection,
} from "@/lib/api";
import type {
  AssistantSettingsViewDto,
  AssistantUiContextDto,
  ChatMessageDto,
  LlmProviderKindDto,
  WriteRequestDto,
} from "@/types";

interface AssistantChatProps {
  onProposeWrite: (req: WriteRequestDto) => void;
  writeDisabled?: boolean;
  uiContext?: AssistantUiContextDto | null;
}

function writeLabel(req: WriteRequestDto): string {
  switch (req.kind) {
    case "stage":
      return `Stage ${req.path}`;
    case "stageMany":
      return `Stage ${req.paths.length} arquivos`;
    case "stageAll":
      return "Stage all";
    case "unstage":
      return `Unstage ${req.path}`;
    case "unstageMany":
      return `Unstage ${req.paths.length} arquivos`;
    case "unstageAll":
      return "Unstage all";
    case "commit":
      return `Commit: ${req.summary}`;
    case "push":
      return "Push";
    case "pullFfOnly":
      return "Pull (--ff-only)";
    case "revert":
      return `Revert ${req.commitId.slice(0, 7)}`;
    case "cherryPick": {
      const ids =
        req.commitIds && req.commitIds.length > 0
          ? req.commitIds
          : req.commitId
            ? [req.commitId]
            : [];
      return `Cherry-pick ${ids.map((id) => id.slice(0, 7)).join(", ") || "…"}`;
    }
    default:
      return req.kind;
  }
}

export function AssistantChat({
  onProposeWrite,
  writeDisabled,
  uiContext,
}: AssistantChatProps) {
  const [settings, setSettings] = useState<AssistantSettingsViewDto | null>(
    null,
  );
  const [showSettings, setShowSettings] = useState(false);
  const [messages, setMessages] = useState<ChatMessageDto[]>([]);
  const [input, setInput] = useState("");
  const [pendingWrites, setPendingWrites] = useState<WriteRequestDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  const [testHint, setTestHint] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  const reloadSettings = useCallback(async () => {
    try {
      const s = await getAssistantSettings();
      setSettings(s);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    void reloadSettings();
  }, [reloadSettings]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, pendingWrites]);

  async function saveSettings(patch: Partial<AssistantSettingsViewDto>) {
    if (!settings) return;
    const next = {
      enabled: patch.enabled ?? settings.enabled,
      provider: patch.provider ?? settings.provider,
      model: patch.model ?? settings.model,
      ollamaBaseUrl: patch.ollamaBaseUrl ?? settings.ollamaBaseUrl,
      sendMetadata: patch.sendMetadata ?? settings.sendMetadata,
      sendDiffs: patch.sendDiffs ?? settings.sendDiffs,
    };
    setLoading(true);
    setError(null);
    try {
      const saved = await setAssistantSettings(next);
      setSettings(saved);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleSaveKey() {
    if (!settings) return;
    const provider =
      settings.provider === "openAi"
        ? "openai"
        : settings.provider === "anthropic"
          ? "anthropic"
          : null;
    if (!provider) return;
    setLoading(true);
    setError(null);
    try {
      await setLlmApiKey(provider, apiKeyDraft);
      setApiKeyDraft("");
      await reloadSettings();
      setTestHint("Chave salva no Credential Manager.");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleClearKey() {
    if (!settings) return;
    const provider =
      settings.provider === "openAi"
        ? "openai"
        : settings.provider === "anthropic"
          ? "anthropic"
          : null;
    if (!provider) return;
    setLoading(true);
    try {
      await clearLlmApiKey(provider);
      await reloadSettings();
      setTestHint("Chave removida.");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleTest() {
    setLoading(true);
    setTestHint(null);
    setError(null);
    try {
      const r = await testLlmConnection();
      setTestHint(`Conexão OK: ${r}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleSend() {
    const text = input.trim();
    if (!text || loading || !settings) return;
    if (!settings.enabled) {
      setError("Ative o assistente nas configurações (opt-in).");
      setShowSettings(true);
      return;
    }
    if (!providerReady(settings)) {
      setError(readinessHint(settings) ?? "Complete a configuração do provedor.");
      setShowSettings(true);
      return;
    }
    const nextMessages: ChatMessageDto[] = [
      ...messages,
      { role: "user", content: text },
    ];
    setMessages(nextMessages);
    setInput("");
    setPendingWrites([]);
    setNotice(null);
    setLoading(true);
    setError(null);
    try {
      const resp = await chatAssistant(nextMessages, uiContext);
      setMessages([
        ...nextMessages,
        { role: "assistant", content: resp.reply },
      ]);
      setPendingWrites(resp.pendingWrites ?? []);
      setNotice(resp.notice ?? null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  if (!settings) {
    return (
      <div className="flex h-full items-center justify-center text-xs text-muted">
        Carregando assistente…
      </div>
    );
  }

  const ready = providerReady(settings);
  const statusLabel = readinessLabel(settings);
  const configHint = readinessHint(settings);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center justify-between border-b border-border px-3 py-1.5">
        <div className="flex items-center gap-1.5 text-[11px] text-muted">
          <Bot
            size={13}
            className={ready ? "text-accent" : "text-amber-600 dark:text-amber-400"}
          />
          <span title={configHint ?? undefined}>
            <span
              className={
                ready
                  ? undefined
                  : settings.enabled
                    ? "font-medium text-amber-700 dark:text-amber-300"
                    : undefined
              }
            >
              {statusLabel}
            </span>
            {" · "}
            {providerLabel(settings.provider)} · {settings.model}
          </span>
        </div>
        <button
          type="button"
          onClick={() => setShowSettings((v) => !v)}
          className="rounded p-1 text-muted hover:bg-bg hover:text-text"
          title="Configurações"
          aria-label="Configurações do assistente"
        >
          <Settings2 size={14} />
        </button>
      </div>

      {showSettings && (
        <div className="shrink-0 space-y-2 border-b border-border bg-bg/40 px-3 py-2 text-[11px]">
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={settings.enabled}
              onChange={(e) => void saveSettings({ enabled: e.target.checked })}
            />
            Ativar assistente (opt-in)
          </label>
          {configHint && (
            <p className="rounded border border-amber-500/40 bg-amber-500/10 px-2 py-1 text-[10px] text-amber-800 dark:text-amber-200">
              {configHint}
            </p>
          )}
          <div className="flex flex-wrap gap-2">
            <label className="flex items-center gap-1">
              Provedor
              <select
                className="rounded border border-border bg-surface px-1.5 py-0.5"
                value={settings.provider}
                onChange={(e) =>
                  void saveSettings({
                    provider: e.target.value as LlmProviderKindDto,
                  })
                }
              >
                <option value="ollama">Ollama (local)</option>
                <option value="openAi">OpenAI</option>
                <option value="anthropic">Anthropic</option>
              </select>
            </label>
            <label className="flex items-center gap-1">
              Modelo
              <input
                className="w-36 rounded border border-border bg-surface px-1.5 py-0.5"
                value={settings.model}
                onChange={(e) =>
                  setSettings({ ...settings, model: e.target.value })
                }
                onBlur={() => void saveSettings({ model: settings.model })}
              />
            </label>
          </div>
          {settings.provider === "ollama" && (
            <label className="flex items-center gap-1">
              URL Ollama
              <input
                className="min-w-[14rem] flex-1 rounded border border-border bg-surface px-1.5 py-0.5"
                value={settings.ollamaBaseUrl}
                onChange={(e) =>
                  setSettings({ ...settings, ollamaBaseUrl: e.target.value })
                }
                onBlur={() =>
                  void saveSettings({ ollamaBaseUrl: settings.ollamaBaseUrl })
                }
              />
            </label>
          )}
          {(settings.provider === "openAi" ||
            settings.provider === "anthropic") && (
            <div className="space-y-1">
              <p className="text-muted">
                Chave:{" "}
                <span
                  className={
                    (settings.provider === "openAi"
                      ? settings.hasOpenaiKey
                      : settings.hasAnthropicKey)
                      ? "text-accent"
                      : "font-medium text-amber-700 dark:text-amber-300"
                  }
                >
                  {settings.provider === "openAi"
                    ? settings.hasOpenaiKey
                      ? "salva"
                      : "ausente"
                    : settings.hasAnthropicKey
                      ? "salva"
                      : "ausente"}
                </span>
              </p>
              <div className="flex flex-wrap gap-1">
                <input
                  type="password"
                  placeholder="API key"
                  className="min-w-[12rem] flex-1 rounded border border-border bg-surface px-1.5 py-0.5"
                  value={apiKeyDraft}
                  onChange={(e) => setApiKeyDraft(e.target.value)}
                />
                <button
                  type="button"
                  className="btn-toolbar"
                  disabled={!apiKeyDraft.trim() || loading}
                  onClick={() => void handleSaveKey()}
                >
                  Salvar chave
                </button>
                <button
                  type="button"
                  className="btn-toolbar"
                  disabled={loading}
                  onClick={() => void handleClearKey()}
                >
                  Remover
                </button>
              </div>
            </div>
          )}
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={settings.sendMetadata}
              onChange={(e) =>
                void saveSettings({ sendMetadata: e.target.checked })
              }
            />
            Enviar metadados (branch/status)
          </label>
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={settings.sendDiffs}
              onChange={(e) =>
                void saveSettings({ sendDiffs: e.target.checked })
              }
            />
            Enviar diffs ao provedor
          </label>
          <div className="flex gap-2">
            <button
              type="button"
              className="btn-toolbar"
              disabled={loading || !ready}
              onClick={() => void handleTest()}
              title={!ready ? (configHint ?? undefined) : undefined}
            >
              Testar conexão
            </button>
            {testHint && <span className="text-muted">{testHint}</span>}
          </div>
          <p className="text-[10px] leading-snug text-muted">
            Allowlist: leitura, stage/unstage/commit, push, pull, revert,
            cherry-pick, blame/grafo e ajuda do Trilho. Toda escrita passa pelo
            preview (RF-08). Reset e force push ficam bloqueados via assistente.
          </p>
        </div>
      )}

      <div className="min-h-0 flex-1 space-y-2 overflow-auto px-3 py-2">
        {messages.length === 0 && (
          <p className="text-[11px] text-muted">
            Peça em português, por exemplo: «como funciona o stash?», «faz push»,
            «reverte este commit» ou «quem alterou esta linha?».
          </p>
        )}
        {messages.map((m, i) => (
          <div
            key={`${m.role}-${i}`}
            className={`rounded-lg px-2.5 py-1.5 text-[11px] leading-snug ${
              m.role === "user"
                ? "ml-6 bg-accent/15 text-text"
                : "mr-6 bg-bg/80 text-text"
            }`}
          >
            <span className="mb-0.5 block text-[9px] font-semibold uppercase tracking-wide text-muted">
              {m.role === "user" ? "Você" : "Assistente"}
            </span>
            <p className="whitespace-pre-wrap">{m.content}</p>
          </div>
        ))}
        {pendingWrites.length > 0 && (
          <div className="rounded-lg border border-accent/40 bg-accent/10 px-2.5 py-2">
            <p className="mb-1 text-[10px] font-semibold text-accent">
              Ações propostas (confirme no preview)
            </p>
            <ul className="space-y-1">
              {pendingWrites.map((w, i) => (
                <li
                  key={`${w.kind}-${i}`}
                  className="flex items-center justify-between gap-2"
                >
                  <span className="font-mono text-[10px]">{writeLabel(w)}</span>
                  <button
                    type="button"
                    className="btn-toolbar-primary shrink-0"
                    disabled={writeDisabled || loading}
                    onClick={() => onProposeWrite(w)}
                  >
                    Pré-visualizar
                  </button>
                </li>
              ))}
            </ul>
          </div>
        )}
        {notice && (
          <p className="text-[10px] text-amber-700 dark:text-amber-300">
            {notice}
          </p>
        )}
        {error && (
          <p className="text-[10px] text-red-600 dark:text-red-400">{error}</p>
        )}
        <div ref={bottomRef} />
      </div>

      <div className="flex shrink-0 gap-1.5 border-t border-border px-2 py-2">
        <input
          className="min-w-0 flex-1 rounded-lg border border-border bg-surface px-2 py-1.5 text-xs"
          placeholder="Pergunte ou peça uma ação…"
          value={input}
          disabled={loading}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              void handleSend();
            }
          }}
        />
        <button
          type="button"
          className="btn-toolbar-primary flex items-center gap-1"
          disabled={loading || !input.trim() || !ready}
          onClick={() => void handleSend()}
          aria-label="Enviar"
          title={!ready ? (configHint ?? undefined) : undefined}
        >
          <Send size={13} />
          {loading ? "…" : "Enviar"}
        </button>
      </div>
    </div>
  );
}

function providerLabel(p: LlmProviderKindDto): string {
  switch (p) {
    case "ollama":
      return "Ollama";
    case "openAi":
      return "OpenAI";
    case "anthropic":
      return "Anthropic";
  }
}

/** Opt-in ligado E credenciais/modelo mínimos do provedor atual. */
function providerReady(s: AssistantSettingsViewDto): boolean {
  if (!s.enabled) return false;
  if (!s.model.trim()) return false;
  switch (s.provider) {
    case "ollama":
      return Boolean(s.ollamaBaseUrl.trim());
    case "openAi":
      return s.hasOpenaiKey;
    case "anthropic":
      return s.hasAnthropicKey;
  }
}

function readinessLabel(s: AssistantSettingsViewDto): string {
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

function readinessHint(s: AssistantSettingsViewDto): string | null {
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
