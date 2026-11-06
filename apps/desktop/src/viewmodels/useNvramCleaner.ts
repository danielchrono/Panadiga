/**
 * @file useNvramCleaner.ts
 * @brief ViewModel for the Spansion S29JL064H NVRAM Sanitizer module.
 *
 * Implements MVVM pattern, managing UI state, input validation, and asynchronous
 * coordination with functional error handling.
 */

import { useState, useCallback } from "react";
import { NvramCleanReport, ExecutionStatus } from "../types/domain";
import { digaApi } from "../services/digaApi";

export interface NvramCleanerViewModel {
  readonly inputPath: string;
  readonly outputPath: string;
  readonly status: ExecutionStatus;
  readonly errorMessage: string | null;
  readonly report: NvramCleanReport | null;
  readonly setInputPath: (path: string) => void;
  readonly setOutputPath: (path: string) => void;
  readonly executeSanitization: () => Promise<void>;
  readonly reset: () => void;
}

export function useNvramCleaner(): NvramCleanerViewModel {
  const [inputPath, setInputPath] = useState<string>("firmware_and_images/spansion_s29jl064h_dump.bin");
  const [outputPath, setOutputPath] = useState<string>("");
  const [status, setStatus] = useState<ExecutionStatus>(ExecutionStatus.IDLE);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [report, setReport] = useState<NvramCleanReport | null>(null);

  const reset = useCallback(() => {
    setStatus(ExecutionStatus.IDLE);
    setErrorMessage(null);
    setReport(null);
  }, []);

  const executeSanitization = useCallback(async () => {
    if (!inputPath || inputPath.trim().length === 0) {
      setErrorMessage("Por favor, informe o caminho do arquivo binário da Flash.");
      setStatus(ExecutionStatus.ERROR);
      return;
    }

    setStatus(ExecutionStatus.RUNNING);
    setErrorMessage(null);

    const result = await digaApi.cleanNvram(inputPath, outputPath || undefined);

    if (result.success) {
      setReport(result.data);
      setStatus(ExecutionStatus.SUCCESS);
    } else {
      setErrorMessage(result.error);
      setStatus(ExecutionStatus.ERROR);
    }
  }, [inputPath, outputPath]);

  return {
    inputPath,
    outputPath,
    status,
    errorMessage,
    report,
    setInputPath,
    setOutputPath,
    executeSanitization,
    reset,
  };
}
