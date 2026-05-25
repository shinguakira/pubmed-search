import { test, expect } from "@playwright/test";

test.describe("App filter — backend-side post-filter, Search-button trigger only", () => {
  test("typing in the keyword does NOT refetch; only Search press does", async ({ page }) => {
    await page.goto("/?q=crispr");

    // Wait for the initial unfiltered page to land.
    await expect(page.getByTestId("result-count")).toBeVisible({ timeout: 20_000 });
    const firstRow = page.locator("main button.grid").first();
    await expect(firstRow).toBeVisible({ timeout: 20_000 });

    // SearchBar auto-focuses on mount and opens the "Suggested queries"
    // hint panel, which would intercept clicks on the app-filter Select.
    // Click on the body to dismiss it.
    await page.locator("body").click({ position: { x: 5, y: 5 } });
    await expect(page.getByText("Suggested queries")).toHaveCount(0);

    const beforeRows = await page.locator("main button.grid").count();
    expect(beforeRows).toBeGreaterThan(10);

    // Type into the app-filter input — must NOT trigger a refetch.
    await page.getByTestId("app-filter-input").fill("review");

    // Open the Select and pick Include — also must NOT trigger.
    await page.getByTestId("app-filter-mode").click();
    await page.getByRole("option", { name: "Include", exact: true }).click();

    // Give React/Vite time to spuriously refetch if there were a bug.
    await page.waitForTimeout(800);

    // URL must still be the bare ?q=crispr, no app_filter params.
    expect(new URL(page.url()).searchParams.has("app_filter")).toBe(false);

    // Row count must be unchanged.
    const afterRows = await page.locator("main button.grid").count();
    expect(afterRows).toBe(beforeRows);

    // The "X / Y shown" badge appears only after Search press, not now.
    await expect(page.getByTestId("app-filter-badge")).toHaveCount(0);

    // Now press Search — this is the only legitimate trigger.
    await page.getByRole("button", { name: "Search", exact: true }).click();

    // Wait for the refetch to land: badge visible + rows re-rendered.
    await expect(page.getByTestId("app-filter-badge")).toBeVisible({ timeout: 20_000 });
    await expect(page.locator("main button.grid").first()).toBeVisible({ timeout: 20_000 });
    // Wait for the badge text to match the actual row count — otherwise
    // we can race between the badge appearing and the row list updating.
    await expect
      .poll(
        async () => {
          const txt = (await page.getByTestId("app-filter-badge").textContent()) ?? "";
          const m = /^(\d+)/.exec(txt.trim());
          const rows = await page.locator("main button.grid").count();
          return m && Number(m[1]) === rows;
        },
        { timeout: 20_000 },
      )
      .toBe(true);

    expect(new URL(page.url()).searchParams.get("app_filter")).toBe("review");
    expect(new URL(page.url()).searchParams.get("app_filter_mode")).toBe("include");

    // After Include filter, the visible row count must be strictly smaller.
    const filteredRows = await page.locator("main button.grid").count();
    expect(filteredRows).toBeLessThan(beforeRows);

    // The badge text agrees with the row count.
    const badgeText = (await page.getByTestId("app-filter-badge").textContent()) ?? "";
    const match = /^(\d+)\s*\/\s*(\d+)\s*shown/i.exec(badgeText.trim());
    expect(match).not.toBeNull();
    expect(Number(match![1])).toBe(filteredRows);
    expect(Number(match![2])).toBeGreaterThanOrEqual(filteredRows);

    // Defocus before screenshot so the hint dropdown doesn't cover anything.
    await page.locator("body").click({ position: { x: 5, y: 5 } });
    await page.screenshot({
      path: "../docs/screenshots/11-app-filter.png",
      fullPage: false,
    });
  });

  test("include + exclude on the same keyword partition the page slice", async ({ page }) => {
    await page.goto("/?q=crispr&app_filter=review&app_filter_mode=include");

    // Wait for the badge to render AND for the result rows to actually
    // populate — the badge becomes visible as soon as the response
    // resolves, but the row count we measure should be the final one.
    await expect(page.getByTestId("app-filter-badge")).toBeVisible({ timeout: 20_000 });
    await expect(page.locator("main button.grid").first()).toBeVisible({ timeout: 20_000 });
    // Dismiss the auto-focused search-bar hint dropdown.
    await page.locator("body").click({ position: { x: 5, y: 5 } });

    // Wait until the badge's "X" matches the actual row count.
    const waitBadgeMatchesRows = async () => {
      await expect
        .poll(
          async () => {
            const txt = (await page.getByTestId("app-filter-badge").textContent()) ?? "";
            const m = /^(\d+)/.exec(txt.trim());
            const rows = await page.locator("main button.grid").count();
            return m && Number(m[1]) === rows;
          },
          { timeout: 20_000 },
        )
        .toBe(true);
    };
    await waitBadgeMatchesRows();

    const includedRows = await page.locator("main button.grid").count();
    const includeBadge = (await page.getByTestId("app-filter-badge").textContent()) ?? "";
    const total = Number(/^\d+\s*\/\s*(\d+)/.exec(includeBadge.trim())?.[1] ?? "0");
    expect(total).toBeGreaterThan(0);

    // Flip to Exclude, re-press Search.
    await page.getByTestId("app-filter-mode").click();
    await page.getByRole("option", { name: "Exclude", exact: true }).click();
    await page.getByRole("button", { name: "Search", exact: true }).click();

    await expect(page.getByTestId("app-filter-badge")).toBeVisible({ timeout: 20_000 });
    await expect(page.locator("main button.grid").first()).toBeVisible({ timeout: 20_000 });
    await waitBadgeMatchesRows();
    const excludedRows = await page.locator("main button.grid").count();

    // Include + Exclude on the same keyword must sum to the unfiltered
    // page-slice size — anything else means the backend isn't applying
    // a consistent mask.
    expect(includedRows + excludedRows).toBe(total);
  });
});
