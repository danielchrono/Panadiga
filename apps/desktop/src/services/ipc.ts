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

function toCamelCaseKey(key: string): string {
  return key.replace(/_([a-z0-9])/g, (_, char: string) => char.toUpperCase());
}

/**
 * Normalizes argument dictionaries so that both camelCase (required by Tauri v2 IPC commands)
 * and snake_case (legacy/internal) keys are populated transparently.
 */
export function normalizeArgsForTauri(args?: Record<string, unknown>): Record<string, unknown> | undefined {
  if (!args) return undefined;
  const normalized: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(args)) {
    const camel = toCamelCaseKey(key);
    normalized[camel] = value;
    if (camel !== key) {
      normalized[key] = value;
    }
  }
  return normalized;
}

/**
 * Universal invoke wrapper with full TypeScript safety.
 * Strictly prioritizes real native Tauri IPC pipeline.
 */
export async function invokeCommand<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  const normalizedArgs = normalizeArgsForTauri(args);

  if (isTauri() && !demoModeActive) {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<T>(cmd, normalizedArgs);
  }

  return handleMockCommand<T>(cmd, normalizedArgs);
}

type EventHandler<T> = (payload: T) => void;
const localEventBus: Map<string, Set<EventHandler<unknown>>> = new Map();

/**
 * Dispatches an event across the frontend subscriber network.
 * Allows ViewModels (like usePicoDevice) to broadcast telemetry to top-level App shell.
 */
export function emitLocalEvent<T>(eventName: string, payload: T): void {
  const handlers = localEventBus.get(eventName);
  if (handlers) {
    handlers.forEach(fn => {
      try {
        fn(payload as unknown);
      } catch (e) {
        logger.warn('IPC', `Error in local event handler for ${eventName}`, e);
      }
    });
  }
}

/**
 * Reactive Event Stream Listener for Tauri & Web Environments.
 * Replaces imperative periodic polling with push-based asynchronous streams.
 */
export async function listenEvent<T>(
  eventName: string,
  handler: (payload: T) => void
): Promise<() => void> {
  if (!localEventBus.has(eventName)) {
    localEventBus.set(eventName, new Set());
  }
  const handlerFn = handler as EventHandler<unknown>;
  localEventBus.get(eventName)!.add(handlerFn);
  const unlistenLocal = () => {
    localEventBus.get(eventName)?.delete(handlerFn);
  };

  let unlistenTauri: (() => void) | undefined;
  if (isTauri() && !demoModeActive) {
    try {
      const { listen } = await import('@tauri-apps/api/event');
      unlistenTauri = await listen<T>(eventName, event => {
        handler(event.payload);
      });
    } catch (err) {
      logger.warn('IPC', `Failed registering Tauri event listener for ${eventName}`, err);
    }
  } else if (!isTauri() || demoModeActive) {
    const unlistenMock = subscribeMockEventStream(eventName, handler);
    return () => {
      unlistenLocal();
      unlistenMock();
    };
  }

  return () => {
    unlistenLocal();
    unlistenTauri?.();
  };
}
