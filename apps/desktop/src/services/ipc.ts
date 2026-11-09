/**
 * IPC Communication & Reactive Event Stream Transport for Panasonic DIGA Suite
 * Single Responsibility: Wraps @tauri-apps/api/core and @tauri-apps/api/event
 * with automatic delegation to `mockDispatcher.ts` when running in Demo Mode.
 */

import { logger } from '@/utils/logger';
import { handleMockCommand, subscribeMockEventStream } from './mockDispatcher';

export const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

let demoModeActive = !isTauriEnvironment();

export const isDemoModeActive = (): boolean => demoModeActive;

export const setDemoMode = (active: boolean): void => {
  demoModeActive = active;
};

/**
 * Universal invoke wrapper with full TypeScript safety
 */
export async function invokeCommand<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  if (demoModeActive || !isTauriEnvironment()) {
    return handleMockCommand<T>(cmd, args);
  }

  const { invoke } = await import('@tauri-apps/api/core');
  return await invoke<T>(cmd, args);
}

/**
 * Reactive Event Stream Listener for Tauri & Web Environments.
 * Replaces imperative periodic polling with push-based asynchronous streams.
 */
export async function listenEvent<T>(
  eventName: string,
  handler: (payload: T) => void
): Promise<() => void> {
  if (isTauriEnvironment() && !demoModeActive) {
    try {
      const { listen } = await import('@tauri-apps/api/event');
      return await listen<T>(eventName, event => {
        handler(event.payload);
      });
    } catch (err) {
      logger.warn('IPC', `Failed registering Tauri event listener for ${eventName}`, err);
    }
  }

  return subscribeMockEventStream(eventName, handler);
}
