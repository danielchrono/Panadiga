import type { Page } from '@playwright/test';
import { buildSync } from 'esbuild';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

let compiledBridgeScript: string | null = null;

function getTestBridgeScript(): string {
  if (!compiledBridgeScript) {
    const entryPoint = resolve(__dirname, '../../src/test/mocks/e2eEntry.ts');
    const tsconfig = resolve(__dirname, '../../tsconfig.json');
    const result = buildSync({
      entryPoints: [entryPoint],
      bundle: true,
      write: false,
      format: 'iife',
      platform: 'browser',
      tsconfig,
      define: {
        'import.meta.env.DEV': 'true',
        'import.meta.env.MODE': '"test"',
      },
    });
    compiledBridgeScript = result.outputFiles[0].text;
  }
  return compiledBridgeScript;
}

/**
 * Injects the test-only IPC bridge into the Playwright browser context
 * before any page scripts run, keeping production bundles free of mocks.
 */
export async function installMockIpcBridge(page: Page): Promise<void> {
  await page.addInitScript({ content: getTestBridgeScript() });
}
