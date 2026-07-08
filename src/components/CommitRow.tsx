import { Cloud, GitBranch, Tag } from "lucide-react";

import type { CommitDto } from "@/types";

interface CommitRowProps {
  commit: CommitDto;
  selected: boolean;
  isHead: boolean;
  onSelect: (commit: CommitDto) => void;
  showSpineBelow: boolean;
  showDot?: boolean;
  dotColor?: string;
  isMerge?: boolean;
  rowHeight?: number;
  /** Nome da branch base quando esta linha é o ponto de divergência (RF-02). */
  divergenceBase?: string;
  /** Linha pertence ao trilho da base (abaixo da divergência) — esmaecida. */
  onBaseTrail?: boolean;
  /** Linha densa (grafo completo): uma linha só, estilo Git Graph do VS Code. */
  compact?: boolean;
}

const MAX_REF_CHIPS = 3;

/** Classifica uma ref pelo nome (prefixo `tag:` vindo do backend). */
function refKind(ref: string): "tag" | "remote" | "branch" {
  if (ref.startsWith("tag:")) return "tag";
  if (ref.includes("/")) return "remote";
  return "branch";
}

function refLabel(ref: string): string {
  return ref.startsWith("tag:") ? ref.slice(4) : ref;
}

/** Pílulas de ref estilo grafo do VS Code: ícone + nome; branch atual (HEAD)
 *  fica preenchida. Tags e remotos com cor/ícone próprios. */
function RefChips({ refs, isHead = false }: { refs: string[]; isHead?: boolean }) {
  if (!refs.length) return null;
  const shown = refs.slice(0, MAX_REF_CHIPS);
  const extra = refs.length - shown.length;
  return (
    <span className="flex shrink-0 items-center gap-1">
      {shown.map((r) => {
        const kind = refKind(r);
        const Icon = kind === "tag" ? Tag : kind === "remote" ? Cloud : GitBranch;
        const style =
          kind === "remote"
            ? "border-sky-500/40 bg-sky-500/10 text-sky-600 dark:text-sky-300"
            : kind === "tag"
              ? "border-amber-500/40 bg-amber-500/10 text-amber-600 dark:text-amber-400"
              : isHead
                ? "border-accent bg-accent text-white"
                : "border-accent/40 bg-accent/10 text-accent";
        return (
          <span
            key={r}
            title={refs.join(" · ")}
            className={`inline-flex shrink-0 items-center gap-0.5 rounded-full border px-1.5 py-0.5 font-mono text-[10px] leading-none ${style}`}
          >
            <Icon size={9} strokeWidth={2.5} className="shrink-0" />
            {refLabel(r)}
          </span>
        );
      })}
      {extra > 0 && (
        <span
          title={refs.join(" · ")}
          className="shrink-0 rounded-full border border-border px-1.5 py-0.5 text-[10px] leading-none text-muted"
        >
          +{extra}
        </span>
      )}
    </span>
  );
}

function formatRelativeTime(iso: string): string {
  const date = new Date(iso);
  const diffMs = Date.now() - date.getTime();
  const mins = Math.floor(diffMs / 60_000);
  if (mins < 1) return "agora";
  if (mins < 60) return `há ${mins} min`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `há ${hours} h`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `há ${days} dia${days > 1 ? "s" : ""}`;
  return date.toLocaleDateString("pt-BR");
}

export function CommitRow({
  commit,
  selected,
  isHead,
  onSelect,
  showSpineBelow,
  showDot = true,
  dotColor,
  isMerge,
  rowHeight = 56,
  divergenceBase,
  onBaseTrail = false,
  compact = false,
}: CommitRowProps) {
  const absTime = new Date(commit.authoredAt).toLocaleString("pt-BR");
  const relTime = formatRelativeTime(commit.authoredAt);

  if (compact) {
    // Grafo do Source Control do VS Code: uma linha só — mensagem + autor
    // esmaecido no mesmo texto truncável, refs (pílulas com ícone) à direita.
    // Sem colunas de data/hash: o detalhe completo fica no painel ao clicar.
    return (
      <li
        style={{ height: rowHeight }}
        className="relative flex items-center overflow-hidden"
      >
        <button
          type="button"
          onClick={() => onSelect(commit)}
          title={`${commit.summary}\n${commit.shortId} · ${commit.authorName} · ${absTime}`}
          aria-selected={selected}
          className={`flex w-full items-center gap-2 rounded px-2 py-0.5 text-left text-[13px] transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/30 ${
            selected ? "bg-accent/10 ring-1 ring-accent/25" : "hover:bg-surface"
          }`}
        >
          <span className="min-w-0 flex-1 truncate">
            <span className={onBaseTrail ? "text-muted" : "text-text"}>
              {commit.summary}
            </span>
            <span className="ml-2 text-muted">{commit.authorName}</span>
          </span>
          {isMerge && (
            <span className="shrink-0 rounded bg-muted/25 px-1 text-[10px] uppercase text-muted">
              M
            </span>
          )}
          <RefChips refs={commit.refs} isHead={isHead} />
        </button>
      </li>
    );
  }

  return (
    // Altura FIXA: o overlay SVG posiciona os nós por row*altura — uma linha
    // mais alta que o previsto desalinharia todos os nós abaixo dela.
    <li style={{ height: rowHeight }} className="relative flex items-center overflow-hidden">
      {showSpineBelow && (
        <span
          className="absolute left-[11px] top-6 bottom-0 w-px bg-border"
          aria-hidden
        />
      )}
      <button
        type="button"
        onClick={() => onSelect(commit)}
        title={absTime}
        aria-selected={selected}
        className={`flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/30 ${
          selected
            ? "bg-surface ring-1 ring-border"
            : "hover:bg-surface/60"
        }`}
      >
        {showDot && (
          <span
            className={`h-2.5 w-2.5 shrink-0 rounded-full ${
              !dotColor && !selected ? "bg-border" : ""
            }`}
            style={
              dotColor || selected
                ? { backgroundColor: dotColor ?? "rgb(var(--accent))" }
                : undefined
            }
          />
        )}
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1.5">
            <p
              className={`truncate font-medium leading-snug ${
                onBaseTrail ? "text-muted" : "text-text"
              }`}
            >
              {commit.summary}
            </p>
            {divergenceBase && (
              <span
                className="shrink-0 rounded bg-amber-500/15 px-1.5 py-0.5 text-[10px] font-semibold tracking-wide text-amber-600 dark:text-amber-400"
                title={`Merge-base: aqui a branch atual divergiu de ${divergenceBase}`}
              >
                ⑂ divergiu de {divergenceBase}
              </span>
            )}
            <RefChips refs={commit.refs} />
            {isHead && (
              <span className="shrink-0 rounded px-1 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-accent">
                HEAD
              </span>
            )}
            {isMerge && (
              <span className="shrink-0 rounded bg-muted/25 px-1 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-muted">
                merge
              </span>
            )}
            {commit.isLocalOnly && (
              <span className="shrink-0 rounded bg-amber-500/15 px-1 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-amber-600 dark:text-amber-400">
                local
              </span>
            )}
          </div>
          <p className="mt-0.5 truncate text-xs text-muted">
            <span className="font-mono text-[11px]">{commit.shortId}</span>
            <span className="mx-1.5 opacity-40">·</span>
            {commit.authorName}
            <span className="mx-1.5 opacity-40">·</span>
            <span title={absTime}>{relTime}</span>
          </p>
        </div>
      </button>
    </li>
  );
}
