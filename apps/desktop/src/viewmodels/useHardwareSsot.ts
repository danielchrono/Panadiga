import { useCallback, useEffect, useState } from 'react';
import type { HardwareSsotSpec } from '@/models';
import { invokeCommand } from '@/services/ipc';

let cachedSsotSpec: HardwareSsotSpec | null = null;
let pendingSsotRequest: Promise<HardwareSsotSpec> | null = null;

async function fetchHardwareSsot(forceReload = false): Promise<{
  spec: HardwareSsotSpec | null;
  error: string | null;
}> {
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
}

export function resetHardwareSsotCacheForTests(): void {
  cachedSsotSpec = null;
  pendingSsotRequest = null;
}

/**
 * Single Source of Truth ViewModel: fetches the canonical hardware, pinout, memory map,
 * thresholds, and Pico CLI command catalog directly from Rust backend once per session.
 */
export function useHardwareSsot() {
  const [ssot, setSsot] = useState<HardwareSsotSpec | null>(cachedSsotSpec);
  const [isLoaded, setIsLoaded] = useState<boolean>(cachedSsotSpec !== null);
  const [isLoading, setIsLoading] = useState<boolean>(cachedSsotSpec === null);
  const [error, setError] = useState<string | null>(null);

  const loadSsot = useCallback((forceReload = false) => {
    setIsLoading(true);
    setError(null);
    return fetchHardwareSsot(forceReload).then(res => {
      setSsot(res.spec);
      setError(res.error);
      setIsLoaded(res.spec !== null && res.error === null);
      setIsLoading(false);
    });
  }, []);

  useEffect(() => {
    let active = true;
    if (cachedSsotSpec) {
      setSsot(cachedSsotSpec);
      setIsLoaded(true);
      setIsLoading(false);
      return;
    }
    fetchHardwareSsot().then(res => {
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
