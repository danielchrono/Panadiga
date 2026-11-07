import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useSerialConsole } from './useSerialConsole';

describe('useSerialConsole ViewModel', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('initializes with mock logs and empty command input', () => {
    const { result } = renderHook(() => useSerialConsole());
    expect(result.current.logs.length).toBeGreaterThan(0);
    expect(result.current.commandInput).toBe('');
  });

  it('adds custom log entry', () => {
    const { result } = renderHook(() => useSerialConsole());
    const initialCount = result.current.logs.length;

    act(() => {
      result.current.addLog('info', 'HOST', 'Test log message', '0x1234');
    });

    expect(result.current.logs.length).toBe(initialCount + 1);
    const lastLog = result.current.logs[result.current.logs.length - 1];
    expect(lastLog.message).toBe('Test log message');
    expect(lastLog.source).toBe('HOST');
    expect(lastLog.hexData).toBe('0x1234');
  });

  it('clears all logs on clearLogs()', () => {
    const { result } = renderHook(() => useSerialConsole());
    expect(result.current.logs.length).toBeGreaterThan(0);

    act(() => {
      result.current.clearLogs();
    });

    expect(result.current.logs.length).toBe(0);
  });

  it('dispatches command and receives delayed simulated response', () => {
    const { result } = renderHook(() => useSerialConsole());

    act(() => {
      result.current.setCommandInput('STATUS');
    });
    expect(result.current.commandInput).toBe('STATUS');

    act(() => {
      result.current.sendCommand();
    });

    // Input should be reset immediately
    expect(result.current.commandInput).toBe('');
    // Host TX log should be added
    const txLog = result.current.logs[result.current.logs.length - 1];
    expect(txLog.message).toBe('> STATUS');
    expect(txLog.level).toBe('tx');

    // Fast forward timer to trigger Pico RX reply
    act(() => {
      vi.advanceTimersByTime(150);
    });

    const rxLog = result.current.logs[result.current.logs.length - 1];
    expect(rxLog.level).toBe('rx');
    expect(rxLog.source).toBe('PICO');
    expect(rxLog.message).toContain('VDD=3.28V');
  });
});
