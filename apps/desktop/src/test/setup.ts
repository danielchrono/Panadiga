import { vi } from "vitest";
import { handleMockCommand, subscribeMockEventStream } from "./mocks/mockDispatcher";

if (typeof window !== "undefined") {
  window.__PANADIGA_TEST_IPC__ = {
    invoke: handleMockCommand,
    listen: subscribeMockEventStream,
  };
}

// Route Tauri core & event imports cleanly when mocked directly in unit tests
vi.mock("@tauri-apps/api/core", () => ({
  isTauri: vi.fn(() => false),
  invoke: vi.fn().mockImplementation((cmd: string, args?: Record<string, unknown>) => {
    const activeBridge =
      typeof window !== "undefined" ? window.__PANADIGA_TEST_IPC__ : undefined;
    return activeBridge
      ? activeBridge.invoke(cmd, args)
      : handleMockCommand(cmd, args);
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockImplementation((eventName: string, handler: (event: { payload: unknown }) => void) => {
    const activeBridge =
      typeof window !== "undefined" ? window.__PANADIGA_TEST_IPC__ : undefined;
    if (activeBridge) {
      return Promise.resolve(activeBridge.listen(eventName, payload => handler({ payload })));
    }
    return Promise.resolve(subscribeMockEventStream(eventName, payload => handler({ payload })));
  }),
}));

// Mock window matchMedia
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
