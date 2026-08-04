import {
  Archive,
  ChevronDown,
  ChevronRight,
  Cloud,
  GitBranch,
  GitCompare,
  Search,
  Tag,
} from "lucide-react";
import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type MouseEvent,
} from "react";

import {
  ContextMenu,
  type ContextMenuItem,
} from "@/components/ContextMenu";
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
  onDeleteLocal?: (branch: string) => void;
  onDeleteRemote?: (remote: string, branch: string) => void;
  onStashApply: (index: number) => void;
  onStashPop: (index: number) => void;
  onStashDrop: (index: number) => void;
  onTagSelect: (commitId: string) => void;
  onTagDelete: (name: string) => void;
  onCompareBranches?: () => void;
}

type RefsMenuState =
  | {
      kind: "local";
      branch: string;
      x: number;
      y: number;
      active: boolean;
    }
  | {
      kind: "remote";
      remote: string;
      branch: string;
      x: number;
      y: number;
      active: boolean;
      hasLocal: boolean;
    }
  | {
      kind: "tag";
      name: string;
      commitId: string;
      x: number;
      y: number;
    }
  | {
      kind: "stash";
      index: number;
      reference: string;
      message: string;
      x: number;
      y: number;
    };

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
  onDeleteLocal,
  onDeleteRemote,
  onStashApply,
  onStashPop,
  onStashDrop,
  onTagSelect,
  onTagDelete,
  onCompareBranches,
}: RefsPanelProps) {
  const [query, setQuery] = useState("");
  const [sections, setSections] = useState<SectionState>(loadSectionState);
  const [menu, setMenu] = useState<RefsMenuState | null>(null);

  useEffect(() => {
    persistSectionState(sections);
  }, [sections]);

  const toggleSection = useCallback((key: SectionKey) => {
    setSections((prev) => ({ ...prev, [key]: !prev[key] }));
  }, []);

  const openLocalMenu = useCallback(
    (e: MouseEvent, branch: string) => {
      e.preventDefault();
      e.stopPropagation();
      setMenu({
        kind: "local",
        branch,
        x: e.clientX,
        y: e.clientY,
        active: branch === currentBranch,
      });
    },
    [currentBranch],
  );

  const openRemoteMenu = useCallback(
    (e: MouseEvent, remote: string, branch: string) => {
      e.preventDefault();
      e.stopPropagation();
      setMenu({
        kind: "remote",
        remote,
        branch,
        x: e.clientX,
        y: e.clientY,
        active: branch === currentBranch,
        hasLocal: branches.includes(branch),
      });
    },
    [branches, currentBranch],
  );

  const openTagMenu = useCallback((e: MouseEvent, name: string, commitId: string) => {
    e.preventDefault();
    e.stopPropagation();
    setMenu({ kind: "tag", name, commitId, x: e.clientX, y: e.clientY });
  }, []);

  const openStashMenu = useCallback(
    (e: MouseEvent, index: number, reference: string, message: string) => {
      e.preventDefault();
      e.stopPropagation();
      setMenu({
        kind: "stash",
        index,
        reference,
        message,
        x: e.clientX,
        y: e.clientY,
      });
    },
    [],
  );

  const menuItems: ContextMenuItem[] = useMemo(() => {
    if (!menu) return [];
    const busy = Boolean(writeDisabled || loading);

    if (menu.kind === "tag") {
      return [
        {
          id: "goto",
          label: "Ir para o commit",
          primary: true,
          onSelect: () => onTagSelect(menu.commitId),
        },
        {
          id: "delete-tag",
          label: "Excluir tag",
          separatorBefore: true,
          disabled: busy,
          onSelect: () => onTagDelete(menu.name),
        },
      ];
    }

    if (menu.kind === "stash") {
      return [
        {
          id: "apply",
          label: "Aplicar",
          primary: true,
          disabled: busy,
          onSelect: () => onStashApply(menu.index),
        },
        {
          id: "pop",
          label: "Pop (aplicar e remover)",
          disabled: busy,
          onSelect: () => onStashPop(menu.index),
        },
        {
          id: "drop",
          label: "Excluir",
          separatorBefore: true,
          disabled: busy,
          onSelect: () => onStashDrop(menu.index),
        },
      ];
    }

    if (menu.kind === "local") {
      const remotesForBranch = [
        ...new Set(
          remoteBranches
            .filter((r) => r.branch === menu.branch)
            .map((r) => r.remote),
        ),
      ];
      const allRemotes = [...new Set(remoteBranches.map((r) => r.remote))];
      const remotesToShow =
        remotesForBranch.length > 0
          ? remotesForBranch
          : allRemotes.includes("origin")
            ? ["origin"]
            : allRemotes.length > 0
              ? [allRemotes[0]!]
              : ["origin"];

      const items: ContextMenuItem[] = [
        {
          id: "checkout",
          label: "Checkout",
          disabled: menu.active || busy,
          primary: !menu.active,
          onSelect: () => onSwitchLocal(menu.branch),
        },
      ];

      if (onDeleteLocal) {
        items.push({
          id: "delete-local",
          label: "Remover localmente",
          separatorBefore: true,
          disabled: menu.active || busy,
          onSelect: () => onDeleteLocal(menu.branch),
        });
      }

      if (onDeleteRemote) {
        for (const remote of remotesToShow) {
          items.push({
            id: `delete-remote-${remote}`,
            label: `Remover no repositório remoto (${remote})`,
            separatorBefore: !onDeleteLocal && remote === remotesToShow[0],
            disabled: menu.active || busy,
            onSelect: () => onDeleteRemote(remote, menu.branch),
          });
        }
      }

      return items;
    }

    const items: ContextMenuItem[] = [
      {
        id: "checkout",
        label: "Checkout",
        disabled: menu.active || busy,
        primary: !menu.active,
        onSelect: () => {
          if (menu.hasLocal) {
            onSwitchLocal(menu.branch);
          } else {
            onSwitchRemote(menu.remote, menu.branch);
          }
        },
      },
    ];

    if (onDeleteRemote) {
      items.push({
        id: "delete-remote",
        label: `Remover no repositório remoto (${menu.remote})`,
        separatorBefore: true,
        disabled: menu.active || busy,
        onSelect: () => onDeleteRemote(menu.remote, menu.branch),
      });
    }

    if (onDeleteLocal && menu.hasLocal) {
      items.push({
        id: "delete-local",
        label: "Remover localmente",
        disabled: menu.active || busy,
        onSelect: () => onDeleteLocal(menu.branch),
      });
    }

    return items;
  }, [
    loading,
    menu,
    onDeleteLocal,
    onDeleteRemote,
    onStashApply,
    onStashDrop,
    onStashPop,
    onSwitchLocal,
    onSwitchRemote,
    onTagDelete,
    onTagSelect,
    remoteBranches,
    writeDisabled,
  ]);

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
      <div className="mb-2 flex shrink-0 items-center gap-1">
        <div className="relative min-w-0 flex-1">
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
        {onCompareBranches ? (
          <button
            type="button"
            onClick={onCompareBranches}
            title="Comparar duas branches (diff de arquivos)"
            aria-label="Comparar branches"
            className="shrink-0 rounded-md border border-border p-1.5 text-muted hover:bg-surface hover:text-accent"
          >
            <GitCompare size={14} />
          </button>
        ) : null}
      </div>

      {loading && branches.length === 0 && stashes.length === 0 ? (
        <p className="shrink-0 text-xs text-muted">Carregando…</p>
      ) : emptyFilter ? (
        <p className="shrink-0 text-xs text-muted">
          Nenhuma ref corresponde ao filtro.
        </p>
      ) : (
        <div
          className="min-h-0 flex-1 overflow-y-auto"
          onContextMenu={(e) => e.preventDefault()}
        >
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
                            onClick={() => onFocusBranch(branch)}
                            onDoubleClick={() => {
                              if (!active) onSwitchLocal(branch);
                            }}
                            onContextMenu={(e) => openLocalMenu(e, branch)}
                            title={
                              active
                                ? "Branch em checkout · Botão direito: ações"
                                : `Clique: commits exclusivos de ${branch} · Duplo clique: checkout · Botão direito: ações`
                            }
                            className={`w-full truncate rounded-md px-2 py-1 text-left text-xs ${
                              active
                                ? "bg-accent/15 font-medium text-accent"
                                : focused
                                  ? "bg-amber-500/15 font-medium text-amber-700 dark:text-amber-300"
                                  : "text-text hover:bg-surface"
                            }`}
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
                                onClick={() => onFocusBranch(ref.branch)}
                                onDoubleClick={() => {
                                  if (active) return;
                                  if (hasLocal) {
                                    onSwitchLocal(ref.branch);
                                  } else {
                                    onSwitchRemote(ref.remote, ref.branch);
                                  }
                                }}
                                onContextMenu={(e) =>
                                  openRemoteMenu(e, ref.remote, ref.branch)
                                }
                                title={
                                  active
                                    ? "Branch em checkout · Botão direito: ações"
                                    : hasLocal
                                      ? `Clique: commits exclusivos · Duplo clique: checkout em ${ref.branch} · Botão direito: ações`
                                      : `Clique: commits exclusivos · Duplo clique: criar e rastrear ${label} · Botão direito: ações`
                                }
                                className={`w-full truncate rounded-md px-2 py-1 text-left text-xs ${
                                  active
                                    ? "bg-accent/15 font-medium text-accent"
                                    : focused
                                      ? "bg-amber-500/15 font-medium text-amber-700 dark:text-amber-300"
                                      : "text-muted hover:bg-surface hover:text-text"
                                }`}
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
                      <li key={tag.name}>
                        <button
                          type="button"
                          onClick={() => onTagSelect(tag.commitId)}
                          onContextMenu={(e) =>
                            openTagMenu(e, tag.name, tag.commitId)
                          }
                          title={`Ir para o commit ${tag.shortId} · Botão direito: ações`}
                          className="w-full truncate rounded-md px-2 py-1 text-left text-xs hover:bg-surface"
                        >
                          <span className="font-medium text-amber-600 dark:text-amber-400">
                            {tag.name}
                          </span>
                          <span className="ml-1 font-mono text-[10px] text-muted">
                            {tag.shortId}
                          </span>
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
                  <ul className="flex flex-col gap-0.5">
                    {filteredStashes.map((stash) => (
                      <li key={stash.reference}>
                        <button
                          type="button"
                          onContextMenu={(e) =>
                            openStashMenu(
                              e,
                              stash.index,
                              stash.reference,
                              stash.message,
                            )
                          }
                          title={`${stash.message} · Botão direito: ações`}
                          className="w-full truncate rounded-md px-2 py-1 text-left text-xs hover:bg-surface"
                        >
                          <span className="font-mono text-[10px] text-muted">
                            {stash.reference}
                          </span>
                          <span className="ml-1 text-text">{stash.message}</span>
                        </button>
                      </li>
                    ))}
                  </ul>
                )}
              </CollapsibleSection>
            ) : null}
          </div>
        </div>
      )}

      {menu ? (
        <ContextMenu
          x={menu.x}
          y={menu.y}
          title={
            menu.kind === "local"
              ? menu.branch
              : menu.kind === "remote"
                ? `${menu.remote}/${menu.branch}`
                : menu.kind === "tag"
                  ? menu.name
                  : menu.reference
          }
          ariaLabel={
            menu.kind === "local"
              ? `Ações da branch ${menu.branch}`
              : menu.kind === "remote"
                ? `Ações da branch remota ${menu.remote}/${menu.branch}`
                : menu.kind === "tag"
                  ? `Ações da tag ${menu.name}`
                  : `Ações do stash ${menu.reference}`
          }
          items={menuItems}
          onClose={() => setMenu(null)}
        />
      ) : null}
    </div>
  );
}
