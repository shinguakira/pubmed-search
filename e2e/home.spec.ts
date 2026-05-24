import { test, expect } from "@playwright/test";

test.describe("home / empty state", () => {
  test("renders masthead and suggested topic chips", async ({ page }) => {
    await page.goto("/");

    await expect(page.getByRole("link", { name: /The PubMed Gazette/i })).toBeVisible();
    await expect(page.getByText("Today's Edition")).toBeVisible();
    await expect(
      page.getByRole("heading", {
        name: /Thirty million letters from the biomedical archive/i,
      }),
    ).toBeVisible();
    await expect(page.getByRole("button", { name: /CRISPR Cas9/i })).toBeVisible();
  });

  test("clicking a suggested chip runs that search", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("button", { name: /CRISPR Cas9/i }).click();

    await expect(page).toHaveURL(/\?q=CRISPR\+Cas9/);
    await expect(page.getByTestId("result-count")).toContainText(/dispatches/i);
  });
});
