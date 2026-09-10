import { expect, test } from "@playwright/test";

// Adapt selectors, launch fixtures, browser matrix, ceilings, and action names
// to the product. This file is a qualification shape, not a browser-only mock.

test("public companion applies one native action and reconciles", async ({ page }) => {
  const companionUrl = process.env.REMOTE_COMPANION_URL;
  const password = process.env.REMOTE_TEST_PASSWORD;
  if (!companionUrl || !password) test.skip(true, "Set isolated qualification credentials and URL");

  await page.goto(companionUrl as string);
  await expect(page.getByTestId("target-status")).toHaveText("Discoverable", { timeout: 15_000 });
  await page.getByTestId("connect-control").click();
  await page.getByLabel("Password").fill(password as string);
  await page.getByRole("button", { name: "Connect" }).click();

  await expect(page.getByTestId("control-status")).toHaveText("Ready", { timeout: 15_000 });
  const before = Number(await page.getByTestId("revision").textContent());
  const started = performance.now();
  await page.getByRole("button", { name: "Activate" }).click();
  await expect(page.getByTestId("native-state")).toHaveText("Active", { timeout: 2_000 });
  const acknowledgementMs = performance.now() - started;
  expect(acknowledgementMs).toBeLessThan(2_000);
  await expect(page.getByTestId("revision")).not.toHaveText(String(before));

  // Add a native-side assertion through the project's test fixture or visible
  // installed app. A changed browser label alone is not authority evidence.
});
