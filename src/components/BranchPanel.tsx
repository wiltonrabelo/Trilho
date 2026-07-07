import { Cloud, GitBranch } from "lucide-react";

import type { RemoteBranchRefDto } from "@/types";

interface BranchPanelProps {
  branches: string[];
  remoteBranches: RemoteBranchRefDto[];
  currentBranch?: string | null;
  loading?: boolean;
  onSwitchLocal: (branch: string) => void;
  onSwitchRemote: (remote: string, branch: string) => void;
}

export function BranchPanel({
  branches,
  remoteBranches,
  currentBranch,
  loading,
  onSwitchLocal,
  onSwitchRemote,
}: BranchPanelProps) {
  const remoteOnly = remoteBranches.filter(
    (ref) => !branches.includes(ref.branch),
  );

  if (branches.length === 0 && remoteOnly.length === 0 && !loading) {
    return null;
  }

  return (
    <div className="border-b border-border px-3 pb-3">
      <div className="mb-1.5 flex items-center gap-1.5 text-[11px] font-medium uppercase tracking-wide text-muted">
        <GitBranch size={12} />
        Ramos
      </div>
      {loading && branches.length === 0 ? (
        <p className="mb-2 text-xs text-muted">Carregando…</p>
      ) : (
        <ul className="mb-3 flex max-h-32 flex-col gap-0.5 overflow-y-auto">
          {branches.map((branch) => {
            const active = branch === currentBranch;
            return (
              <li key={branch}>
                <button
                  type="button"
                  disabled={active}
                  onClick={() => onSwitchLocal(branch)}
                  title={active ? "Branch atual" : `Trocar para ${branch}`}
                  className={`w-full truncate rounded-md px-2 py-1 text-left text-xs ${
                    active
                      ? "bg-accent/15 font-medium text-accent"
                      : "text-text hover:bg-surface"
                  } disabled:cursor-default`}
                >
                  {branch}
                  {active ? " ✓" : ""}
                </button>
              </li>
            );
          })}
        </ul>
      )}

      {remoteBranches.length > 0 && (
        <>
          <div className="mb-1.5 flex items-center gap-1.5 text-[11px] font-medium uppercase tracking-wide text-muted">
            <Cloud size={12} />
            Remotos
          </div>
          <ul className="flex max-h-32 flex-col gap-0.5 overflow-y-auto">
            {remoteBranches.map((ref) => {
              const label = `${ref.remote}/${ref.branch}`;
              const active = ref.branch === currentBranch;
              const hasLocal = branches.includes(ref.branch);
              return (
                <li key={label}>
                  <button
                    type="button"
                    disabled={active}
                    onClick={() =>
                      hasLocal
                        ? onSwitchLocal(ref.branch)
                        : onSwitchRemote(ref.remote, ref.branch)
                    }
                    title={
                      active
                        ? "Branch atual"
                        : hasLocal
                          ? `Trocar para ${ref.branch} (local)`
                          : `Criar e rastrear ${label}`
                    }
                    className={`w-full truncate rounded-md px-2 py-1 text-left text-xs ${
                      active
                        ? "bg-accent/15 font-medium text-accent"
                        : "text-muted hover:bg-surface hover:text-text"
                    } disabled:cursor-default`}
                  >
                    {label}
                    {!hasLocal ? " ↓" : ""}
                    {active ? " ✓" : ""}
                  </button>
                </li>
              );
            })}
          </ul>
        </>
      )}
    </div>
  );
}
