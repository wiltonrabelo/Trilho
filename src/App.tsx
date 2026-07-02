import { GitCommitHorizontal, TrainFront } from "lucide-react";
import { useEffect, useState } from "react";
import { ThemeToggle } from "@/components/ThemeToggle";
import { getAppInfo, listCommitsMock, runningInTauri } from "@/lib/api";
import type { AppInfo, CommitDto } from "@/types";

function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [commits, setCommits] = useState<CommitDto[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [webOnly, setWebOnly] = useState(false);

  useEffect(() => {
    setWebOnly(!runningInTauri());

    Promise.all([getAppInfo(), listCommitsMock()])
      .then(([appInfo, list]) => {
        setInfo(appInfo);
        setCommits(list);
      })
      .catch((e) => setError(String(e)));
  }, []);

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-border bg-surface px-5 py-3">
        <div className="flex items-center gap-2.5">
          <TrainFront className="text-accent" size={22} />
          <div className="flex items-baseline gap-2">
            <h1 className="text-lg font-semibold tracking-tight">Trilho</h1>
            <span className="text-xs text-muted">
              {info ? `v${info.version}` : "…"}
            </span>
          </div>
        </div>
        <ThemeToggle />
      </header>

      <main className="flex-1 overflow-auto p-5">
        <section className="mx-auto max-w-3xl">
          {webOnly && (
            <div className="mb-4 rounded-lg border border-amber-500/40 bg-amber-500/10 px-4 py-3 text-sm text-text">
              <strong>Modo navegador</strong> — sem backend Rust. Os dados abaixo
              são de exemplo. Para o app completo, feche esta aba e rode{" "}
              <code className="rounded bg-surface px-1 font-mono text-xs">
                npm run dev
              </code>{" "}
              em <code className="font-mono text-xs">C:\Projetos\Trilho</code>{" "}
              (abre a janela desktop <strong>Trilho.exe</strong>).
            </div>
          )}

          <div className="mb-4 flex items-center gap-2 text-sm text-muted">
            <GitCommitHorizontal size={16} />
            <span>
              Trilha de commits
              {webOnly ? " (dados locais — M0)" : " (dados de exemplo — M0)"}
            </span>
          </div>

          {error && (
            <div className="rounded-lg border border-border bg-surface p-4 text-sm text-red-500">
              Falha ao falar com o backend: {error}
            </div>
          )}
          <ol className="relative space-y-3">
            {commits.map((c) => (
              <li
                key={c.id}
                className="flex items-start gap-3 rounded-lg border border-border bg-surface px-4 py-3"
              >
                <span className="mt-1 h-2.5 w-2.5 shrink-0 rounded-full bg-accent" />
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <p className="truncate text-sm font-medium">{c.summary}</p>
                    {c.isLocalOnly && (
                      <span className="rounded bg-accent/15 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-accent">
                        local
                      </span>
                    )}
                  </div>
                  <p className="mt-0.5 text-xs text-muted">
                    <span className="font-mono">{c.shortId}</span> ·{" "}
                    {c.authorName} ·{" "}
                    {new Date(c.authoredAt).toLocaleString("pt-BR")}
                  </p>
                </div>
              </li>
            ))}
          </ol>
        </section>
      </main>
    </div>
  );
}

export default App;
