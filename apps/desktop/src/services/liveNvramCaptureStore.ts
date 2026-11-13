import { AM34_NVRAM_BASE_ADDR } from '@/constants';
import { LiveNvramCaptureEvent, NvramCleanReport } from '@/models';
import { parseP7501DumpBytes } from './diagnosticParser';
import { invokeCommand } from './ipc';

type CaptureListener = () => void;

let latestCapture: LiveNvramCaptureEvent | null = null;
const listeners = new Set<CaptureListener>();

export function subscribeLiveNvramCapture(listener: CaptureListener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getLatestLiveNvramCapture(): LiveNvramCaptureEvent | null {
  return latestCapture;
}

export function recordLiveNvramCapture(event: LiveNvramCaptureEvent): void {
  if (latestCapture === event) {
    return;
  }
  latestCapture = event;
  listeners.forEach(listener => listener());
}

export function clearLiveNvramCapture(): void {
  latestCapture = null;
  listeners.forEach(listener => listener());
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
