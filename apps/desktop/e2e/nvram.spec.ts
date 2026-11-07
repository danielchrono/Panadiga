import { test, expect } from '@playwright/test';

test.describe('NVRAM & Spansion S29JL064H Surgical Recovery E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.click('button:has-text("Flash & NVRAM")');
    await expect(page.locator('header')).toContainText('Flash Spansion S29JL064H');
  });

  test('renders Spansion NOR Flash specifications and surgical action card', async ({ page }) => {
    await expect(page.getByText('Spansion S29JL064H NOR Flash')).toBeVisible();
    await expect(page.getByText('Status da NVRAM')).toBeVisible();
    await expect(page.getByText('Tamanho da Flash NOR')).toBeVisible();
  });

  test('displays microsector breakdown and telemetry metrics', async ({ page }) => {
    await expect(page.getByText('Topologia Física dos 71 Setores')).toBeVisible();
    await expect(page.getByText('Setores Corrompidos')).toBeVisible();
  });
});
