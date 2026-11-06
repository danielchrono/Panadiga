/**
 * @file useHddForensics.ts
 * @brief ViewModel for the Panasonic MEIHDFS Storage & Partition Forensics module.
 */

import { useState, useCallback } from "react";
import { HddInspectionReport, ExecutionStatus } from "../types/domain";
import { digaApi } from "../services/digaApi";

export interface HddForensicsViewModel {
  readonly imagePath: string;
  readonly status: ExecutionStatus;
  readonly errorMessage: string | null;
  readonly report: HddInspectionReport | null;
  readonly setImagePath: (path: string) => void;
  readonly executeInspection: () => Promise<void>;
  readonly reset: () => void;
}

export function useHddForensics(): HddForensicsViewModel {
  const [imagePath, setImagePath] = useState<string>(
    "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"
  );
  const [status, setStatus] = useState<ExecutionStatus>(ExecutionStatus.IDLE);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [report, setReport] = useState<HddInspectionReport | null>(null);

  const reset = useCallback(() => {
    setStatus(ExecutionStatus.IDLE);
    setErrorMessage(null);
    setReport(null);
  }, []);

  const executeInspection = useCallback(async () => {
    if (!imagePath || imagePath.trim().length === 0) {
      setErrorMessage("Por favor, selecione ou digite o caminho da imagem de HD.");
      setStatus(ExecutionStatus.ERROR);
      return;
    }

    setStatus(ExecutionStatus.RUNNING);
    setErrorMessage(null);

    const result = await digaApi.inspectHdd(imagePath);

    if (result.success) {
      setReport(result.data);
      setStatus(ExecutionStatus.SUCCESS);
    } else {
      setErrorMessage(result.error);
      setStatus(ExecutionStatus.ERROR);
    }
  }, [imagePath]);

  return {
    imagePath,
    status,
    errorMessage,
    report,
    setImagePath,
    executeInspection,
    reset,
  };
}
