import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useNvramViewModel } from './useNvramViewModel';

describe('useNvramViewModel MVVM Test', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initializes sector map and provides filtering and surgical clean actions', async () => {
    const { result } = renderHook(() => useNvramViewModel());

    expect(Array.isArray(result.current.sectorMap)).toBe(true);
    expect(result.current.filterType).toBe('all');
    expect(typeof result.current.executeSurgicalClean).toBe('function');
    expect(typeof result.current.setFilterType).toBe('function');
    expect(typeof result.current.setSelectedSector).toBe('function');

    await waitFor(() => {
      expect(result.current.sectorMap.length).toBeGreaterThan(0);
    });
  });

  it('filters sectors according to category criteria', async () => {
    const { result } = renderHook(() => useNvramViewModel());

    await waitFor(() => {
      expect(result.current.sectorMap.length).toBeGreaterThan(0);
    });

    await act(async () => {
      result.current.setFilterType('corrupted');
    });

    expect(result.current.filterType).toBe('corrupted');
    expect(result.current.filteredSectors.every(s => s.is_corrupted_area)).toBe(true);
  });

  it('supports selecting a specific microsector for deep inspection', async () => {
    const { result } = renderHook(() => useNvramViewModel());

    await waitFor(() => {
      expect(result.current.sectorMap.length).toBeGreaterThan(0);
    });

    const targetSector = result.current.sectorMap[0];

    await act(async () => {
      result.current.setSelectedSector(targetSector);
    });

    expect(result.current.selectedSector).toBe(targetSector);
  });
});
