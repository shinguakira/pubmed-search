import { test, expect } from "@playwright/test";

test.describe("search (live NCBI)", () => {
  test("URL ?q=… returns results, shows elapsed time", async ({ page }) => {
    await page.goto("/?q=crispr");

    const count = page.getByTestId("result-count");
    await expect(count).toContainText(/dispatches/i);
    // Should be a sizeable corpus.
    await expect(count).toContainText(/\d{3,}/);

    // elapsed_ms rendered as "X.XXs"
    await expect(page.getByTestId("elapsed-time")).toHaveText(/\d+\.\d{2}s/);

    // At least one article row rendered.
    await expect(page.locator("article").first()).toBeVisible();
  });

  test("typing into the search bar updates results", async ({ page }) => {
    await page.goto("/");
    const input = page.getByPlaceholder(/Inquire of the archive/i);
    await input.fill("long covid");
    await page.getByRole("button", { name: /^Search$/ }).click();

    await expect(page).toHaveURL(/\?q=long\+covid/);
    await expect(page.getByTestId("result-count")).toContainText(/dispatches/i);
  });

  test("applying a filter narrows results", async ({ page }) => {
    await page.goto("/?q=crispr");
    await expect(page.getByTestId("result-count")).toContainText(/dispatches/i);
    const before = await readCount(page);

    await page.getByLabel("Review", { exact: true }).check();
    await expect(page.getByTestId("result-count")).toContainText(/dispatches/i);
    // After filter the count should change (usually drop).
    await expect
      .poll(() => readCount(page), { timeout: 15_000 })
      .not.toBe(before);
  });
});

async function readCount(page: import("@playwright/test").Page): Promise<number> {
  const txt = await page.getByTestId("result-count").innerText();
  const m = txt.match(/[\d,]+/);
  return m ? Number(m[0].replace(/,/g, "")) : 0;
}
