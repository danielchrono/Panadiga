import { test, expect } from '@playwright/test';
import { installMockIpcBridge } from './fixtures/mockIpcBridge';

test.describe('Panadiga Desktop Navigation & Layout E2E', () => {
  test.beforeEach(async ({ page }) => {
    await installMockIpcBridge(page);
    await page.goto('/');
    await expect(page.locator('h1')).toContainText('DIGA SUITE');
    await expect(page.getByText(/Modo Demo|Demonstração/i)).toHaveCount(0);
  });

  test('loads Dashboard View by default with system metrics', async ({ page }) => {
    await expect(page.locator('header')).toContainText('Visão Geral do Gravador');
    await expect(page.getByText('Spansion NOR Flash').first()).toBeVisible();
    await expect(page.getByText(/Head Dump/).first()).toBeVisible();
  });

  test('navigates seamlessly across all workbench modules', async ({ page }) => {
    // 1. Flash & NVRAM Module
    await page.click('button:has-text("Flash & NVRAM")');
    await expect(page.locator('header')).toContainText('Flash Spansion S29JL064H');
    await expect(page.getByText(/Cirurgia|Surgical/).first()).toBeVisible();

    // 2. Armazenamento HD Module
    await page.click('button:has-text("Armazenamento HD")');
    await expect(page.locator('header')).toContainText('Armazenamento MEIHDFS');

    // 3. Bancada PicoBench Module
    await page.click('button:has-text("Bancada PicoBench")');
    await expect(page.locator('header')).toContainText('Bancada Raspberry Pi Pico');

    // 4. Criptoanálise FRM Module
    await page.click('button:has-text("Criptoanálise FRM")');
    await expect(page.locator('header')).toContainText('Criptoanálise Óptica');

    // 5. Manuais & Esquemas Module
    await page.click('button:has-text("Manuais & Esquemas")');
    await expect(page.locator('header')).toContainText('Manuais de Serviço');

    // 6. Inspetor Hexadecimal Module
    await page.click('button:has-text("Inspetor Hexadecimal")');
    await expect(page.locator('header')).toContainText('Inspetor Hexadecimal');
  });
});
