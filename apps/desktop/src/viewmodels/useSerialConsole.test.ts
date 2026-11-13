import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useSerialConsole } from './useSerialConsole';

describe('useSerialConsole ViewModel', () => {
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

  it('dispatches command to selected port via IPC bridge and logs response', async () => {
    const { result } = renderHook(() => useSerialConsole({ selectedPort: 'COM10' }));

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

  it('logs an explicit error instead of falling back to mocks when no port is selected', async () => {
    const { result } = renderHook(() => useSerialConsole());

    act(() => {
      result.current.setCommandInput('STATUS');
    });

    await act(async () => {
      await result.current.sendCommand();
    });

    const errLog = result.current.logs[result.current.logs.length - 1];
    expect(errLog.level).toBe('error');
  });
});
