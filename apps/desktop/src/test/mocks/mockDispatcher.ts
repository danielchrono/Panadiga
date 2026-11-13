/**
 * High-Fidelity Mock Command Dispatcher & Reactive Stream Simulator
 * Single Responsibility: Thin router delegating domain IPC calls to `mocks/` handlers
 * when running in Browser/Workbench Demo Mode or headless Vitest/Playwright suites.
 */

import { logger } from '@/utils/logger';
import { handleMockCryptoCommand } from './mockCryptoHandlers';
import { handleMockHddCommand } from './mockHddHandlers';
import { buildMockPicoStatus, handleMockPicoCommand } from './mockPicoHandlers';
import { handleMockSystemCommand } from './mockSystemHandlers';

type MockDomainHandler = (cmd: string, args?: Record<string, unknown>) => unknown | undefined;

const DOMAIN_HANDLERS: MockDomainHandler[] = [
  handleMockSystemCommand,
  handleMockHddCommand,
  handleMockCryptoCommand,
  handleMockPicoCommand,
];

export async function handleMockCommand<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  await new Promise(resolve => setTimeout(resolve, 80));

  for (const handler of DOMAIN_HANDLERS) {
    const result = handler(cmd, args);
    if (result !== undefined) {
      return result as T;
    }
  }

  logger.warn('MockIPC', `Unhandled command: ${cmd}`);
  return {} as unknown as T;
}

export function subscribeMockEventStream<T>(
  eventName: string,
  handler: (payload: T) => void
): () => void {
  if (eventName === 'pico_telemetry' || eventName === 'pico://telemetry') {
    const interval = setInterval(() => {
      handler(buildMockPicoStatus() as unknown as T);
    }, 2000);
    return () => clearInterval(interval);
  }
  return () => { };
}
