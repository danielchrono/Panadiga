/**
 * Cross-boundary in-memory pub/sub channel for Playwright/Vitest mock IPC events.
 * Single Responsibility: Bridges mock backend command handlers with `window.__PANADIGA_TEST_IPC__.listen`
 * so events emitted inside the Playwright init-script IIFE reach the Vite React module graph.
 */

type MockEventHandler = (payload: unknown) => void;

const mockListeners: Record<string, MockEventHandler[]> = {};

export function publishMockEvent<T>(eventName: string, payload: T): void {
  const registered = mockListeners[eventName];
  if (!registered) return;
  for (const listener of [...registered]) {
    listener(payload);
  }
}

export function subscribeMockChannel<T>(
  eventName: string,
  handler: (payload: T) => void
): () => void {
  const registered = mockListeners[eventName] ?? (mockListeners[eventName] = []);
  const wrapped: MockEventHandler = payload => handler(payload as T);
  registered.push(wrapped);
  return () => {
    const idx = registered.indexOf(wrapped);
    if (idx >= 0) {
      registered.splice(idx, 1);
    }
  };
}
