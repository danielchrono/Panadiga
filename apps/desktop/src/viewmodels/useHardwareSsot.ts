import { useEffect, useState } from 'react';
import { DEFAULT_HARDWARE_SSOT_SPEC } from '@/constants';
import type { HardwareSsotSpec } from '@/models';
import { invokeCommand } from '@/services/ipc';

let cachedSsotSpec: HardwareSsotSpec | null = null;
let pendingSsotRequest: Promise<HardwareSsotSpec> | null = null;

async function fetchHardwareSsot(): Promise<{ spec: HardwareSsotSpec; error: string | null }> {
  if (cachedSsotSpec) return { spec: cachedSsotSpec, error: null };
  if (!pendingSsotRequest) {
    pendingSsotRequest = invokeCommand<HardwareSsotSpec>('get_hardware_ssot_spec')
      .then(spec => {
        if (spec && Array.isArray(spec.pico_commands) && spec.pico_commands.length > 0) {
          cachedSsotSpec = spec;
          return spec;
        }
        return DEFAULT_HARDWARE_SSOT_SPEC;
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
    return { spec: DEFAULT_HARDWARE_SSOT_SPEC, error: msg };
  }
}

/**
 * Single Source of Truth ViewModel: fetches the canonical hardware, pinout, memory map,
 * thresholds, and Pico CLI command catalog directly from Rust backend once per session.
 */
export function useHardwareSsot() {
  const [ssot, setSsot] = useState<HardwareSsotSpec>(cachedSsotSpec ?? DEFAULT_HARDWARE_SSOT_SPEC);
  const [isLoaded, setIsLoaded] = useState<boolean>(cachedSsotSpec !== null);
  const [isLoading, setIsLoading] = useState<boolean>(cachedSsotSpec === null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    fetchHardwareSsot().then(res => {
      if (active) {
        setSsot(res.spec);
        setError(res.error);
        setIsLoaded(res.error === null);
        setIsLoading(false);
      }
    });
    return () => {
      active = false;
    };
  }, []);

  return { ssot, isLoaded, isLoading, error };
}
