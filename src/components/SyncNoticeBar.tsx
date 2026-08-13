import { syncNotices, type SyncNoticeInput } from "@/lib/syncNotices";

const CLASSE_POR_SEVERIDADE = {
  warning:
    "border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-300",
  error: "border-red-500/40 bg-red-500/10 text-red-700 dark:text-red-300",
} as const;

/**
 * Faixa de avisos logo abaixo do cabeçalho. Cada aviso ocupa a largura toda,
 * em vez de disputar espaço com os botões de sincronização.
 */
export function SyncNoticeBar(props: SyncNoticeInput) {
  const notices = syncNotices(props);
  if (notices.length === 0) return null;

  return (
    <div role="status" aria-label="Avisos de sincronização">
      {notices.map((notice) => (
        <div
          key={notice.id}
          className={`border-b px-5 py-1.5 text-xs ${CLASSE_POR_SEVERIDADE[notice.severity]}`}
        >
          {notice.message}
        </div>
      ))}
    </div>
  );
}
