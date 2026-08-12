import { useCallback, useMemo, useState } from "react";

import type { CommitContextMenuItem } from "@/components/CommitContextMenu";
import type { ContextMenuItem } from "@/components/ContextMenu";
import type {
  CommitFileContext,
  WorktreeFileContext,
} from "@/components/StatusPanel";
import {
  openWorktreePath,
  resolveWorktreePath,
  revealWorktreePath,
  runningInTauri,
} from "@/lib/api";
import type { CommitDto, WriteRequestDto } from "@/types";

interface FileMenuState {
  title: string;
  x: number;
  y: number;
  items: ContextMenuItem[];
}

interface CommitMenuState {
  commit: CommitDto;
  x: number;
  y: number;
}

interface ContextMenusParams {
  /** Estado que habilita/desabilita ações de escrita (detached HEAD). */
  writeDisabled: boolean;
  focusedBranch: string | null;
  headCommit: CommitDto | null;
  canAmend: boolean;
  upstreamConfigured: boolean;
  isDetached: boolean;
  requestWrite: (req: WriteRequestDto) => void;
  /** Passa o foco para o commit clicado antes de abrir o menu. */
  focusCommit: (commit: CommitDto) => void;
  selectFile: (path: string, staged: boolean) => void;
  selectCommitFile: (path: string) => Promise<void>;
  clearFileSelection: () => void;
  /** Abre o blame do arquivo de commit já selecionado. */
  openBlameOnCommitFile: () => void;
  openReset: () => void;
  openCherryPick: () => void;
  openTag: () => void;
  openReword: () => void;
  /** Edita a mensagem do HEAD local via amend no formulário de commit. */
  startAmend: () => void;
}

/**
 * Menus de contexto de commit e de arquivo (working tree e commit), com o
 * estado de posicionamento e a montagem condicional dos itens.
 */
