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

function writeSuccessMessage(req: WriteRequestDto): string {
  switch (req.kind) {
    case "stage":
      return `Executado: «${req.path}» está em stage.`;
    case "stageMany":
      return `Executado: ${req.paths.length} arquivo(s) em stage.`;
    case "stageAll":
      return "Executado: todos os arquivos alterados estão em stage.";
    case "unstage":
      return `Executado: «${req.path}» voltou para working tree (unstaged).`;
    case "unstageMany":
      return `Executado: ${req.paths.length} arquivo(s) voltaram para working tree.`;
    case "unstageAll":
      return "Executado: todos os arquivos voltaram para working tree (unstaged).";
    case "commit":
      return `Executado: commit «${req.summary}».`;
    case "push":
      return "Executado: push concluído.";
    case "pullFfOnly":
      return "Executado: pull (--ff-only) concluído.";
    case "revert":
      return `Executado: revert do commit ${req.commitId.slice(0, 7)}.`;
    case "cherryPick": {
      const ids =
        req.commitIds && req.commitIds.length > 0
          ? req.commitIds
          : req.commitId
            ? [req.commitId]
            : [];
      return `Executado: cherry-pick ${ids.map((id) => id.slice(0, 7)).join(", ") || "…"}.`;
    }
    default:
      return `Executado: ${writeLabel(req)}.`;
  }
}

function writesMatch(a: WriteRequestDto, b: WriteRequestDto): boolean {
  if (a.kind !== b.kind) return false;
  switch (a.kind) {
    case "stage":
    case "unstage":
    case "discardWorktree":
    case "removeUntracked":
    case "saveWorktreeFile":
      return b.kind === a.kind && a.path === b.path;
    case "stageMany":
    case "unstageMany":
    case "discardWorktreeMany":
    case "removeUntrackedMany":
      return (
        b.kind === a.kind &&
        a.paths.length === b.paths.length &&
        a.paths.every((p, i) => p === b.paths[i])
      );
    case "commit":
      return b.kind === a.kind && a.summary === b.summary;
    case "revert":
    case "reword":
      return b.kind === a.kind && a.commitId === b.commitId;
    case "cherryPick":
      return b.kind === a.kind && JSON.stringify(a) === JSON.stringify(b);
    default:
      return JSON.stringify(a) === JSON.stringify(b);
  }
}

export function AssistantChat({
  onProposeWrite,
  writeDisabled,
  uiContext,
  writeCompleted,
  onWriteCompletedAck,
}: AssistantChatProps) {
  const [settings, setSettings] = useState<AssistantSettingsViewDto | null>(
    null,
  );
  const [showSettings, setShowSettings] = useState(false);
  const [messages, setMessages] = useState<ChatMessageView[]>([]);
  const [input, setInput] = useState("");
  const [pendingWrites, setPendingWrites] = useState<WriteRequestDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [thinkingSince, setThinkingSince] = useState<number | null>(null);
  const [settingsSaving, setSettingsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  const [testHint, setTestHint] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const settingsRef = useRef<AssistantSettingsViewDto | null>(null);
  const saveSeqRef = useRef(0);

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  const defaultModelFor = (provider: LlmProviderKindDto): string => {
    switch (provider) {
      case "ollama":
        return "llama3.2";
      case "openAi":
        return "gpt-4o-mini";
      case "anthropic":
        return "claude-3-5-haiku-latest";
    }
  };

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
    setPendingWrites((prev) =>
      prev.filter((w) => !writesMatch(w, writeCompleted.req)),
    );
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
                <option value="openAi">OpenAI</option>
                <option value="anthropic">Anthropic</option>
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
                onBlur={(e) =>
                  void saveSettings({ model: e.target.value.trim() })
                }
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
                Ollama é local — não usa API key. Informe URL e modelo (ex. llama3.2).
              </p>
            </div>
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
              disabled={settingsSaving}
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
            key={`${m.role}-${m.at}-${i}`}
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
              <span className="shrink-0 font-mono text-[9px] text-muted">
                {formatMessageTime(m.at)}
                {m.responseSecs != null && (
                  <span title="Tempo de resposta">
                    {" "}
                    · {formatDuration(m.responseSecs)}
                  </span>
                )}
              </span>
            </div>
            <p className="whitespace-pre-wrap">{m.content}</p>
          </div>
        ))}
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
