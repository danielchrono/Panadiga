import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useInCircuitErase } from './useInCircuitErase';

describe('useInCircuitErase ViewModel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes in non-erasing state with null eraseResult', () => {
    const { result } = renderHook(() => useInCircuitErase());
    expect(result.current.state.status).toBe('idle');
    expect(result.current.isErasing).toBe(false);
    expect(result.current.eraseResult).toBeNull();
    expect(typeof result.current.injectRamLoaderAndErase).toBe('function');
  });

  it('triggers onLog when invoking injectRamLoaderAndErase', async () => {
    const onLog = vi.fn();
    const onSuccess = vi.fn();
    const { result } = renderHook(() =>
      useInCircuitErase({ onLog, onSuccess })
    );

    await act(async () => {
      await result.current.injectRamLoaderAndErase();
    });

    expect(onLog).toHaveBeenCalledWith(
      'info',
      'HOST',
      expect.stringContaining('Disparando sequência cirúrgica')
    );
    expect(result.current.isErasing).toBe(false);
  });
});
