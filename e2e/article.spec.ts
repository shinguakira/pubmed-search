import { test, expect } from "@playwright/test";

test.describe("article detail (live NCBI)", () => {
  test("opening a result navigates to its detail page with an abstract", async ({
    page,
  }) => {
    await page.goto("/?q=crispr");
    await expect(page.getByTestId("result-count")).toContainText(/dispatches/i);

    // Click the title link of the first article.
    const firstTitle = page.locator("article a").first();
    await firstTitle.click();

    await expect(page).toHaveURL(/\/article\/\d+/);
    await expect(page.getByRole("heading", { name: "Abstract" })).toBeVisible();
  });

  test("Cite dialog opens and shows citation formats", async ({ page }) => {
    await page.goto("/?q=crispr");
    await expect(page.locator("article").first()).toBeVisible();

    await page.getByRole("button", { name: /^▸ Cite$/ }).first().click();

    await expect(
      page.getByRole("heading", { name: /Cite this article/i }),
    ).toBeVisible();
    await expect(page.getByRole("tab", { name: "AMA" })).toBeVisible();
    await expect(page.getByRole("tab", { name: "BibTeX" })).toBeVisible();
  });
});
