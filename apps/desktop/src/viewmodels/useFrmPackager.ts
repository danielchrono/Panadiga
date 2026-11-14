import { useCallback, useEffect, useState } from 'react';
import {
  ExtrheadUnpackReport,
  FrmPackReport,
  PackFrmRequest,
} from '@/models';
import { invokeCommand } from '@/services/ipc';
import { logger } from '@/utils/logger';

export interface UnpackExtrheadParams {
  inputPath: string;
  mode: 'auto' | 'main' | 'drv' | 'plain';
  outPath?: string;
}

/**
 * ViewModel for listing .FRM containers, packing PANA__SD.FRM / PANA_DVD.FRM / DIGADIAG.FRM,
 * and unpacking/verifying EXTRHEAD/EXTRFOOT payloads via native Rust backend.
 */
export function useFrmPackager() {
  const [containers, setContainers] = useState<string[]>([]);
  const [loadingContainers, setLoadingContainers] = useState<boolean>(false);

  const [packLoading, setPackLoading] = useState<boolean>(false);
  const [packError, setPackError] = useState<string | null>(null);
  const [packResult, setPackResult] = useState<FrmPackReport | null>(null);

  const [unpackLoading, setUnpackLoading] = useState<boolean>(false);
  const [unpackError, setUnpackError] = useState<string | null>(null);
  const [unpackResult, setUnpackResult] = useState<ExtrheadUnpackReport | null>(null);

  const refreshContainers = useCallback(async () => {
    setLoadingContainers(true);
    try {
      const list = await invokeCommand<string[]>('list_frm_containers_command');
      setContainers(Array.isArray(list) ? list : []);
    } catch (err) {
      logger.warn('FrmPackager', 'Failed listing .FRM containers', err);
    } finally {
      setLoadingContainers(false);
    }
  }, []);

  useEffect(() => {
    void refreshContainers();
  }, [refreshContainers]);

  const packFrm = useCallback(
    async (request: PackFrmRequest): Promise<FrmPackReport | null> => {
      setPackLoading(true);
      setPackError(null);
      try {
        const report = await invokeCommand<FrmPackReport>('pack_frm_command', { request });
        setPackResult(report);
        await refreshContainers();
        return report;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setPackError(msg);
        return null;
      } finally {
        setPackLoading(false);
      }
    },
    [refreshContainers]
  );

  const unpackExtrhead = useCallback(
    async ({ inputPath, mode, outPath }: UnpackExtrheadParams): Promise<ExtrheadUnpackReport | null> => {
      setUnpackLoading(true);
      setUnpackError(null);
      try {
        const trimmedOut = outPath?.trim();
        const report = await invokeCommand<ExtrheadUnpackReport>('unpack_extrhead_command', {
          input_path: inputPath.trim(),
          inputPath: inputPath.trim(),
          mode,
          out_path: trimmedOut ? trimmedOut : null,
          outPath: trimmedOut ? trimmedOut : null,
        });
        setUnpackResult(report);
        return report;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setUnpackError(msg);
        return null;
      } finally {
        setUnpackLoading(false);
      }
    },
    []
  );

  return {
    containers,
    loadingContainers,
    refreshContainers,
    packLoading,
    packError,
    packResult,
    packFrm,
    unpackLoading,
    unpackError,
    unpackResult,
    unpackExtrhead,
  };
}
