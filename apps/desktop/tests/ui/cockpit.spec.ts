import { expect, test, type Page } from "@playwright/test";

const views = [
  { title: "Overview", heading: "Capability overview" },
  { title: "Capabilities", heading: "Capabilities" },
  { title: "Planning", heading: "Planning" },
  { title: "Execution", heading: "Execution" },
  { title: "Federation", heading: "Federation" },
  { title: "Evidence", heading: "Evidence" },
];

async function loadEnglish(page: Page) {
  await page.addInitScript(() => {
    localStorage.setItem("poie-lang", "en");
    localStorage.setItem("poie-theme", "light");
  });
  await page.goto("/");
  await expect(page.locator(".cockpit")).toBeVisible();
}

test.describe("Poieweave cockpit", () => {
  test("loads and navigates all six views", async ({ page }, testInfo) => {
    const consoleErrors: string[] = [];
    page.on("pageerror", (error) => consoleErrors.push(error.message));
    await loadEnglish(page);
    for (const view of views) {
      await page.getByRole("button", { name: view.title, exact: true }).click();
      await expect(page.getByRole("heading", { level: 1 })).toHaveText(view.heading);
      await testInfo.attach(`view-${view.title.toLowerCase()}`, {
        body: await page.screenshot(),
        contentType: "image/png",
      });
    }
    expect(consoleErrors).toEqual([]);
  });

  test("keeps the layout and visual tokens stable", async ({ page }) => {
    await loadEnglish(page);
    const layout = await page.locator(".cockpit").evaluate((element) => {
      const style = getComputedStyle(element);
      const root = getComputedStyle(document.documentElement);
      return {
        columns: style.gridTemplateColumns.split(" ").length,
        pane: root.getPropertyValue("--pane").trim(),
        sidebar: root.getPropertyValue("--sidebar").trim(),
        theme: document.documentElement.dataset.theme,
      };
    });
    expect(layout.columns).toBe(4);
    expect(layout.theme).toBe("light");
    expect(layout.pane).toBe("#f7f7f8");
    expect(layout.sidebar).toBe("#e8e8ea");

    await page.locator('.rail button[aria-label="Toggle theme"]').click();
    const dark = await page.evaluate(() => ({
      theme: document.documentElement.dataset.theme,
      pane: getComputedStyle(document.documentElement).getPropertyValue("--pane").trim(),
    }));
    expect(dark.theme).toBe("dark");
    expect(dark.pane).toBe("#1c1c1c");
  });

  test("switches language without leaving missing i18n keys", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("poie-lang", "en");
      localStorage.setItem("poie-theme", "light");
    });
    await page.goto("/");
    await page.locator('.rail button[aria-label="Toggle language"]').click();
    await expect(page.locator("html")).toHaveAttribute("lang", "zh-CN");
    await expect(page.getByRole("button", { name: "总览", exact: true })).toBeVisible();
    const text = await page.locator("body").innerText();
    expect(text).not.toContain("undefined");
    expect(text).not.toMatch(/(^|\s)(nav|settings|agent|execution|federation)\.[a-z]/);
  });

  test("opens settings and keeps auth/inference behind the host", async ({ page }) => {
    await loadEnglish(page);
    await page.locator('.rail button[aria-label="Settings"]').click();
    const dialog = page.getByRole("dialog", { name: "Settings" });
    await expect(dialog).toBeVisible();
    await expect(dialog.getByText("Local operator authentication runs in the Tauri host only.")).toBeVisible();
    await expect(dialog.getByText("Remote inference is enabled only in the Tauri host.")).toBeVisible();
    await dialog.getByRole("button", { name: "Close" }).click();
    await expect(dialog).toBeHidden();
  });

  test("renders the agent composer in the inspector", async ({ page }) => {
    await loadEnglish(page);
    const agent = page.locator(".agent-panel");
    await expect(agent).toBeVisible();
    await expect(agent.getByText("Browser preview cannot call host inference.")).toBeVisible();
    await expect(agent.getByRole("textbox")).toBeDisabled();
    await expect(agent.getByLabel("Tool profile")).toBeDisabled();
  });

  test("shows the M9 bench card with host-only controls disabled", async ({ page }) => {
    await loadEnglish(page);
    await page.getByRole("button", { name: "Execution", exact: true }).click();
    await expect(page.getByText("M9 bench drills")).toBeVisible();
    await expect(page.getByRole("button", { name: "B6", exact: true })).toBeDisabled();
    await expect(page.getByLabel("I confirmed the bench is safe (simulated)")).toBeDisabled();
    await expect(page.getByRole("button", { name: "Clear unknown/fallback" })).toBeDisabled();
  });

  test("focused window mode hides the full cockpit chrome", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("poie-lang", "en");
      localStorage.setItem("poie-theme", "light");
    });
    await page.goto("/?view=federation&focus=1");
    await expect(page.locator(".cockpit--focus")).toBeVisible();
    await expect(page.getByRole("heading", { level: 1 })).toHaveText("Federation");
    await expect(page.locator(".rail")).toBeHidden();
    await expect(page.locator(".sidebar")).toBeHidden();
    await expect(page.locator(".inspector")).toBeHidden();
  });

  test("gives every icon control an accessible name", async ({ page }) => {
    await loadEnglish(page);
    const unnamed = await page.locator("button.icon-button, button.rail__button").evaluateAll((buttons) =>
      buttons.filter((button) => {
        const label = button.getAttribute("aria-label") ?? button.getAttribute("title") ?? "";
        return label.trim().length === 0;
      }).length,
    );
    expect(unnamed).toBe(0);
    await page.getByRole("button", { name: "Execution", exact: true }).click();
    await expect(page.locator(".gate-card")).toBeVisible();
  });
});
