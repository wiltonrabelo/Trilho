import { Monitor, Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";
import {
  getStoredPreference,
  setPreference,
  type ThemePreference,
} from "@/lib/theme";

const OPTIONS: { value: ThemePreference; label: string; icon: typeof Sun }[] = [
  { value: "light", label: "Claro", icon: Sun },
  { value: "dark", label: "Escuro", icon: Moon },
  { value: "system", label: "Sistema", icon: Monitor },
];

export function ThemeToggle() {
  const [pref, setPref] = useState<ThemePreference>(getStoredPreference());

  useEffect(() => {
    setPreference(pref);
  }, [pref]);

  return (
    <div className="inline-flex items-center gap-1 rounded-lg border border-border bg-surface p-1">
      {OPTIONS.map(({ value, label, icon: Icon }) => {
        const active = pref === value;
        return (
          <button
            key={value}
            type="button"
            aria-label={`Tema ${label}`}
            aria-pressed={active}
            onClick={() => setPref(value)}
            className={
              "flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium transition-colors " +
              (active
                ? "bg-accent text-white"
                : "text-muted hover:text-text hover:bg-border/40")
            }
          >
            <Icon size={14} />
            <span>{label}</span>
          </button>
        );
      })}
    </div>
  );
}
