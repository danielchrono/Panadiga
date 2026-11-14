import { test, expect } from '@playwright/test';
import { installMockIpcBridge } from './fixtures/mockIpcBridge';

test.describe('PANA_DVD / PANA__SD FRM Decrypted TOC & Packager E2E', () => {
  test.beforeEach(async ({ page }) => {
    await installMockIpcBridge(page);
    await page.goto('/');
    await page.click('button:has-text("Criptoanálise FRM")');
    await expect(page.locator('header')).toContainText('Criptoanálise Óptica');
  });

  test('renders decrypted Firm list TOC entries and executes Pack + EXTRHEAD Unpack', async ({ page }) => {
    await page.getByRole('button', { name: /TOC \(Feistel 48B \/ SD 64B\)/i }).click();
    await expect(page.getByText('DRV S211').first()).toBeVisible();
    await expect(page.getByText('MAIN3060NP').first()).toBeVisible();
    await expect(page.getByText(/0x0AE1 OK/i)).toBeVisible();

    await page.getByRole('button', { name: /(Packager|Empacotador).*EXTRHEAD/i }).click();
    await page.locator('input[placeholder*="Caminho payload"]').first().fill('firmware_and_images/out/drv_s211.bin');
    await page.locator('button[type="submit"]:has-text("Empacotar"), button[type="submit"]:has-text("Pack .FRM Container")').first().click();
    await expect(page.getByText(/DIAG01/i).first()).toBeVisible();

    await page.locator('button[type="submit"]:has-text("Desempacotar"), button[type="submit"]:has-text("Unpack & Verify")').first().click();
    await expect(page.getByText(/Adler32 OK/i)).toBeVisible();
  });
});
