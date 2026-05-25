import { test, expect } from "@playwright/test";

/**
 * Capture the secondary controls — filters sidebar, sort dropdown,
 * fetch-mode toggle — for the user-manual screenshots. Each test
 * starts from a populated `crispr` result page so the screenshot
 * has real data behind the chrome being demonstrated.
 */

async function searchCrispr(page: import("@playwright/test").Page) {
  await page.goto("/?q=crispr");
  await expect(page.getByTestId("result-count")).toBeVisible({ timeout: 20_000 });
  await expect(page.locator("main button.grid").first()).toBeVisible({ timeout: 20_000 });
  // Defocus the search bar so the hint dropdown does not cover content.
  await page.locator("body").click({ position: { x: 5, y: 5 } });
}

test("applying the Review filter triggers a refetch and a narrowed list", async ({ page }) => {
  await searchCrispr(page);

  // Check the Review article-type filter and re-submit.
  await page.getByLabel("Review", { exact: true }).check();
  await page.getByRole("button", { name: "Search", exact: true }).click();
  await expect(page.getByTestId("result-count")).toBeVisible({ timeout: 20_000 });

  await page.locator("body").click({ position: { x: 5, y: 5 } });
  await page.screenshot({ path: "../docs/screenshots/04-filter-applied.png", fullPage: false });
});

test("the Sort dropdown reveals the available sort orders", async ({ page }) => {
  await searchCrispr(page);

  // Open the Sort select (Radix renders a portal with role=listbox).
  await page.locator("section").locator('button[role="combobox"]').first().click();
  await expect(page.getByRole("option", { name: "Best match" })).toBeVisible();

  await page.screenshot({ path: "../docs/screenshots/05-sort-open.png", fullPage: false });
});

test("the fetch-mode toggle highlights Bulk in rust when selected", async ({ page }) => {
  await searchCrispr(page);

  await page.getByRole("button", { name: "Bulk", exact: true }).click();
  // The rust highlight is a class change, no async work — small wait so
  // the screenshot picks up the new colour reliably.
  await page.waitForTimeout(100);

  await page.screenshot({ path: "../docs/screenshots/06-fetch-mode.png", fullPage: false });
});
