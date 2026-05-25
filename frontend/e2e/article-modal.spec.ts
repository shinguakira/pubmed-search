import { test, expect } from "@playwright/test";

const VARIANTS = [
  { label: "Envelope", file: "07-modal-envelope.png", settleMs: 2400 },
  { label: "Unfold", file: "08-modal-unfold.png", settleMs: 1800 },
  { label: "Drop", file: "09-modal-drop.png", settleMs: 1700 },
] as const;

test.describe("article modal — settled state per open animation", () => {
  for (const { label, file, settleMs } of VARIANTS) {
    test(`${label.toLowerCase()} variant settles with abstract + references visible`, async ({
      page,
    }) => {
      await page.goto("/?q=crispr");

      const firstResult = page.locator("main button.grid").first();
      await expect(firstResult).toBeVisible({ timeout: 20_000 });

      // Lock the ANIM switcher to the variant under test.
      await page
        .getByTestId("anim-switcher")
        .getByRole("button", { name: label, exact: true })
        .click();

      await firstResult.click();

      // Wait out the variant's reveal sequence and then capture.
      const drawer = page.getByTestId("article-drawer");
      await expect(drawer).toBeVisible();
      await page.waitForTimeout(settleMs);
      await expect(drawer.locator("h1")).toBeVisible();

      await page.screenshot({ path: `../docs/screenshots/${file}`, fullPage: false });

      await page.keyboard.press("Escape");
      await expect(page.getByTestId("article-modal-backdrop")).toHaveCount(0);
    });
  }
});

test("article modal with the references section scrolled into view", async ({ page }) => {
  await page.goto("/?q=crispr");

  const firstResult = page.locator("main button.grid").first();
  await expect(firstResult).toBeVisible({ timeout: 20_000 });

  // Force Unfold (no flap covering content) for the cleanest references shot.
  await page
    .getByTestId("anim-switcher")
    .getByRole("button", { name: "Unfold", exact: true })
    .click();

  await firstResult.click();

  const drawer = page.getByTestId("article-drawer");
  await expect(drawer).toBeVisible({ timeout: 10_000 });
  await page.waitForTimeout(2400);

  // Scroll inside the dialog to reveal the references panel.
  await drawer.evaluate((el) => el.scrollTo({ top: el.scrollHeight, behavior: "instant" }));
  await page.waitForTimeout(300);

  await page.screenshot({
    path: "../docs/screenshots/10-modal-references.png",
    fullPage: false,
  });
});
