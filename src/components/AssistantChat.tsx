import { Bot, Check, Copy, Send, Settings2 } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import {
  defaultModelFor,
  providerLabel,
  providerReady,
  readinessHint,
  readinessLabel,
} from "@/lib/assistantProviders";
import { writeLabel, writeSuccessMessage } from "@/lib/operationLabels";
import { writesMatch } from "@/lib/writeRequestMatch";
import {
  chatAssistant,
  clearLlmApiKey,
  getAssistantSettings,
  setAssistantSettings,
  setLlmApiKey,
  testLlmConnection,
} from "@/lib/api";
import type {
  AssistantSettingsDto,
  AssistantSettingsViewDto,
  AssistantUiContextDto,
  AssistantWriteCompletedDto,
  ChatMessageDto,
  LlmProviderKindDto,
  WriteRequestDto,
} from "@/types";

interface AssistantChatProps {
  onProposeWrite: (req: WriteRequestDto) => void;
  writeDisabled?: boolean;
  uiContext?: AssistantUiContextDto | null;
  writeCompleted?: AssistantWriteCompletedDto | null;
  onWriteCompletedAck?: () => void;
}

type ChatMessageRole = "user" | "assistant" | "system";

interface ChatMessageView {
  role: ChatMessageRole;
  content: string;
  at: number;
  responseSecs?: number;
}

