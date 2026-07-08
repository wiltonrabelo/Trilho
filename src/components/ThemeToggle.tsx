import { Coffee, Monitor, Moon, Sparkles, Sun } from "lucide-react";
import { useEffect, useState } from "react";
import {
  getStoredPreference,
  setPreference,
  type ThemePreference,
} from "@/lib/theme";

const OPTIONS: { value: ThemePreference; label: string; icon: typeof Sun }[] = [
  { value: "light", label: "Claro", icon: Sun },
  { value: "dark", label: "Escuro", icon: Moon },
  { value: "violet", label: "Violeta", icon: Sparkles },
  { value: "coffee", label: "Café", icon: Coffee },
  { value: "system", label: "Sistema", icon: Monitor },
];

export function ThemeToggle() {
  const [pref, setPref] = useState<ThemePreference>(getStoredPreference());

  useEffect(() => {
    setPreference(pref);
  }, [pref]);

  return (
    <div
      className="inline-flex items-center rounded-md border border-border bg-bg p-0.5"
      role="group"
      aria-label="Tema da interface"
    >
      {OPTIONS.map(({ value, label, icon: Icon }) => {
        const active = pref === value;
        return (
          <button
            key={value}
            type="button"
            aria-label={`Tema ${label}`}
            aria-pressed={active}
            title={label}
            onClick={() => setPref(value)}
            className={
              "rounded px-2 py-1 transition-colors " +
              (active
                ? "bg-accent text-white shadow-sm"
                : "text-muted hover:bg-surface hover:text-text")
            }
          >
            <Icon size={14} />
          </button>
        );
      })}
    </div>
  );
}
