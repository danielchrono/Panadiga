import { test, expect } from '@playwright/test';
import { installMockIpcBridge } from './fixtures/mockIpcBridge';

test.describe('Pico Bench & P7501 UART Hardware Workbench E2E', () => {
  test.beforeEach(async ({ page }) => {
    await installMockIpcBridge(page);
    await page.goto('/');
    await page.click('button:has-text("Bancada PicoBench")');
    await expect(page.locator('header')).toContainText('Bancada Raspberry Pi Pico');
    await expect(page.getByLabel('Selecionar porta serial do Pico')).toHaveValue(/.+/);
  });

  test('displays hardware controller telemetry and architecture badge', async ({ page }) => {
    await expect(page.getByText('Interface Primária P7501 UART 115200 8N1')).toBeVisible();
    await expect(page.getByText('Conexões Físicas (Pico → P7501)')).toBeVisible();
  });

  test('renders pins in the P7501 Pinout table with GND disclaimer', async ({ page }) => {
    await expect(page.getByText('DISCLAIMER CRÍTICO: IDENTIFICAÇÃO DO TERRA (GND)')).toBeVisible();
    await expect(page.getByText('Pino 1 [P7501 SW_UART / XFWE]')).toBeVisible();
    await expect(page.getByText('Pino 3 [P7501 UARTM2P_T / Q7514]')).toBeVisible();
    await expect(page.getByText('Pino 4 [P7501 GND]')).toBeVisible();
    await expect(page.getByText('Pino 5 [P7501 NSW5V / IC1506]')).toBeVisible();
    await expect(page.getByText('Pino 6 [P7501 UARTP2M_T / MAX TXD]')).toBeVisible();
  });

  test('switches P7501 Boot/Reset profile modes dynamically', async ({ page }) => {
    await expect(page.getByText('Seletor de Perfil Boot/Reset P7501')).toBeVisible();
    await expect(page.getByText('Modo 2: JIG Hold LOW (SW_UART Retido)')).toBeVisible();

    // Click Mode 1
    await page.click('text=Modo 1: Serial Pulse (XFWE Pré-Reset)');
    await expect(page.getByText('Modo 1: Serial Pulse (XFWE Pré-Reset)')).toBeVisible();
  });

  test('interacts with the Serial Console terminal', async ({ page }) => {
    const input = page.locator('input[placeholder*="STATUS, RESET, ADC"]');
    await expect(input).toBeVisible();

    await input.fill('STATUS');
    await page.click('button:has-text("Enviar")');

    // Verify TX entry appeared in log
    await expect(page.locator('div').filter({ hasText: '> STATUS' }).first()).toBeVisible();

    // Verify simulated RX response arrived
    await expect(page.getByText('VDD=3.28V | UART=115200-8N1')).toBeVisible();
  });

  test('executes granular PINCHECK and switches Radix tabs in Diagnostic Studio', async ({ page }) => {
    await expect(page.getByText(/Estúdio Granular P7501 Async UART/i)).toBeVisible();
    await page.click('button:has-text("Executar PINCHECK")');
    await expect(page.getByRole('cell', { name: /GP2 \(Pin6 RXD\s*\)/ })).toBeVisible();

    await page.click('[role="tab"]:has-text("OCD & NOR Flash")');
    await page.click('button:has-text("Executar DUMP")');
    await expect(page.getByRole('cell', { name: '0x807F0000' }).first()).toBeVisible();

    await page.click('[role="tab"]:has-text("Mapa Memória AM34")');
    await expect(page.getByText('Top-Boot NVRAM (EEMUL / U99)')).toBeVisible();
  });

  test('operates interactive SC0 Bridge passthrough and quick RSP/Drive triggers', async ({ page }) => {
    await expect(page.getByText(/Interactive SC0 Bridge|Ponte Interativa SC0/i)).toBeVisible();
    await page.click('button:has-text("Start BRIDGE"), button:has-text("Iniciar BRIDGE")');
    await expect(page.getByText(/SC0 BRIDGE ACTIVE|BRIDGE SC0 ATIVA/i)).toBeVisible();

    await page.click('button:has-text("$g#67 Regs")');
    await expect(page.getByText('+$OK#9A').first()).toBeVisible();

    await page.click('button:has-text("PASS")');
    await expect(page.getByText('PASS').first()).toBeVisible();

    await page.click('button:has-text("Exit Bridge (~~~)"), button:has-text("Sair da Ponte (~~~)")');
    await expect(page.getByText('STANDBY').first()).toBeVisible();
  });
});
