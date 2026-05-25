import { test, expect } from "@playwright/test";

test("search bar shows the suggested-query hint dropdown when focused", async ({ page }) => {
  await page.goto("/");
  await page.getByPlaceholder("Search PubMed…").click();
  await expect(page.getByText("Suggested queries")).toBeVisible();

  await page.screenshot({ path: "../docs/screenshots/02-search-hints.png", fullPage: false });
});

test("search for 'crispr' returns a list of results with the result count badge", async ({
  page,
}) => {
  await page.goto("/");

  await page.getByPlaceholder("Search PubMed…").fill("crispr");
  await page.getByRole("button", { name: "Search" }).click();

  // NCBI round-trip takes a few seconds; allow up to 20s.
  await expect(page.getByTestId("result-count")).toBeVisible({ timeout: 20_000 });
  await expect(page.locator("main button.grid").first()).toBeVisible();

  // Defocus so the hint dropdown doesn't cover results.
  await page.locator("body").click({ position: { x: 5, y: 5 } });
  await page.screenshot({ path: "../docs/screenshots/03-search-results.png", fullPage: false });
});