export function useContextMenus({
  writeDisabled,
  focusedBranch,
  headCommit,
  canAmend,
  upstreamConfigured,
  isDetached,
  requestWrite,
  focusCommit,
  selectFile,
  selectCommitFile,
  clearFileSelection,
  openBlameOnCommitFile,
  openReset,
  openCherryPick,
  openTag,
  openReword,
  startAmend,
}: ContextMenusParams) {
  const [commitMenu, setCommitMenu] = useState<CommitMenuState | null>(null);
  const [fileMenu, setFileMenu] = useState<FileMenuState | null>(null);

  const closeCommitMenu = useCallback(() => setCommitMenu(null), []);
  const closeFileMenu = useCallback(() => setFileMenu(null), []);

  const handleCommitContextMenu = useCallback(
    (commit: CommitDto, clientX: number, clientY: number) => {
      focusCommit(commit);
      setFileMenu(null);
      setCommitMenu({ commit, x: clientX, y: clientY });
    },
    [focusCommit],
  );

  const copyPath = useCallback(async (path: string) => {
    try {
      const abs = runningInTauri() ? await resolveWorktreePath(path) : path;
      await navigator.clipboard.writeText(abs);
    } catch {
      /* clipboard pode falhar sem permissão */
    }
  }, []);

  const handleWorktreeFileContextMenu = useCallback(
    (ctx: WorktreeFileContext) => {
      setCommitMenu(null);
      const canOpen = ctx.kind !== "deleted";
      const items: ContextMenuItem[] = [
        {
          id: "view",
          label: "Ver diff / detalhes",
          onSelect: () => selectFile(ctx.path, ctx.section === "staged"),
        },
        {
          id: "open",
          label: "Abrir",
          disabled: !canOpen || !runningInTauri(),
          onSelect: () => void openWorktreePath(ctx.path).catch(() => undefined),
        },
        {
          id: "reveal",
          label: "Mostrar no Explorer",
          disabled: !runningInTauri(),
          onSelect: () =>
            void revealWorktreePath(ctx.path).catch(() => undefined),
        },
        {
          id: "copy",
          label: "Copiar caminho",
          onSelect: () => void copyPath(ctx.path),
        },
      ];

      if (ctx.section === "staged" && !writeDisabled && ctx.kind !== "conflicted") {
        items.push({
          id: "unstage",
          label: "Unstage",
          separatorBefore: true,
          onSelect: () => requestWrite({ kind: "unstage", path: ctx.path }),
        });
      }
      if (
        (ctx.section === "unstaged" || ctx.section === "untracked") &&
        !writeDisabled &&
        ctx.kind !== "conflicted"
      ) {
        items.push({
          id: "stage",
          label: ctx.section === "untracked" ? "Adicionar (stage)" : "Stage",
          separatorBefore: true,
          primary: true,
          onSelect: () => requestWrite({ kind: "stage", path: ctx.path }),
        });
      }
      if (
        ctx.section === "unstaged" &&
        !writeDisabled &&
        ctx.kind !== "conflicted"
      ) {
        items.push({
          id: "discard",
          label: "Descartar alterações",
          onSelect: () =>
            requestWrite({ kind: "discardWorktree", path: ctx.path }),
        });
      }
      if (ctx.section === "untracked" && !writeDisabled) {
        items.push({
          id: "remove",
          label: "Remover",
          onSelect: () =>
            requestWrite({ kind: "removeUntracked", path: ctx.path }),
        });
      }
      if (ctx.kind === "conflicted" && !writeDisabled) {
        items.push({
          id: "conflict",
          label: "Resolver conflito…",
          separatorBefore: true,
          primary: true,
          onSelect: () => selectFile(ctx.path, ctx.section === "staged"),
        });
      }

      setFileMenu({
        title: ctx.path,
        x: ctx.clientX,
        y: ctx.clientY,
        items,
      });
    },
    [copyPath, requestWrite, selectFile, writeDisabled],
  );

  const handleCommitFileContextMenu = useCallback(
    (ctx: CommitFileContext) => {
      setCommitMenu(null);
      const openCommitFile = async () => {
        clearFileSelection();
        await selectCommitFile(ctx.path);
      };
      // Arquivo já commitado: sem Abrir / Explorer (só working tree / stage).
      const items: ContextMenuItem[] = [
        {
          id: "view",
          label: "Ver diff",
          onSelect: () => void openCommitFile(),
        },
        {
          id: "blame",
          label: "Blame",
          disabled: ctx.kind === "deleted" || ctx.kind === "added",
          onSelect: () => {
            void openCommitFile().then(openBlameOnCommitFile);
          },
        },
        {
          id: "copy",
          label: "Copiar caminho",
          separatorBefore: true,
          onSelect: () => void copyPath(ctx.path),
        },
      ];
      setFileMenu({
        title: ctx.path,
        x: ctx.clientX,
        y: ctx.clientY,
        items,
      });
    },
    [clearFileSelection, copyPath, openBlameOnCommitFile, selectCommitFile],
  );

  const commitMenuItems = useMemo((): CommitContextMenuItem[] => {
    if (!commitMenu) return [];
    const c = commitMenu.commit;
    const isHead = Boolean(headCommit && c.id === headCommit.id);
    const items: CommitContextMenuItem[] = [];

    const showRevert =
      c.parentIds.length <= 1 && !writeDisabled && !focusedBranch;
    if (showRevert) {
      items.push({
        id: "revert",
        label: "Reverter commit",
        onSelect: () => requestWrite({ kind: "revert", commitId: c.id }),
      });
    }

    const showReset = headCommit && !isHead && !writeDisabled && !focusedBranch;
    if (showReset) {
      items.push({
        id: "reset",
        label: "Resetar para aqui…",
        onSelect: openReset,
      });
    }

    const showCherryPick =
      headCommit && !isHead && !writeDisabled && c.parentIds.length <= 1;
    if (showCherryPick) {
      items.push({
        id: "cherryPick",
        label: "Cherry-pick",
        onSelect: openCherryPick,
      });
    }

    items.push({
      id: "tag",
      label: "Criar tag…",
      onSelect: openTag,
    });

    const showEditHead = isHead && canAmend && !writeDisabled;
    const showReword =
      !writeDisabled &&
      !focusedBranch &&
      (c.isLocalOnly || upstreamConfigured) &&
      (!isHead || !c.isLocalOnly);
    if (showEditHead || showReword) {
      items.push({
        id: "editMessage",
        label: "Editar mensagem",
        primary: true,
        onSelect: () => {
          if (showEditHead) {
            startAmend();
          } else {
            openReword();
          }
        },
      });
    }

    const showUncommit =
      isHead && Boolean(headCommit?.isLocalOnly) && !writeDisabled && !isDetached;
    if (showUncommit) {
      items.push({
        id: "uncommit",
        label: "Uncommit (soft)",
        onSelect: () => requestWrite({ kind: "uncommit" }),
      });
    }

    return items;
  }, [
    commitMenu,
    headCommit,
    writeDisabled,
    focusedBranch,
    canAmend,
    upstreamConfigured,
    isDetached,
    requestWrite,
    openReset,
    openCherryPick,
    openTag,
    openReword,
    startAmend,
  ]);

  return {
    commitMenu,
    fileMenu,
    commitMenuItems,
    closeCommitMenu,
    closeFileMenu,
    handleCommitContextMenu,
    handleWorktreeFileContextMenu,
    handleCommitFileContextMenu,
    copyPath,
  };
}
