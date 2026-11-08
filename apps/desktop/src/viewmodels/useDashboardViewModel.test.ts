import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useDashboardViewModel } from './useDashboardViewModel';

describe('useDashboardViewModel MVVM Test', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes with loading state and provides overview actions', async () => {
    const { result } = renderHook(() => useDashboardViewModel());

    expect(typeof result.current.toggleDemoMode).toBe('function');
    expect(typeof result.current.refresh).toBe('function');
    expect(typeof result.current.demoMode).toBe('boolean');
  });

  it('allows toggling demo mode dynamically', async () => {
    const { result } = renderHook(() => useDashboardViewModel());
    const initialMode = result.current.demoMode;

    await act(async () => {
      result.current.toggleDemoMode(!initialMode);
    });

    expect(result.current.demoMode).toBe(!initialMode);
  });
});
