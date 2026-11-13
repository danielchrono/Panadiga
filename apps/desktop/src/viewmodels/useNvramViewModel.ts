import { useState, useEffect, useCallback, useSyncExternalStore } from 'react';
import { NvramCleanReport, FlashSectorInfo, PicoDeviceInfo } from '@/models';
import { invokeCommand } from '@/services/ipc';
import { bleService } from '@/services/bleService';
import {
  connectAndroidUsbOtg,
  getAndroidCapabilities,
  isAndroidNativeBridgeAvailable,
  listAndroidUsbOtgDevices,
  sendAndroidUsbOtgCommand,
} from '@/services/androidBridge';
import {
  clearLiveNvramCapture,
  getLatestLiveNvramCapture,
  ingestP7501DumpOutput,
  recordLiveNvramCapture,
  subscribeLiveNvramCapture,
} from '@/services/liveNvramCaptureStore';
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
  const liveCapture = useSyncExternalStore(
    subscribeLiveNvramCapture,
    getLatestLiveNvramCapture
  );
  const [report, setReport] = useState<NvramCleanReport | null>(() => liveCapture?.report ?? null);
  const [sectorMap, setSectorMap] = useState<FlashSectorInfo[]>(() =>
    liveCapture?.report ? buildSectorMapFromReport(liveCapture.report) : []
  );
  const [selectedSector, setSelectedSector] = useState<FlashSectorInfo | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [capturingLive, setCapturingLive] = useState<boolean>(false);
  const [cleaning, setCleaning] = useState<boolean>(false);
  const [cleanSuccess, setCleanSuccess] = useState<boolean>(false);
  const [filterType, setFilterType] = useState<string>('all');
  const [error, setError] = useState<string | null>(null);

  const applyReportState = useCallback((nextReport: NvramCleanReport) => {
    setReport(nextReport);
    const dynamicSectors = buildSectorMapFromReport(nextReport);
    setSectorMap(dynamicSectors);
    setCleanSuccess(nextReport.nvram_non_ff_bytes === 0);

    const firstCorrupted = dynamicSectors.find(s => s.is_corrupted_area);
    const firstMicro = dynamicSectors.find(s => s.sector_type === 'nvram_microsector');
    setSelectedSector(firstCorrupted || firstMicro || dynamicSectors[0] || null);
  }, []);

  useEffect(() => {
    if (liveCapture?.report) {
      applyReportState(liveCapture.report);
      setError(null);
    }
  }, [liveCapture, applyReportState]);

  const loadAnalysis = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invokeCommand<NvramCleanReport>('analyze_flash_dump');
      if (!getLatestLiveNvramCapture()) {
        applyReportState(data);
      }
    } catch (err) {
      if (!getLatestLiveNvramCapture()) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
        logger.error('NvramViewModel', 'Failed analyzing flash dump', err);
      }
    } finally {
      setLoading(false);
    }
  }, [applyReportState]);

  useEffect(() => {
    loadAnalysis();
  }, [loadAnalysis]);

  const captureFromLivePico = useCallback(
    async (explicitPort?: string, baseAddr = 0x807f0000, wordCount = 16384) => {
      setCapturingLive(true);
      setError(null);
      try {
        const androidCaps = isAndroidNativeBridgeAvailable() ? getAndroidCapabilities() : null;
        if (
          androidCaps?.usbHostSupported &&
          !explicitPort?.startsWith('COM') &&
          !explicitPort?.startsWith('/dev/')
        ) {
          const dumpCmd = `DUMP 0x${baseAddr.toString(16).toUpperCase()} ${Math.min(wordCount, 2048)}`;
          let reply = sendAndroidUsbOtgCommand(dumpCmd, 6000);
          if (!reply.ok && listAndroidUsbOtgDevices().length > 0) {
            const conn = connectAndroidUsbOtg();
            if (conn.connected) {
              reply = sendAndroidUsbOtgCommand(dumpCmd, 6000);
            }
          }
          if (!reply.ok) {
            throw new Error(reply.output || 'Falha no comando DUMP via Android USB-OTG.');
          }
          const captured = await ingestP7501DumpOutput(reply.output, 'ANDROID_USB_OTG', 'ANDROID-USB-OTG', baseAddr);
          if (!captured) {
            throw new Error('Nenhuma palavra hexadecimal capturada na resposta DUMP USB-OTG.');
          }
          return captured;
        }

        if (bleService.isConnected() && (!explicitPort || explicitPort.startsWith('BLE_'))) {
          const reply = await bleService.sendAndAwaitReply(
            `DUMP 0x${baseAddr.toString(16).toUpperCase()} ${Math.min(wordCount, 256)}`,
            6000
          );
          const captured = await ingestP7501DumpOutput(reply, 'BLE_NUS', 'BLE_PICO_W', baseAddr);
          if (!captured) {
            throw new Error('Nenhuma palavra hexadecimal capturada na resposta DUMP BLE NUS.');
          }
          return captured;
        }

        let targetPort = explicitPort?.trim() || '';
        if (!targetPort) {
          const ports = await invokeCommand<PicoDeviceInfo[]>('list_pico_serial_ports');
          const preferred = ports.find(p => p.architecture !== 'Bootloader' && !p.port_name.startsWith('MOCK'));
          targetPort = preferred?.port_name || '';
        }
        if (!targetPort) {
          throw new Error(
            'Nenhuma sonda Pico conectada (P7501 Async UART). Conecte via USB CDC/OTG ou BLE e entre em CDDIAG.'
          );
        }

        const evt = await invokeCommand<import('@/models').LiveNvramCaptureEvent>('pico_capture_nvram_live', {
          portName: targetPort,
          port_name: targetPort,
          baseAddr,
          base_addr: baseAddr,
          wordCount,
          word_count: wordCount,
        });
        recordLiveNvramCapture(evt);
        return evt;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
        logger.error('NvramViewModel', 'Failed live Pico NVRAM capture', err);
        return null;
      } finally {
        setCapturingLive(false);
      }
    },
    []
  );

  const clearLiveCaptureAndReload = useCallback(async () => {
    clearLiveNvramCapture();
    await loadAnalysis();
  }, [loadAnalysis]);

  const executeSurgicalClean = useCallback(async () => {
    setCleaning(true);
    setError(null);
    try {
      const data = await invokeCommand<NvramCleanReport>('clean_nvram');
      setReport(data);
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
    if (filterType === 'nvram') return s.sector_type === 'nvram_microsector';
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
    capturingLive,
    liveCapture,
    cleaning,
    cleanSuccess,
    filterType,
    setFilterType,
    error,
    executeSurgicalClean,
    captureFromLivePico,
    clearLiveCapture: clearLiveCaptureAndReload,
    refresh: loadAnalysis,
  };
}
