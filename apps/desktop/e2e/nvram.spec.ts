import { test, expect } from '@playwright/test';
import { installMockIpcBridge } from './fixtures/mockIpcBridge';

test.describe('NVRAM & Spansion S29JL064H Surgical Recovery E2E', () => {
  test.beforeEach(async ({ page }) => {
    await installMockIpcBridge(page);
    await page.goto('/');
    await page.click('button:has-text("Flash & NVRAM")');
    await expect(page.locator('header')).toContainText('Flash Spansion S29JL064H');
  });

  test('renders Spansion NOR Flash specifications and surgical action card', async ({ page }) => {
    await expect(page.getByText('Spansion S29JL064H NOR Flash')).toBeVisible();
    await expect(page.getByText('Status da NVRAM')).toBeVisible();
    await expect(page.getByText('Tamanho da Flash NOR')).toBeVisible();
    await expect(page.getByRole('button', { name: /Capturar NVRAM via Pico/i })).toBeVisible();
  });

  test('displays microsector breakdown and telemetry metrics', async ({ page }) => {
    await expect(page.getByText('Topologia Física dos 71 Setores')).toBeVisible();
    await expect(page.getByText('Setores Corrompidos')).toBeVisible();
  });

  test('streams live Pico NVRAM capture into Sector Map and toggles back to disk dump', async ({ page }) => {
    // 1. Disconnected guard: clicking capture without active Pico reports explicit requirement (no silent mock)
    await page.getByRole('button', { name: /Capturar NVRAM via Pico/i }).click();
    await expect(page.getByText(/Nenhuma sonda Pico conectada/i)).toBeVisible();

    // 2. Navigate to PicoBench, trigger real DUMP command, and verify reactive NVRAM Sector Map sync
    await page.click('button:has-text("Bancada PicoBench")');
    await page.click('[role="tab"]:has-text("OCD & NOR Flash")');
    await page.click('button:has-text("Executar DUMP")');
    await expect(page.getByRole('cell', { name: '0x807F0000' }).first()).toBeVisible();

    await page.click('button:has-text("Flash & NVRAM")');
    await expect(page.getByRole('button', { name: /Voltar ao Dump em Disco/i })).toBeVisible();
    await page.getByRole('button', { name: /Voltar ao Dump em Disco/i }).click();
    await expect(page.getByRole('button', { name: /Voltar ao Dump em Disco/i })).toHaveCount(0);
  });
});
