import { chromium } from "@playwright/test";

const alvo = process.argv[2] ?? "http://127.0.0.1:1420/";
const larguras = process.argv[3]
  ? process.argv[3].split(",").map(Number)
  : [1024, 1366, 1920];

// Nome longo de verdade, como o `origin/dependabot/...` que estourou o topo.
const BRANCH_LONGA = "dependabot/cargo/src-tauri/tauri-plugin-dialog-2.7.2";

const browser = await chromium.launch();
for (const width of larguras) {
  const page = await browser.newPage({ viewport: { width, height: 768 } });
  await page.goto(alvo, { waitUntil: "networkidle" });
  const abrir = page.getByRole("button", { name: /abrir|repositório/i }).first();
  if (await abrir.count()) {
    await abrir.click().catch(() => {});
  }
  await page.waitForTimeout(1000);
  // Truncagem é CSS: basta o texto ser longo, não importa a origem dele.
  await page.evaluate((nome) => {
    const alvo = document.querySelector("header .truncate");
    if (alvo) alvo.textContent = nome;
  }, BRANCH_LONGA);
  await page.waitForTimeout(200);
  const altura = await page.evaluate(
    () => document.querySelector("header")?.getBoundingClientRect().height ?? 0,
  );
  console.log(`${width}px → header ${altura}px`);
  await page.screenshot({
    path: `scripts/header-${width}.png`,
    clip: { x: 0, y: 0, width, height: 120 },
  });
  await page.close();
}
await browser.close();
