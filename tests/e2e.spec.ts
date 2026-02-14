import { test, expect } from '@playwright/test';

test.describe('MT2 Draft Assistant', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('http://localhost:5173');
  });

  test('should display the app title', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('MT2 Draft Assistant');
  });

  test('should have navigation tabs', async ({ page }) => {
    await expect(page.locator('button:has-text("Deck")')).toBeVisible();
    await expect(page.locator('button:has-text("Cards")')).toBeVisible();
    await expect(page.locator('button:has-text("Settings")')).toBeVisible();
  });

  test('should switch to Cards tab', async ({ page }) => {
    await page.click('button:has-text("Cards")');
    await expect(page.locator('input[placeholder="Search cards..."]')).toBeVisible();
  });

  test('should switch to Settings tab', async ({ page }) => {
    await page.click('button:has-text("Settings")');
    await expect(page.locator('text=OCR Mode')).toBeVisible();
    await expect(page.locator('text=Covenant Level')).toBeVisible();
  });

  test('Deck tab should show champion selector', async ({ page }) => {
    // Deck tab is active by default
    await expect(page.locator('text=Champion')).toBeVisible();
  });
});
