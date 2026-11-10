/**
 * IPC Communication & Reactive Event Stream Transport for Panasonic DIGA Suite
 * Single Responsibility: Wraps @tauri-apps/api/core and @tauri-apps/api/event
 * with automatic fallback to `mockDispatcher.ts` ONLY when outside Tauri (e.g. tests).
 */

import { isTauri } from '@tauri-apps/api/core';
import { logger } from '@/utils/logger';
import { handleMockCommand, subscribeMockEventStream } from './mockDispatcher';

export const isTauriEnvironment = (): boolean => {
  return isTauri();
};

let demoModeActive = false;

export const isDemoModeActive = (): boolean => demoModeActive;

export const setDemoMode = (active: boolean): void => {
  demoModeActive = active;
};

/**
 * Universal invoke wrapper with full TypeScript safety.
 * Strictly prioritizes real native Tauri IPC pipeline.
 */
export async function invokeCommand<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  if (isTauri() && !demoModeActive) {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<T>(cmd, args);
  }

  return handleMockCommand<T>(cmd, args);
}

/**
 * Reactive Event Stream Listener for Tauri & Web Environments.
 * Replaces imperative periodic polling with push-based asynchronous streams.
 */
export async function listenEvent<T>(
  eventName: string,
  handler: (payload: T) => void
): Promise<() => void> {
  if (isTauri() && !demoModeActive) {
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
