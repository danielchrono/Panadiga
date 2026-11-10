import { useState, useEffect, useCallback } from 'react';
import { NvramCleanReport, FlashSectorInfo } from '@/models';
import { invokeCommand } from '@/services/ipc';
import { logger } from '@/utils/logger';

/**
 * Dynamically synthesizes the Flash Sector Map (SSOT) directly from the live
 * NvramCleanReport returned by the Rust kernel, supporting any DIGA geometry
 * (2 MB, 4 MB, 8 MB, 16 MB) without hardcoded mock arrays.
 */
export function buildSectorMapFromReport(report: NvramCleanReport | null | undefined): FlashSectorInfo[] {
  if (!report) return [];
  const sectors: FlashSectorInfo[] = [];
  const nvramStart = report.nvram_offset_start ?? 0x007F0000;
  const standardSectorSize = 65536; // 64 KB

  // Number of standard 64KB sectors before the top-boot NVRAM microsectors
  const standardSectorCount = Math.floor(nvramStart / standardSectorSize);

  for (let i = 0; i < standardSectorCount; i++) {
    const startOffset = i * standardSectorSize;
    const endOffset = startOffset + standardSectorSize - 1;
    const isBootloader = i < 16;

    sectors.push({
      index: i,
      name: `SA${i}`,
      start_offset: startOffset,
      end_offset: endOffset,
      size_bytes: standardSectorSize,
      sector_type: isBootloader ? 'bootloader' : 'os_firmware',
      is_corrupted_area: false,
      status: isBootloader ? 'immutable' : 'clean',
      active_bytes: standardSectorSize,
      notes: isBootloader
        ? `Primary AM34 Bootloader Bank (0x${startOffset.toString(16).toUpperCase().padStart(8, '0')})`
        : `Main OS & Codec Executable Segment (0x${startOffset.toString(16).toUpperCase().padStart(8, '0')})`,
    });
  }

  // Parameter Microsectors from live report audit
  (report.microsectors || []).forEach((ms, idx) => {
    const sectorIndex = standardSectorCount + idx;
    const isCorrupted = ms.active_bytes > 0;
    sectors.push({
      index: sectorIndex,
      name: ms.sector_name,
      start_offset: ms.offset,
      end_offset: ms.offset + ms.total_bytes - 1,
      size_bytes: ms.total_bytes,
      sector_type: 'nvram_microsector',
      is_corrupted_area: isCorrupted,
      status: isCorrupted ? 'corrupted' : 'clean',
      active_bytes: ms.active_bytes,
      notes: `Top Parameter Bank: ${ms.sector_name} (EEMUL NVRAM / 0x${ms.offset.toString(16).toUpperCase()})`,
    });
  });

  return sectors;
}

export function useNvramViewModel() {
  const [report, setReport] = useState<NvramCleanReport | null>(null);
  const [sectorMap, setSectorMap] = useState<FlashSectorInfo[]>([]);
  const [selectedSector, setSelectedSector] = useState<FlashSectorInfo | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [cleaning, setCleaning] = useState<boolean>(false);
  const [cleanSuccess, setCleanSuccess] = useState<boolean>(false);
  const [filterType, setFilterType] = useState<string>('all');
  const [error, setError] = useState<string | null>(null);

  const loadAnalysis = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invokeCommand<NvramCleanReport>('analyze_flash_dump');
      setReport(data);
      const dynamicSectors = buildSectorMapFromReport(data);
      setSectorMap(dynamicSectors);

      // Default selection to first corrupted microsector or first microsector
      const firstCorrupted = dynamicSectors.find(s => s.is_corrupted_area);
      const firstMicro = dynamicSectors.find(s => s.sector_type === 'nvram_microsector');
      setSelectedSector(firstCorrupted || firstMicro || dynamicSectors[0] || null);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      logger.error('NvramViewModel', 'Failed analyzing flash dump', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadAnalysis();
  }, [loadAnalysis]);

  const executeSurgicalClean = useCallback(async () => {
    setCleaning(true);
    setError(null);
    try {
      const data = await invokeCommand<NvramCleanReport>('clean_nvram');
      setReport(data);
      // Dynamically rebuild sector map from newly cleaned state
      const cleanedSectors = buildSectorMapFromReport(data);
      setSectorMap(cleanedSectors);
      setCleanSuccess(true);

      const firstMicro = cleanedSectors.find(s => s.sector_type === 'nvram_microsector');
      if (firstMicro) setSelectedSector(firstMicro);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      logger.error('NvramViewModel', 'Failed cleaning NVRAM', err);
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
    error,
    executeSurgicalClean,
    refresh: loadAnalysis,
  };
}
