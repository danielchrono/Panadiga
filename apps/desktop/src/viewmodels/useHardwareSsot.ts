import { useCallback, useEffect, useState } from 'react';
import type { HardwareSsotSpec, LiveNvramCaptureEvent, PicoDeviceStatus } from '@/models';
import { invokeCommand, listenEvent } from '@/services/ipc';
import { recordLiveNvramCapture } from '@/services/liveNvramCaptureStore';

function createHardwareSsotCacheStore() {
  let cachedSsotSpec: HardwareSsotSpec | null = null;
  let pendingSsotRequest: Promise<HardwareSsotSpec> | null = null;

  const getCached = (): HardwareSsotSpec | null => cachedSsotSpec;

  const reset = (): void => {
    cachedSsotSpec = null;
    pendingSsotRequest = null;
  };

  const fetchSpec = async (forceReload = false): Promise<{
    spec: HardwareSsotSpec | null;
    error: string | null;
  }> => {
    if (!forceReload && cachedSsotSpec) {
      return { spec: cachedSsotSpec, error: null };
    }
    if (forceReload) {
      cachedSsotSpec = null;
    }
    if (!pendingSsotRequest) {
      pendingSsotRequest = invokeCommand<HardwareSsotSpec>('get_hardware_ssot_spec')
        .then(spec => {
          if (!spec || !Array.isArray(spec.pico_commands) || spec.pico_commands.length === 0) {
            throw new Error('Especificação SSOT de hardware vazia ou inválida recebida do backend.');
          }
          cachedSsotSpec = spec;
          return spec;
        })
        .finally(() => {
          pendingSsotRequest = null;
        });
    }
    try {
      const spec = await pendingSsotRequest;
      return { spec, error: null };
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      return { spec: null, error: msg };
    }
  };

  return Object.freeze({ getCached, reset, fetchSpec });
}

const ssotCacheStore = createHardwareSsotCacheStore();

export function resetHardwareSsotCacheForTests(): void {
  ssotCacheStore.reset();
}

export interface ShellTelemetryStatus {
  readonly connected: boolean;
  readonly vdd_volts: number;
}

/**
 * Top-level App shell ViewModel: listens for reactive Pico telemetry and live NVRAM capture events
 * so UI shell components remain decoupled from raw IPC and store internals.
 */
export function useShellTelemetryListener(): ShellTelemetryStatus {
  const [hardwareStatus, setHardwareStatus] = useState<ShellTelemetryStatus>({
    connected: false,
    vdd_volts: 0.0,
  });

  useEffect(() => {
    let unlistenLegacy: (() => void) | undefined;
    let unlistenStream: (() => void) | undefined;
    let unlistenLiveNvram: (() => void) | undefined;

    const applyStatus = (incoming: PicoDeviceStatus) => {
      if (!incoming) return;
      setHardwareStatus(prev => ({
        connected: Boolean(incoming.connected),
        vdd_volts:
          typeof incoming.vdd_volts === 'number' && incoming.vdd_volts > 0
            ? incoming.vdd_volts
            : prev.connected && !incoming.connected
              ? 0.0
              : prev.vdd_volts > 0 && incoming.connected
                ? prev.vdd_volts
                : Number(incoming.vdd_volts ?? 0),
      }));
    };

    listenEvent<PicoDeviceStatus>('pico_telemetry', applyStatus).then(fn => {
      unlistenLegacy = fn;
    });
    listenEvent<PicoDeviceStatus>('pico://telemetry', applyStatus).then(fn => {
      unlistenStream = fn;
    });
    listenEvent<LiveNvramCaptureEvent>('pico://nvram-live-capture', evt => {
      if (evt?.report) {
        recordLiveNvramCapture(evt);
      }
    }).then(fn => {
      unlistenLiveNvram = fn;
    });

    return () => {
      unlistenLegacy?.();
      unlistenStream?.();
      unlistenLiveNvram?.();
    };
  }, []);

  return hardwareStatus;
}

/**
 * Single Source of Truth ViewModel: fetches the canonical hardware, pinout, memory map,
 * thresholds, and Pico CLI command catalog directly from Rust backend once per session.
 */
export function useHardwareSsot() {
  const initialCached = ssotCacheStore.getCached();
  const [ssot, setSsot] = useState<HardwareSsotSpec | null>(initialCached);
  const [isLoaded, setIsLoaded] = useState<boolean>(initialCached !== null);
  const [isLoading, setIsLoading] = useState<boolean>(initialCached === null);
  const [error, setError] = useState<string | null>(null);

  const loadSsot = useCallback((forceReload = false) => {
    setIsLoading(true);
    setError(null);
    return ssotCacheStore.fetchSpec(forceReload).then(res => {
      setSsot(res.spec);
      setError(res.error);
      setIsLoaded(res.spec !== null && res.error === null);
      setIsLoading(false);
    });
  }, []);

  useEffect(() => {
    let active = true;
    const cached = ssotCacheStore.getCached();
    if (cached) {
      setSsot(cached);
      setIsLoaded(true);
      setIsLoading(false);
      return;
    }
    ssotCacheStore.fetchSpec().then(res => {
      if (active) {
        setSsot(res.spec);
        setError(res.error);
        setIsLoaded(res.spec !== null && res.error === null);
        setIsLoading(false);
      }
    });
    return () => {
      active = false;
    };
  }, []);

  const reload = useCallback(() => {
    void loadSsot(true);
  }, [loadSsot]);

  return { ssot, isLoaded, isLoading, error, reload };
}
