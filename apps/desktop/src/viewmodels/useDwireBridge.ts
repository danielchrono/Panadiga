/**
 * @file useDwireBridge.ts
 * @brief ViewModel for the Panasonic DWire32A Pico In-Circuit Bridge module.
 */

import { useState, useCallback } from "react";
import { DwireScanStats, DwireBreakInMode, ExecutionStatus } from "../types/domain";
import { digaApi } from "../services/digaApi";

export interface DwireBridgeViewModel {
  readonly mode: DwireBreakInMode;
  readonly status: ExecutionStatus;
  readonly errorMessage: string | null;
  readonly stats: DwireScanStats | null;
  readonly setMode: (mode: DwireBreakInMode) => void;
  readonly executeScan: () => Promise<void>;
  readonly reset: () => void;
}

export function useDwireBridge(): DwireBridgeViewModel {
  const [mode, setMode] = useState<DwireBreakInMode>(DwireBreakInMode.MODE_0);
  const [status, setStatus] = useState<ExecutionStatus>(ExecutionStatus.IDLE);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [stats, setStats] = useState<DwireScanStats | null>(null);

  const reset = useCallback(() => {
    setStatus(ExecutionStatus.IDLE);
    setErrorMessage(null);
    setStats(null);
  }, []);

  const executeScan = useCallback(async () => {
    setStatus(ExecutionStatus.RUNNING);
    setErrorMessage(null);

    const result = await digaApi.scanDwireBus(mode);

    if (result.success) {
      setStats(result.data);
      setStatus(ExecutionStatus.SUCCESS);
    } else {
      setErrorMessage(result.error);
      setStatus(ExecutionStatus.ERROR);
    }
  }, [mode]);

  return {
    mode,
    status,
    errorMessage,
    stats,
    setMode,
    executeScan,
    reset,
  };
}