function formatMessageTime(ms: number): string {
  return new Date(ms).toLocaleTimeString("pt-BR", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function formatDuration(secs: number): string {
  if (secs < 1) return "<1s";
  if (secs < 60) return `${secs.toFixed(1)}s`;
  const m = Math.floor(secs / 60);
  const s = Math.round(secs % 60);
  return `${m}m ${s}s`;
}

function roleLabel(role: ChatMessageRole): string {
  switch (role) {
    case "user":
      return "Você";
    case "assistant":
      return "Assistente";
    case "system":
      return "Sistema";
  }
}

export function AssistantChat({
  onProposeWrite,
  writeDisabled,
  uiContext,
  writeCompleted,
  onWriteCompletedAck,
}: AssistantChatProps) {
  const [settings, setSettings] = useState<AssistantSettingsViewDto | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [messages, setMessages] = useState<ChatMessageView[]>([]);
  const [input, setInput] = useState("");
  const [pendingWrites, setPendingWrites] = useState<WriteRequestDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [thinkingSince, setThinkingSince] = useState<number | null>(null);
  const [settingsSaving, setSettingsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const copyTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  const [testHint, setTestHint] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const settingsRef = useRef<AssistantSettingsViewDto | null>(null);
  const saveSeqRef = useRef(0);

  useEffect(() => {
    return () => {
      if (copyTimerRef.current) clearTimeout(copyTimerRef.current);
    };
  }, []);

  const copyMessage = useCallback(async (key: string, text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedKey(key);
      if (copyTimerRef.current) clearTimeout(copyTimerRef.current);
      copyTimerRef.current = setTimeout(() => setCopiedKey(null), 1500);
    } catch {
      /* clipboard pode falhar sem permissão */
    }
  }, []);

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  const toSettingsDto = (view: AssistantSettingsViewDto): AssistantSettingsDto => ({
    enabled: view.enabled,
    provider: view.provider,
    model: view.model,
    ollamaBaseUrl: view.ollamaBaseUrl,
    sendMetadata: view.sendMetadata,
    sendDiffs: view.sendDiffs,
  });

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
  }, [messages, pendingWrites, loading, thinkingSince]);

  useEffect(() => {
    if (!writeCompleted) return;
    const at = Date.now();
    setMessages((prev) => [
      ...prev,
      {
        role: "system",
        content: writeSuccessMessage(writeCompleted.req),
        at,
      },
    ]);
    setPendingWrites((prev) => prev.filter((w) => !writesMatch(w, writeCompleted.req)));
    onWriteCompletedAck?.();
  }, [writeCompleted, onWriteCompletedAck]);

  async function saveSettings(patch: Partial<AssistantSettingsViewDto>) {
    const base = settingsRef.current;
    if (!base) return;

    const nextView: AssistantSettingsViewDto = {
      ...base,
      ...patch,
    };
    const seq = ++saveSeqRef.current;

    setSettings(nextView);
    settingsRef.current = nextView;
    setSettingsSaving(true);
    setError(null);

    try {
      const saved = await setAssistantSettings(toSettingsDto(nextView));
      if (seq !== saveSeqRef.current) return;
      setSettings(saved);
      settingsRef.current = saved;
    } catch (e) {
      if (seq !== saveSeqRef.current) return;
      setSettings(base);
      settingsRef.current = base;
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      if (seq === saveSeqRef.current) {
        setSettingsSaving(false);
      }
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
    const userAt = Date.now();
    const userMsg: ChatMessageView = { role: "user", content: text, at: userAt };
    const nextMessages: ChatMessageDto[] = [
      ...messages
        .filter((m) => m.role === "user" || m.role === "assistant")
        .map(({ role, content }) => ({ role, content })),
      { role: "user", content: text },
    ];
    setMessages((prev) => [...prev, userMsg]);
    setInput("");
    setPendingWrites([]);
    setNotice(null);
    setLoading(true);
    setThinkingSince(userAt);
    setError(null);
    try {
      const resp = await chatAssistant(nextMessages, uiContext);
      const assistantAt = Date.now();
      const responseSecs = (assistantAt - userAt) / 1000;
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          content: resp.reply,
          at: assistantAt,
          responseSecs,
        },
      ]);
      setPendingWrites(resp.pendingWrites ?? []);
      setNotice(resp.notice ?? null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
      setThinkingSince(null);
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
              disabled={settingsSaving}
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
                disabled={settingsSaving}
                onChange={(e) => {
                  const provider = e.target.value as LlmProviderKindDto;
                  void saveSettings({
                    provider,
                    model: defaultModelFor(provider),
                  });
                }}
              >
                <option value="ollama">Ollama (local)</option>
                <option value="openAi">OpenAI (API key)</option>
                <option value="codexCli">Codex CLI (ChatGPT)</option>
                <option value="anthropic">Anthropic (API key)</option>
                <option value="claudeCode">Claude Code (plano)</option>
              </select>
            </label>
            <label className="flex items-center gap-1">
              Modelo
              <input
                className="w-36 rounded border border-border bg-surface px-1.5 py-0.5"
                value={settings.model}
                disabled={settingsSaving}
                onChange={(e) => {
                  const next = { ...settings, model: e.target.value };
                  setSettings(next);
                  settingsRef.current = next;
                }}
                onBlur={(e) => void saveSettings({ model: e.target.value.trim() })}
              />
            </label>
          </div>
          {settings.provider === "ollama" && (
            <div className="space-y-1">
              <label className="flex items-center gap-1">
                URL Ollama
                <input
                  className="min-w-[14rem] flex-1 rounded border border-border bg-surface px-1.5 py-0.5"
                  value={settings.ollamaBaseUrl}
                  disabled={settingsSaving}
                  onChange={(e) => {
                    const next = { ...settings, ollamaBaseUrl: e.target.value };
                    setSettings(next);
                    settingsRef.current = next;
                  }}
                  onBlur={(e) =>
                    void saveSettings({ ollamaBaseUrl: e.target.value.trim() })
                  }
                />
              </label>
              <p className="text-[10px] text-muted">
                Ollama via app local (URL abaixo). Modelos “:cloud” (ex. glm-5.2:cloud)
                usam a conta Ollama Cloud — exigem assinatura, não só login.
              </p>
            </div>
          )}
          {settings.provider === "claudeCode" && (
            <p className="text-[10px] leading-snug text-muted">
              Usa o Claude Code (CLI ou extensão VS Code/Cursor) já autenticado neste PC
              — não o app Desktop/chat. Tools do Trilho (leitura / propostas) usam o
              mesmo loop dos outros provedores. Cada mensagem sobe o CLI de novo (pode
              levar vários segundos a mais). Se só usa a extensão, o Trilho procura o
              binário em{" "}
              <span className="font-mono">
                .vscode/extensions/anthropic.claude-code-*
              </span>
              . Modelo: sonnet (ou opus / haiku). Plano Pro/Max — sem API key.
            </p>
          )}
          {settings.provider === "codexCli" && (
            <p className="text-[10px] leading-snug text-muted">
              Usa o Codex CLI (`codex exec`) com login ChatGPT neste PC — não a API key
              OpenAI. Rode <span className="font-mono">codex login</span> uma vez. Tools
              do Trilho usam o mesmo loop; o agent do Codex fica em sandbox read-only e
              cwd neutro (não edita o repo). Modelo típico: gpt-5.4-mini (ou o que o
              catálogo Codex listar).
            </p>
          )}
          {(settings.provider === "openAi" || settings.provider === "anthropic") && (
            <div className="space-y-1">
              <p className="text-muted">
                Chave:{" "}
                <span
                  className={
                    (
                      settings.provider === "openAi"
                        ? settings.hasOpenaiKey
                        : settings.hasAnthropicKey
                    )
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
                  disabled={!apiKeyDraft.trim() || loading || settingsSaving}
                  onClick={() => void handleSaveKey()}
                >
                  Salvar chave
                </button>
                <button
                  type="button"
                  className="btn-toolbar"
                  disabled={loading || settingsSaving}
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
              disabled={settingsSaving}
              onChange={(e) => void saveSettings({ sendMetadata: e.target.checked })}
            />
            Enviar metadados (branch/status)
          </label>
          <label className="flex flex-col gap-1">
            <span className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={settings.sendDiffs}
                disabled={settingsSaving}
                onChange={(e) => void saveSettings({ sendDiffs: e.target.checked })}
              />
              Enviar diffs ao provedor (revisão de código)
            </span>
            <span className="pl-6 text-[10px] leading-snug text-muted">
              Permite diffs e leitura de arquivos em refs. A revisão só cobre o que as
              tools buscarem (pode truncar); não varre o repo inteiro e não substitui
              testes/CI. O assistente deve avisar isso antes de revisar.
            </span>
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
            Allowlist: leitura, stage/unstage/commit, push, pull, revert, cherry-pick,
            blame/grafo e ajuda do Trilho. Toda escrita passa pelo preview (RF-08).
            Reset e force push ficam bloqueados via assistente.
          </p>
        </div>
      )}

      <div className="min-h-0 flex-1 space-y-2 overflow-auto px-3 py-2">
        {messages.length === 0 && (
          <p className="text-[11px] text-muted">
            Peça em português, por exemplo: «como funciona o stash?», «faz push»,
            «reverte este commit», «quem alterou esta linha?» ou «revise esta branch
            contra master» (com «Enviar diffs» ligado — revisão parcial).
          </p>
        )}
        {messages.map((m, i) => {
          const msgKey = `${m.role}-${m.at}-${i}`;
          const canCopy = m.role === "assistant" && m.content.trim().length > 0;
          return (
            <div
              key={msgKey}
              className={`rounded-lg px-2.5 py-1.5 text-[11px] leading-snug ${
                m.role === "user"
                  ? "ml-6 bg-accent/15 text-text"
                  : m.role === "system"
                    ? "mx-2 border border-emerald-500/30 bg-emerald-500/10 text-text"
                    : "mr-6 bg-bg/80 text-text"
              }`}
            >
              <div className="mb-0.5 flex items-center justify-between gap-2">
                <span className="text-[9px] font-semibold uppercase tracking-wide text-muted">
                  {roleLabel(m.role)}
                </span>
                <span className="flex shrink-0 items-center gap-1 font-mono text-[9px] text-muted">
                  {formatMessageTime(m.at)}
                  {m.responseSecs != null && (
                    <span title="Tempo de resposta">
                      · {formatDuration(m.responseSecs)}
                    </span>
                  )}
                  {canCopy && (
                    <button
                      type="button"
                      className="rounded p-0.5 text-muted opacity-50 transition-opacity hover:bg-surface hover:text-text hover:opacity-100"
                      title={copiedKey === msgKey ? "Copiado" : "Copiar resposta"}
                      aria-label={copiedKey === msgKey ? "Copiado" : "Copiar resposta"}
                      onClick={() => void copyMessage(msgKey, m.content)}
                    >
                      {copiedKey === msgKey ? (
                        <Check size={11} className="text-accent" />
                      ) : (
                        <Copy size={11} />
                      )}
                    </button>
                  )}
                </span>
              </div>
              <p className="whitespace-pre-wrap">{m.content}</p>
            </div>
          );
        })}
        {loading && thinkingSince != null && (
          <div className="mr-6 rounded-lg bg-bg/80 px-2.5 py-1.5 text-[11px] leading-snug text-text">
            <div className="mb-0.5 flex items-center justify-between gap-2">
              <span className="text-[9px] font-semibold uppercase tracking-wide text-muted">
                Assistente
              </span>
              <span className="font-mono text-[9px] text-muted">
                desde {formatMessageTime(thinkingSince)}
              </span>
            </div>
            <p className="animate-pulse text-muted">Pensando…</p>
          </div>
        )}
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
          <p className="text-[10px] text-amber-700 dark:text-amber-300">{notice}</p>
        )}
        {error && <p className="text-[10px] text-red-600 dark:text-red-400">{error}</p>}
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
