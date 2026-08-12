const SECTION_STORAGE_KEY = "trilho.refs.sections.v3";
const CHAVES_ANTIGAS = ["trilho.refs.sections.v2", "trilho.refs.sections.v1"];

export type SectionKey = "locals" | "remotes" | "tags" | "stashes";

export interface SectionState {
  locals: boolean;
  remotes: boolean;
  tags: boolean;
  stashes: boolean;
}

const TUDO_ABERTO: SectionState = {
  locals: true,
  remotes: true,
  tags: true,
  stashes: true,
};

/** Estado salvo das seções do painel de refs; tudo aberto por padrão. */
export function loadSectionState(): SectionState {
  try {
    const raw =
      localStorage.getItem(SECTION_STORAGE_KEY) ??
      CHAVES_ANTIGAS.map((k) => localStorage.getItem(k)).find(Boolean) ??
      null;
    if (!raw) return { ...TUDO_ABERTO };
    const parsed = JSON.parse(raw) as Partial<SectionState>;
    return {
      locals: parsed.locals !== false,
      remotes: parsed.remotes !== false,
      tags: parsed.tags !== false,
      stashes: parsed.stashes !== false,
    };
  } catch {
    return { ...TUDO_ABERTO };
  }
}

export function persistSectionState(state: SectionState) {
  localStorage.setItem(SECTION_STORAGE_KEY, JSON.stringify(state));
}
