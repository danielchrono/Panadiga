import { AM34_NVRAM_BASE_ADDR } from '@/constants';
import { LiveNvramCaptureEvent, NvramCleanReport } from '@/models';
import { parseP7501DumpBytes } from './diagnosticParser';
import { invokeCommand } from './ipc';

type CaptureListener = () => void;

function createLiveNvramCaptureStore() {
  let latestCapture: LiveNvramCaptureEvent | null = null;
  const listeners = new Set<CaptureListener>();

  const subscribe = (listener: CaptureListener): (() => void) => {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  };

  const getLatest = (): LiveNvramCaptureEvent | null => latestCapture;

  const record = (event: LiveNvramCaptureEvent): void => {
    if (latestCapture === event) {
      return;
    }
    latestCapture = event;
    listeners.forEach(listener => listener());
  };

  const clear = (): void => {
    latestCapture = null;
    listeners.forEach(listener => listener());
  };

  return Object.freeze({ subscribe, getLatest, record, clear });
}

const liveNvramCaptureStore = createLiveNvramCaptureStore();

export function subscribeLiveNvramCapture(listener: CaptureListener): () => void {
  return liveNvramCaptureStore.subscribe(listener);
}

export function getLatestLiveNvramCapture(): LiveNvramCaptureEvent | null {
  return liveNvramCaptureStore.getLatest();
}

export function recordLiveNvramCapture(event: LiveNvramCaptureEvent): void {
  liveNvramCaptureStore.record(event);
}

export function clearLiveNvramCapture(): void {
  liveNvramCaptureStore.clear();
}

/**
 * Decodes real `DUMP` hex words streamed from Desktop CDC, Android USB-OTG, or BLE NUS
 * and runs Rust core NVRAM/microsector structural analysis into the reactive UI SSOT.
 */
export async function ingestP7501DumpOutput(
  rawOutput: string,
  source: string,
  port: string,
  baseAddressOverride?: number
): Promise<LiveNvramCaptureEvent | null> {
  const { baseAddress, bytes } = parseP7501DumpBytes(rawOutput);
  if (bytes.length === 0) {
    return null;
  }

  const effectiveBase = baseAddressOverride ?? baseAddress ?? AM34_NVRAM_BASE_ADDR;
  const baseHex = `0x${effectiveBase.toString(16).toUpperCase().padStart(8, '0')}`;
  const sourceLabel = `Live Pico ${source} (${port}) @ ${baseHex}`;

  const report = await invokeCommand<NvramCleanReport>('analyze_live_nvram_bytes', {
    bytes,
    baseBusAddress: effectiveBase,
    sourceLabel,
  });

  const event: LiveNvramCaptureEvent = {
    source,
    port,
    base_bus_address: baseHex,
    captured_bytes: bytes.length,
    captured_at: new Date().toLocaleTimeString('pt-BR', { hour12: false }),
    report,
  };

  recordLiveNvramCapture(event);
  return event;
}
