import { test, expect } from '@playwright/test';

test.describe('Pico Bench & DWire32A Hardware Workbench E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.click('button:has-text("Bancada PicoBench")');
    await expect(page.locator('header')).toContainText('Bancada Raspberry Pi Pico');
  });

  test('displays hardware controller telemetry and architecture badge', async ({ page }) => {
    await expect(page.getByText('Matsushita AM34 / UniPhier Single-Wire OCD Host Controller')).toBeVisible();
    await expect(page.getByText('Conexões Físicas (Pico → CN55001)')).toBeVisible();
  });

  test('renders pins in the DWire32A Pinout table with GND disclaimer', async ({ page }) => {
    await expect(page.getByText('DISCLAIMER CRÍTICO: IDENTIFICAÇÃO DO TERRA (GND)')).toBeVisible();
    await expect(page.getByText('Pino 1 (Manual) / Pino 6 (Físico) [VDD 3.3V]:')).toBeVisible();
    await expect(page.getByText('Pino 3 (Manual) / Pino 4 (Físico) [SDATA]:')).toBeVisible();
    await expect(page.getByText('Pino 4 (Manual) / Pino 3 (Físico) [GND]:')).toBeVisible();
    await expect(page.getByText('Pino 5 (Manual) / Pino 2 (Físico) [SCLK]:')).toBeVisible();
    await expect(page.getByText('Pino 6 (Manual) / Pino 1 (Físico) [EXTRG]:')).toBeVisible();
  });

  test('switches Cold Break-In modes dynamically', async ({ page }) => {
    await expect(page.getByText('Seletor de Modo de Cold Break-In')).toBeVisible();
    await expect(page.getByText('Modo 2: EXTRG High (Pre + Post Burst)')).toBeVisible();

    // Click Mode 1
    await page.click('text=Modo 1: EXTRG Low (Pre-Only)');
    await expect(page.getByText('Modo 1: EXTRG Low (Pre-Only)')).toBeVisible();
  });

  test('interacts with the Serial Console terminal', async ({ page }) => {
    const input = page.locator('input[placeholder*="STATUS, RESET, ADC"]');
    await expect(input).toBeVisible();

    await input.fill('STATUS');
    await page.click('button:has-text("Enviar")');

    // Verify TX entry appeared in log
    await expect(page.locator('div').filter({ hasText: '> STATUS' }).first()).toBeVisible();

    // Verify simulated RX response arrived
    await expect(page.getByText('VDD=3.28V | OCD=HALTED')).toBeVisible();
  });
});
