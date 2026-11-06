/**
 * @file useFrmAnalyzer.ts
 * @brief ViewModel for the PANA_DVD.FRM Container & Cryptanalysis module.
 */

import { useState, useCallback } from "react";
import { FrmAnalysisReport, ExecutionStatus } from "../types/domain";
import { digaApi } from "../services/digaApi";

export interface FrmAnalyzerViewModel {
  readonly frmPath: string;
  readonly status: ExecutionStatus;
  readonly errorMessage: string | null;
  readonly report: FrmAnalysisReport | null;
  readonly setFrmPath: (path: string) => void;
  readonly executeAnalysis: () => Promise<void>;
  readonly reset: () => void;
}

export function useFrmAnalyzer(): FrmAnalyzerViewModel {
  const [frmPath, setFrmPath] = useState<string>("firmware_and_images/PANA_DVD.FRM");
  const [status, setStatus] = useState<ExecutionStatus>(ExecutionStatus.IDLE);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [report, setReport] = useState<FrmAnalysisReport | null>(null);

  const reset = useCallback(() => {
    setStatus(ExecutionStatus.IDLE);
    setErrorMessage(null);
    setReport(null);
  }, []);

  const executeAnalysis = useCallback(async () => {
    if (!frmPath || frmPath.trim().length === 0) {
      setErrorMessage("Por favor, selecione ou digite o caminho do arquivo PANA_DVD.FRM.");
      setStatus(ExecutionStatus.ERROR);
      return;
    }

    setStatus(ExecutionStatus.RUNNING);
    setErrorMessage(null);

    const result = await digaApi.analyzeFrm(frmPath);

    if (result.success) {
      setReport(result.data);
      setStatus(ExecutionStatus.SUCCESS);
    } else {
      setErrorMessage(result.error);
      setStatus(ExecutionStatus.ERROR);
    }
  }, [frmPath]);

  return {
    frmPath,
    status,
    errorMessage,
    report,
    setFrmPath,
    executeAnalysis,
    reset,
  };
}
