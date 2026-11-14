import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useSc0Bridge } from './useSc0Bridge';
import { handleMockCommand, subscribeMockEventStream } from '@/test/mocks/mockDispatcher';
import type { PanadigaTestIpcBridge } from '@/services/ipc';

function createBridgeTestHarness() {
  const onExec = vi.fn().mockResolvedValue(null);
  const rendered = renderHook(() =>
    useSc0Bridge({ selectedPort: 'COM10', onExecuteDiagnostic: onExec })
  );
  return { onExec, ...rendered };
}

describe('useSc0Bridge ViewModel', () => {
  beforeEach(() => {
    const win = window as Window & { __PANADIGA_TEST_IPC__?: PanadigaTestIpcBridge };
    win.__PANADIGA_TEST_IPC__ = {
      invoke: <T,>(cmd: string, args?: Record<string, unknown>) =>
        handleMockCommand<T>(cmd, args),
      listen: <T,>(eventName: string, handler: (payload: T) => void) =>
        subscribeMockEventStream<T>(eventName, handler),
    };
  });

  it('starts live SC0 bridge, sends RSP/drive payloads, and exits cleanly with ~~~', async () => {
    const { onExec, result } = createBridgeTestHarness();

    await act(async () => {
      await result.current.startBridge();
    });
    expect(result.current.bridgeStatus.running).toBe(true);
    expect(result.current.bridgeLogs.some(l => l.text.includes('[BRIDGE]'))).toBe(true);

    await act(async () => {
      await result.current.sendDriveCommand('PASS');
      await result.current.sendPanaRspPreset('rsp', 'g', '6000 JIG:350 G');
    });
    expect(onExec).not.toHaveBeenCalled();
    expect(result.current.bridgeLogs.some(l => l.text.includes('+$OK#9A'))).toBe(true);

    await act(async () => {
      await result.current.stopBridge();
    });
    expect(result.current.bridgeStatus.running).toBe(false);
    expect(result.current.bridgeLogs.some(l => l.text.includes('BRIDGE: OK (~~~)'))).toBe(true);
  });

  it('falls back to PANARSP and UARTSEND when SC0 bridge is standby', async () => {
    const { onExec, result } = createBridgeTestHarness();

    await act(async () => {
      await result.current.sendPanaRspPreset('hex', '15', '4000 JIG:350 NAK');
      await result.current.sendDriveCommand('DSCREG');
    });
    expect(onExec).toHaveBeenCalledWith('PANARSP', '4000 JIG:350 NAK');
    expect(onExec).toHaveBeenCalledWith('UARTSEND', 'DSCREG');
  });
});
