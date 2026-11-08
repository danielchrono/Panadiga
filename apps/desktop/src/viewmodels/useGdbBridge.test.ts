import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useGdbBridge } from './useGdbBridge';

describe('useGdbBridge ViewModel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes with default status and exposes bridge actions', () => {
    const onLog = vi.fn();
    const { result } = renderHook(() =>
      useGdbBridge({ selectedPort: 'COM5', onLog })
    );

    expect(result.current.state.status).toBe('stopped');
    expect(result.current.gdbStatus).toBeNull();
    expect(result.current.isGdbStarting).toBe(false);
    expect(typeof result.current.startGdbServer).toBe('function');
    expect(typeof result.current.stopGdbServer).toBe('function');
    expect(typeof result.current.launchTool).toBe('function');
  });

  it('triggers onLog when starting and stopping GDB server', async () => {
    const onLog = vi.fn();
    const { result } = renderHook(() =>
      useGdbBridge({ selectedPort: 'COM5', onLog })
    );

    await act(async () => {
      await result.current.startGdbServer('127.0.0.1:3333');
    });

    expect(onLog).toHaveBeenCalledWith(
      'info',
      'HOST',
      expect.stringContaining('Iniciando Servidor Ponte GDB RSP')
    );

    await act(async () => {
      await result.current.stopGdbServer();
    });

    expect(onLog).toHaveBeenCalledWith(
      'warn',
      'HOST',
      expect.stringContaining('Finalizando Servidor Ponte GDB RSP')
    );
  });
});
