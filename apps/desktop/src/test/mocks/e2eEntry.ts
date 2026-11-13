import type { PanadigaTestIpcBridge } from '@/services/ipc';
import { handleMockCommand, subscribeMockEventStream } from './mockDispatcher';

const bridge: PanadigaTestIpcBridge = {
  invoke: <T>(cmd: string, args?: Record<string, unknown>) => handleMockCommand<T>(cmd, args),
  listen: <T>(eventName: string, handler: (payload: T) => void) =>
    subscribeMockEventStream<T>(eventName, handler),
};

(window as Window & { __PANADIGA_TEST_IPC__?: PanadigaTestIpcBridge }).__PANADIGA_TEST_IPC__ = bridge;
