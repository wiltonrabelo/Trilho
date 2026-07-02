import { afterEach, describe, expect, it, vi } from "vitest";
import { resolveTheme, type ThemePreference } from "@/lib/theme";

function mockMatchMedia(dark: boolean) {
  vi.stubGlobal(
    "matchMedia",
    vi.fn().mockImplementation((query: string) => ({
      matches: query.includes("(prefers-color-scheme: dark)") && dark,
      media: query,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  );
}

describe("resolveTheme", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("retorna light quando preferência é light", () => {
    expect(resolveTheme("light")).toBe("light");
  });

  it("retorna dark quando preferência é dark", () => {
    expect(resolveTheme("dark")).toBe("dark");
  });

  it("system delega ao prefers-color-scheme (dark)", () => {
    mockMatchMedia(true);
    const pref: ThemePreference = "system";
    expect(resolveTheme(pref)).toBe("dark");
  });

  it("system delega ao prefers-color-scheme (light)", () => {
    mockMatchMedia(false);
    const pref: ThemePreference = "system";
    expect(resolveTheme(pref)).toBe("light");
  });
});
