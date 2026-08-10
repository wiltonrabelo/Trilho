import { test, expect, type Page } from "@playwright/test";

/**
 * B-04 — limites de segurança na UI (modo web + mocks).
 * Contrato RF-08/A-02: preview com comando → confirmação humana.
 * Contratos IPC reais: testes Rust (write_auth, worktree, git_cli, security_contract).
 */
test.describe("Segurança RF-08 (mocks)", () => {
  async function openMockRepo(page: Page) {
    await page.goto("/");
    await page
      .getByRole("button", { name: /Abrir pasta de repositório Git/i })
      .click();
    await expect(page.getByText(/^Alterações\s*\(/)).toBeVisible({
      timeout: 15_000,
    });
  }

  async function expectRf08Dialog(page: Page) {
    const dialog = page.getByRole("dialog").filter({ hasText: /Comando Git/i });
    await expect(dialog).toBeVisible({ timeout: 10_000 });
    await expect(dialog.locator("pre")).toContainText(/git/i);
    await expect(dialog.getByRole("button", { name: "Confirmar" })).toBeEnabled();
    return dialog;
  }

  test("escrita exige diálogo com comando Git e pode ser cancelada", async ({
    page,
  }) => {
    await openMockRepo(page);
    await page.getByRole("button", { name: "Stage tudo" }).click();

    const dialog = await expectRf08Dialog(page);
    await dialog.getByRole("button", { name: "Cancelar" }).click();
    await expect(dialog).toHaveCount(0);
  });

  test("Escape fecha o preview sem executar", async ({ page }) => {
    await openMockRepo(page);
    await page.getByRole("button", { name: "Stage tudo" }).click();

    const dialog = await expectRf08Dialog(page);
    await page.keyboard.press("Escape");
    await expect(dialog).toHaveCount(0);
  });

  test("Confirmar fecha o preview após autorização mock", async ({ page }) => {
    await openMockRepo(page);
    await page.getByRole("button", { name: "Stage tudo" }).click();

    const dialog = await expectRf08Dialog(page);
    await dialog.getByRole("button", { name: "Confirmar" }).click();
    await expect(dialog).toHaveCount(0);
  });

  test("Publicar também passa por preview RF-08", async ({ page }) => {
    await openMockRepo(page);
    await page.getByRole("button", { name: /Publicar branch no remoto/i }).click();

    const publishForm = page.getByRole("dialog", { name: /Publicar branch/i });
    await expect(publishForm).toBeVisible();
    await publishForm
      .getByRole("textbox", { name: /URL do repositório remoto/i })
      .fill("https://github.com/example/demo.git");
    await publishForm.getByRole("button", { name: "Continuar" }).click();

    const dialog = await expectRf08Dialog(page);
    await expect(dialog.getByText(/Confirmar publicação/i)).toBeVisible();
    await dialog.getByRole("button", { name: "Cancelar" }).click();
    await expect(dialog).toHaveCount(0);
  });

  test("clone remoto também passa por preview RF-08", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("button", { name: /Clonar repositório remoto/i }).click();

    const cloneDialog = page.getByRole("dialog");
    await expect(
      cloneDialog.getByRole("heading", { name: /Clonar repositório/i }),
    ).toBeVisible();

    await cloneDialog
      .getByLabel(/URL do repositório/i)
      .fill("https://github.com/example/demo.git");
    await cloneDialog.getByRole("button", { name: /Escolher/i }).click();
    await expect(cloneDialog.getByLabel(/Nome da pasta/i)).not.toHaveValue("");

    await cloneDialog.getByRole("button", { name: /Continuar/i }).click();

    const dialog = await expectRf08Dialog(page);
    await expect(dialog.locator("pre")).toContainText(/clone/i);
    await dialog.getByRole("button", { name: "Cancelar" }).click();
  });
});
