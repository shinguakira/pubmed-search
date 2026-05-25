import { test, expect } from "@playwright/test";

test("home page renders the search bar, filters sidebar and ANIM switcher", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByPlaceholder("Search PubMed…")).toBeVisible();
  await expect(page.getByRole("button", { name: "Search" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Filters" })).toBeVisible();
  await expect(page.getByTestId("anim-switcher")).toBeVisible();

  // Defocus the search bar so the hint dropdown doesn't show.
  await page.locator("body").click({ position: { x: 5, y: 5 } });
  await page.screenshot({ path: "../docs/screenshots/01-overview.png", fullPage: false });
});
