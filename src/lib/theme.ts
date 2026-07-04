/**
 * Gerenciamento de tema (RF-17): Claro / Escuro / Café / Seguir o sistema.
 * Preferência persistida em localStorage; aplica as classes de tema no <html>.
 * "Café" é um tema escuro quente: carrega `.dark` (pras variantes `dark:` do
 * Tailwind) e `.coffee` (que sobrescreve os tokens com tons de café).
 */
export type ThemePreference = "light" | "dark" | "system" | "coffee";
export type EffectiveTheme = "light" | "dark" | "coffee";

const STORAGE_KEY = "trilho.theme";

export function getStoredPreference(): ThemePreference {
  const value = localStorage.getItem(STORAGE_KEY);
  if (
    value === "light" ||
    value === "dark" ||
    value === "system" ||
    value === "coffee"
  ) {
    return value;
  }
  return "system";
}

function prefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

/** Resolve a preferência para o tema efetivo (light/dark/coffee). */
export function resolveTheme(pref: ThemePreference): EffectiveTheme {
  if (pref === "system") return prefersDark() ? "dark" : "light";
  return pref;
}

export function applyTheme(pref: ThemePreference): void {
  const effective = resolveTheme(pref);
  const root = document.documentElement;
  // Café herda o esquema escuro (variantes `dark:` + scrollbars) e adiciona
  // seus próprios tokens por cima via `.coffee`.
  root.classList.toggle("dark", effective === "dark" || effective === "coffee");
  root.classList.toggle("coffee", effective === "coffee");
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
