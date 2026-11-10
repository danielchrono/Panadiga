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

  it('initializes with clean logs and empty command input', () => {
    const { result } = renderHook(() => useSerialConsole());
    expect(result.current.logs.length).toBe(0);
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
    act(() => {
      result.current.addLog('info', 'HOST', 'Pre-existing log');
    });
    expect(result.current.logs.length).toBe(1);

    act(() => {
      result.current.clearLogs();
    });

    expect(result.current.logs.length).toBe(0);
  });

  it('dispatches command and receives delayed simulated response', async () => {
    const { result } = renderHook(() => useSerialConsole());

    act(() => {
      result.current.setCommandInput('STATUS');
    });
    expect(result.current.commandInput).toBe('STATUS');

    await act(async () => {
      await result.current.sendCommand();
    });

    // Input should be reset immediately
    expect(result.current.commandInput).toBe('');

    const rxLog = result.current.logs[result.current.logs.length - 1];
    expect(rxLog.level).toBe('rx');
    expect(rxLog.source).toBe('PICO');
    expect(rxLog.message).toContain('VDD=3.28V');
  });
});
