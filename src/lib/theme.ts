/**
 * Gerenciamento de tema (RF-17): Claro / Escuro / Seguir o sistema.
 * Preferência persistida em localStorage; aplica a classe `.dark` no <html>.
 */
export type ThemePreference = "light" | "dark" | "system";

const STORAGE_KEY = "trilho.theme";

export function getStoredPreference(): ThemePreference {
  const value = localStorage.getItem(STORAGE_KEY);
  if (value === "light" || value === "dark" || value === "system") {
    return value;
  }
  return "system";
}

function prefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

/** Resolve a preferência para o tema efetivo (light/dark). */
export function resolveTheme(pref: ThemePreference): "light" | "dark" {
  if (pref === "system") return prefersDark() ? "dark" : "light";
  return pref;
}

export function applyTheme(pref: ThemePreference): void {
  const effective = resolveTheme(pref);
  document.documentElement.classList.toggle("dark", effective === "dark");
}

export function setPreference(pref: ThemePreference): void {
  localStorage.setItem(STORAGE_KEY, pref);
  applyTheme(pref);
}

/**
 * Inicializa o tema e reage a mudanças do sistema quando a preferência é "system".
 * Retorna uma função de limpeza do listener.
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
