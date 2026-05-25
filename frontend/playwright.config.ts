import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright config for the PubMed-search frontend.
 *
 * Pre-conditions:
 *   * `npm run dev` (Vite) is running on http://localhost:5173
 *   * The backend (cargo run -p backend) is running on http://localhost:8787
 *
 * The tests hit live NCBI through the backend, so they're not hermetic;
 * they're meant for taking screenshots / smoke-checking the UI rather
 * than CI gating. `webServer` is intentionally omitted so the user
 * keeps full control over server lifecycles.
 */
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: "http://localhost:5173",
    trace: "on-first-retry",
    video: "retain-on-failure",
    screenshot: "only-on-failure",
    viewport: { width: 1440, height: 900 },
    actionTimeout: 10_000,
    navigationTimeout: 15_000,
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
