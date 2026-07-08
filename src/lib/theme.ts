/**
 * Gerenciamento de tema (RF-17): Claro / Escuro / Violeta / Café / Sistema.
 * Claro e escuro usam paleta neutra + acento azul (estilo GitHub Desktop).
 */
export type ThemePreference = "light" | "dark" | "system" | "coffee" | "violet";
export type EffectiveTheme = "light" | "dark" | "coffee" | "violet";

const STORAGE_KEY = "trilho.theme";

/** Tema padrão em instalações novas (sem preferência salva). */
const DEFAULT_PREFERENCE: ThemePreference = "light";

export function getStoredPreference(): ThemePreference {
  const value = localStorage.getItem(STORAGE_KEY);
  if (
    value === "light" ||
    value === "dark" ||
    value === "system" ||
    value === "coffee" ||
    value === "violet"
  ) {
    return value;
  }
  return DEFAULT_PREFERENCE;
}

function prefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

/** Resolve a preferência para o tema efetivo. */
export function resolveTheme(pref: ThemePreference): EffectiveTheme {
  if (pref === "system") return prefersDark() ? "dark" : "light";
  return pref;
}

export function applyTheme(pref: ThemePreference): void {
  const effective = resolveTheme(pref);
  const root = document.documentElement;
  root.classList.toggle("dark", effective === "dark" || effective === "coffee");
  root.classList.toggle("coffee", effective === "coffee");
  root.classList.toggle("violet", effective === "violet");
}

export function setPreference(pref: ThemePreference): void {
  localStorage.setItem(STORAGE_KEY, pref);
  applyTheme(pref);
}

/**
 * Inicializa o tema e reage a mudanças do sistema quando a preferência é "system".
 */
export function initTheme(): () => void {
  applyTheme(getStoredPreference());

  const media = window.matchMedia("(prefers-color-scheme: dark)");
  const onChange = () => {
    if (getStoredPreference() === "system") {
      applyTheme("system");
    }
  };
  media.addEventListener("change", onChange);
  return () => media.removeEventListener("change", onChange);
}
