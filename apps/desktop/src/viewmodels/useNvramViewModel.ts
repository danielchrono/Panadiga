import { useState, useEffect, useCallback } from 'react';
import { NvramCleanReport, FlashSectorInfo } from '@/models';
import { invokeCommand } from '@/services/ipc';
import { mockFlashSectorMap } from '@/services/mockData';

export function useNvramViewModel() {
  const [report, setReport] = useState<NvramCleanReport | null>(null);
  const [sectorMap, setSectorMap] = useState<FlashSectorInfo[]>(mockFlashSectorMap);
  const [selectedSector, setSelectedSector] = useState<FlashSectorInfo | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [cleaning, setCleaning] = useState<boolean>(false);
  const [cleanSuccess, setCleanSuccess] = useState<boolean>(false);
  const [filterType, setFilterType] = useState<string>('all');

  const loadAnalysis = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invokeCommand<NvramCleanReport>('analyze_flash_dump');
      setReport(data);
      // Default selection to SA134
      const sa134 = mockFlashSectorMap.find(s => s.name === 'SA134');
      if (sa134) setSelectedSector(sa134);
    } catch (err) {
      console.error('Failed analyzing flash dump:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadAnalysis();
  }, [loadAnalysis]);

  const executeSurgicalClean = useCallback(async () => {
    setCleaning(true);
    try {
      const data = await invokeCommand<NvramCleanReport>('clean_nvram');
      setReport(data);
      // Mark microsectors SA134..SA141 as clean in memory
      setSectorMap(prev =>
        prev.map(sec =>
          sec.is_corrupted_area
            ? { ...sec, status: 'clean', active_bytes: 0, notes: 'Virgin 0xFF (Factory Cleared - U99 Resolved)' }
            : sec
        )
      );
      setCleanSuccess(true);
    } catch (err) {
      console.error('Failed cleaning NVRAM:', err);
    } finally {
      setCleaning(false);
    }
  }, []);

  const filteredSectors = sectorMap.filter(s => {
    if (filterType === 'corrupted') return s.is_corrupted_area;
    if (filterType === 'bootloader') return s.sector_type === 'bootloader';
    if (filterType === 'os') return s.sector_type === 'os_firmware';
    return true;
  });

  return {
    report,
    sectorMap,
    filteredSectors,
    selectedSector,
    setSelectedSector,
    loading,
    cleaning,
    cleanSuccess,
    filterType,
    setFilterType,
    executeSurgicalClean,
    refresh: loadAnalysis,
  };
}
