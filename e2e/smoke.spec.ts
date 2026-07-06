import { test, expect } from "@playwright/test";

test.describe("Trilho web (mocks)", () => {
  test("exibe tela inicial com repo picker", async ({ page }) => {
    await page.goto("/");
    await expect(
      page.getByRole("button", { name: /Abrir pasta de repositório Git/i }),
    ).toBeVisible();
    await expect(page.getByRole("heading", { name: "Trilho" })).toBeVisible();
  });

  test("skip link para conteúdo principal", async ({ page }) => {
    await page.goto("/");
    await expect(
      page.getByRole("link", { name: /Ir para o conteúdo principal/i }),
    ).toBeAttached();
  });

  test("alterna tema sem quebrar layout", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("button", { name: "Tema Escuro" }).click();
    await expect(
      page.getByRole("button", { name: "Tema Escuro" }),
    ).toHaveAttribute("aria-pressed", "true");
  });
});
