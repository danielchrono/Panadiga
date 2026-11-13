import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useDashboardViewModel } from './useDashboardViewModel';

describe('useDashboardViewModel MVVM Test', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('carries overview data after initialization and exposes refresh()', async () => {
    const { result } = renderHook(() => useDashboardViewModel());

    expect(typeof result.current.refresh).toBe('function');

    await act(async () => {
      await result.current.refresh();
    });

    expect(result.current.loading).toBe(false);
    expect(result.current.overview).not.toBeNull();
    expect(result.current.error).toBeNull();
  });

  it('surfaces backend IPC failures explicitly without silent demo fallback', async () => {
    const previousBridge = window.__PANADIGA_TEST_IPC__;
    window.__PANADIGA_TEST_IPC__ = {
      invoke: async () => {
        throw new Error('Backend IPC offline');
      },
      listen: () => () => {},
    };

    try {
      const { result } = renderHook(() => useDashboardViewModel());
      await act(async () => {
        await result.current.refresh();
      });

      expect(result.current.loading).toBe(false);
      expect(result.current.error).toContain('Backend IPC offline');
    } finally {
      window.__PANADIGA_TEST_IPC__ = previousBridge;
    }
  });
});
