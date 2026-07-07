import {
  Archive,
  ChevronDown,
  ChevronRight,
  Cloud,
  GitBranch,
  Search,
  Tag,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";

import {
  filterBranches,
  filterRemoteBranches,
  filterStashes,
  filterTags,
  groupByRemote,
} from "@/lib/refs-filter";
import type { RemoteBranchRefDto, StashEntryDto, TagEntryDto } from "@/types";

const SECTION_STORAGE_KEY = "trilho.refs.sections.v3";

type SectionKey = "locals" | "remotes" | "tags" | "stashes";

interface SectionState {
  locals: boolean;
  remotes: boolean;
  tags: boolean;
  stashes: boolean;
}

function loadSectionState(): SectionState {
  try {
    const raw =
      localStorage.getItem(SECTION_STORAGE_KEY) ??
      localStorage.getItem("trilho.refs.sections.v2") ??
      localStorage.getItem("trilho.refs.sections.v1");
    if (!raw) return { locals: true, remotes: true, tags: true, stashes: true };
    const parsed = JSON.parse(raw) as Partial<SectionState>;
    return {
      locals: parsed.locals !== false,
      remotes: parsed.remotes !== false,
      tags: parsed.tags !== false,
      stashes: parsed.stashes !== false,
    };
  } catch {
    return { locals: true, remotes: true, tags: true, stashes: true };
  }
}

function persistSectionState(state: SectionState) {
  localStorage.setItem(SECTION_STORAGE_KEY, JSON.stringify(state));
}

interface RefsPanelProps {
  branches: string[];
  remoteBranches: RemoteBranchRefDto[];
  tags: TagEntryDto[];
  stashes: StashEntryDto[];
  currentBranch?: string | null;
  focusedBranch?: string | null;
  loading?: boolean;
  tagsLoading?: boolean;
  stashesLoading?: boolean;
  writeDisabled?: boolean;
  onFocusBranch: (branch: string) => void;
  onSwitchLocal: (branch: string) => void;
  onSwitchRemote: (remote: string, branch: string) => void;
  onStashApply: (index: number) => void;
  onStashPop: (index: number) => void;
  onStashDrop: (index: number) => void;
  onTagSelect: (commitId: string) => void;
  onTagDelete: (name: string) => void;
}

function CollapsibleSection({
  title,
  icon,
  count,
  open,
  onToggle,
  children,
}: {
  title: string;
  icon: React.ReactNode;
  count: number;
  open: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}) {
  return (
    <section className="flex min-h-0 flex-col">
      <button
        type="button"
        onClick={onToggle}
        aria-expanded={open}
        className="flex shrink-0 items-center gap-1.5 rounded-md px-1 py-1 text-[11px] font-medium uppercase tracking-wide text-muted hover:bg-surface hover:text-text"
      >
        {open ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
        {icon}
        {title}
        <span className="ml-auto text-[10px] font-normal normal-case tracking-normal">
          {count}
        </span>
      </button>
      {open ? <div className="py-0.5">{children}</div> : null}
    </section>
  );
}

export function RefsPanel({
  branches,
  remoteBranches,
  tags,
  stashes,
  currentBranch,
  focusedBranch,
  loading,
  tagsLoading,
  stashesLoading,
  writeDisabled,
  onFocusBranch,
  onSwitchLocal,
  onSwitchRemote,
  onStashApply,
  onStashPop,
  onStashDrop,
  onTagSelect,
  onTagDelete,
}: RefsPanelProps) {
  const [query, setQuery] = useState("");
  const [sections, setSections] = useState<SectionState>(loadSectionState);

  useEffect(() => {
    persistSectionState(sections);
  }, [sections]);

  const toggleSection = useCallback((key: SectionKey) => {
    setSections((prev) => ({ ...prev, [key]: !prev[key] }));
  }, []);

  const filteredLocals = useMemo(
    () => filterBranches(branches, query),
    [branches, query],
  );
  const filteredRemotes = useMemo(
    () => filterRemoteBranches(remoteBranches, query),
    [remoteBranches, query],
  );
  const filteredStashes = useMemo(
    () => filterStashes(stashes, query),
    [stashes, query],
  );
  const filteredTags = useMemo(
    () => filterTags(tags, query),
    [tags, query],
  );
  const remoteGroups = useMemo(
    () => groupByRemote(filteredRemotes),
    [filteredRemotes],
  );

  const hasAny =
    branches.length > 0 ||
    remoteBranches.length > 0 ||
    tags.length > 0 ||
    stashes.length > 0 ||
    loading ||
    tagsLoading ||
    stashesLoading;

  if (!hasAny) {
    return null;
  }

  const emptyFilter =
    query.trim().length > 0 &&
    filteredLocals.length === 0 &&
    filteredRemotes.length === 0 &&
    filteredTags.length === 0 &&
    filteredStashes.length === 0;

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden px-3">
      <div className="relative mb-2 shrink-0">
        <Search
          size={12}
          className="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-muted"
        />
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Filtrar ramos, tags…"
          aria-label="Filtrar branches, remotos, tags e pilhas"
          className="w-full rounded-md border border-border bg-bg py-1.5 pl-7 pr-2 text-xs text-text placeholder:text-muted focus:border-accent focus:outline-none"
        />
      </div>

      {loading && branches.length === 0 && stashes.length === 0 ? (
        <p className="shrink-0 text-xs text-muted">Carregando…</p>
      ) : emptyFilter ? (
        <p className="shrink-0 text-xs text-muted">
          Nenhuma ref corresponde ao filtro.
        </p>
      ) : (
        <div className="min-h-0 flex-1 overflow-y-auto">
          <div className="flex flex-col gap-2 pb-1">
            {filteredLocals.length > 0 || (!query && branches.length > 0) ? (
              <CollapsibleSection
                title="Ramos"
                icon={<GitBranch size={12} />}
                count={filteredLocals.length}
                open={sections.locals}
                onToggle={() => toggleSection("locals")}
              >
                {filteredLocals.length === 0 ? (
                  <p className="px-2 text-xs text-muted">Nenhum ramo local.</p>
                ) : (
                  <ul className="flex flex-col gap-0.5">
                    {filteredLocals.map((branch) => {
                      const active = branch === currentBranch;
                      const focused = branch === focusedBranch;
                      return (
                        <li key={branch}>
                          <button
                            type="button"
                            disabled={active}
                            onClick={() => onFocusBranch(branch)}
                            onDoubleClick={() => {
                              if (!active) onSwitchLocal(branch);
                            }}
                            title={
                              active
                                ? "Branch em checkout"
                                : `Clique: commits exclusivos de ${branch} · Duplo clique: checkout`
                            }
                            className={`w-full truncate rounded-md px-2 py-1 text-left text-xs ${
                              active
                                ? "bg-accent/15 font-medium text-accent"
                                : focused
                                  ? "bg-amber-500/15 font-medium text-amber-700 dark:text-amber-300"
                                  : "text-text hover:bg-surface"
                            } disabled:cursor-default`}
                          >
                            {branch}
                            {active ? " ✓" : focused ? " ◉" : ""}
                          </button>
                        </li>
                      );
                    })}
                  </ul>
                )}
              </CollapsibleSection>
            ) : null}

            {remoteGroups.length > 0 ? (
              <CollapsibleSection
                title="Remotos"
                icon={<Cloud size={12} />}
                count={filteredRemotes.length}
                open={sections.remotes}
                onToggle={() => toggleSection("remotes")}
              >
                <div className="flex flex-col gap-2">
                  {remoteGroups.map(([remote, refs]) => (
                    <div key={remote}>
                      <div className="px-2 pb-0.5 text-[10px] font-medium text-muted">
                        {remote}
                      </div>
                      <ul className="flex flex-col gap-0.5">
                        {refs.map((ref) => {
                          const label = `${ref.remote}/${ref.branch}`;
                          const active = ref.branch === currentBranch;
                          const focused = ref.branch === focusedBranch;
                          const hasLocal = branches.includes(ref.branch);
                          return (
                            <li key={label}>
                              <button
                                type="button"
                                disabled={active}
                                onClick={() => onFocusBranch(ref.branch)}
                                onDoubleClick={() => {
                                  if (active) return;
                                  if (hasLocal) {
                                    onSwitchLocal(ref.branch);
                                  } else {
                                    onSwitchRemote(ref.remote, ref.branch);
                                  }
                                }}
                                title={
                                  active
                                    ? "Branch em checkout"
                                    : hasLocal
                                      ? `Clique: commits exclusivos · Duplo clique: checkout em ${ref.branch}`
                                      : `Clique: commits exclusivos · Duplo clique: criar e rastrear ${label}`
                                }
                                className={`w-full truncate rounded-md px-2 py-1 text-left text-xs ${
                                  active
                                    ? "bg-accent/15 font-medium text-accent"
                                    : focused
                                      ? "bg-amber-500/15 font-medium text-amber-700 dark:text-amber-300"
                                      : "text-muted hover:bg-surface hover:text-text"
                                } disabled:cursor-default`}
                              >
                                {ref.branch}
                                {!hasLocal ? " ↓" : ""}
                                {active ? " ✓" : focused ? " ◉" : ""}
                              </button>
                            </li>
                          );
                        })}
                      </ul>
                    </div>
                  ))}
                </div>
              </CollapsibleSection>
            ) : null}

            {!query || filteredTags.length > 0 || tags.length > 0 ? (
              <CollapsibleSection
                title="Tags"
                icon={<Tag size={12} />}
                count={filteredTags.length}
                open={sections.tags}
                onToggle={() => toggleSection("tags")}
              >
                {tagsLoading && tags.length === 0 ? (
                  <p className="px-2 text-xs text-muted">Carregando…</p>
                ) : filteredTags.length === 0 ? (
                  <p className="px-2 text-xs text-muted">Nenhuma tag.</p>
                ) : (
                  <ul className="flex flex-col gap-0.5">
                    {filteredTags.map((tag) => (
                      <li
                        key={tag.name}
                        className="rounded-md px-2 py-1 hover:bg-surface"
                      >
                        <button
                          type="button"
                          onClick={() => onTagSelect(tag.commitId)}
                          title={`Ir para o commit ${tag.shortId}`}
                          className="w-full truncate text-left text-xs"
                        >
                          <span className="font-medium text-amber-600 dark:text-amber-400">
                            {tag.name}
                          </span>
                          <span className="ml-1 font-mono text-[10px] text-muted">
                            {tag.shortId}
                          </span>
                        </button>
                        <button
                          type="button"
                          onClick={() => onTagDelete(tag.name)}
                          className="mt-0.5 text-[10px] text-red-600 hover:underline dark:text-red-400"
                        >
                          Excluir
                        </button>
                      </li>
                    ))}
                  </ul>
                )}
              </CollapsibleSection>
            ) : null}

            {!query || filteredStashes.length > 0 || stashes.length > 0 ? (
              <CollapsibleSection
                title="Pilhas"
                icon={<Archive size={12} />}
                count={filteredStashes.length}
                open={sections.stashes}
                onToggle={() => toggleSection("stashes")}
              >
                {stashesLoading && stashes.length === 0 ? (
                  <p className="px-2 text-xs text-muted">Carregando…</p>
                ) : filteredStashes.length === 0 ? (
                  <p className="px-2 text-xs text-muted">
                    Nenhum stash guardado.
                  </p>
                ) : (
                  <ul className="flex flex-col gap-1">
                    {filteredStashes.map((stash) => (
                      <li
                        key={stash.reference}
                        className="rounded-md px-2 py-1 hover:bg-surface"
                      >
                        <div
                          className="truncate text-xs text-text"
                          title={stash.message}
                        >
                          <span className="font-mono text-[10px] text-muted">
                            {stash.reference}
                          </span>
                          <span className="ml-1">{stash.message}</span>
                        </div>
                        {!writeDisabled ? (
                          <div className="mt-1 flex flex-wrap gap-2">
                            <button
                              type="button"
                              onClick={() => onStashApply(stash.index)}
                              className="text-[10px] text-accent hover:underline"
                            >
                              Aplicar
                            </button>
                            <button
                              type="button"
                              onClick={() => onStashPop(stash.index)}
                              className="text-[10px] text-accent hover:underline"
                            >
                              Pop
                            </button>
                            <button
                              type="button"
                              onClick={() => onStashDrop(stash.index)}
                              className="text-[10px] text-red-600 hover:underline dark:text-red-400"
                            >
                              Excluir
                            </button>
                          </div>
                        ) : null}
                      </li>
                    ))}
                  </ul>
                )}
              </CollapsibleSection>
            ) : null}
          </div>
        </div>
      )}
    </div>
  );
}
