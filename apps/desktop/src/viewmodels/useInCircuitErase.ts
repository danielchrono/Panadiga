import { useState, useCallback } from 'react';
import { InCircuitNvramEraseResult } from '@/models';
import { invokeCommand } from '@/services/ipc';

export interface UseInCircuitEraseOptions {
  onLog?: (level: 'info' | 'warn' | 'error' | 'success' | 'rx', source: 'HOST' | 'PICO', message: string) => void;
  onSuccess?: () => void;
}

/**
 * Single Responsibility: Manages in-circuit RAM loader injection and NVRAM sector erasure lifecycle.
 */
export function useInCircuitErase({ onLog, onSuccess }: UseInCircuitEraseOptions = {}) {
  const [isErasing, setIsErasing] = useState<boolean>(false);
  const [eraseResult, setEraseResult] = useState<InCircuitNvramEraseResult | null>(null);

  const injectRamLoaderAndErase = useCallback(async () => {
    setIsErasing(true);
    onLog?.('info', 'HOST', 'Disparando sequência cirúrgica: Injeção de RAM Loader + Apagamento de NVRAM In-Circuit...');
    try {
      const result = await invokeCommand<InCircuitNvramEraseResult>('pico_inject_ram_loader');
      setEraseResult(result);
      result.log_lines.forEach((line, index) => {
        setTimeout(() => {
          onLog?.('rx', 'PICO', line);
        }, index * 80);
      });
      onSuccess?.();
    } catch (err) {
      onLog?.('error', 'HOST', `Falha no procedimento in-circuit: ${err}`);
    } finally {
      setIsErasing(false);
    }
  }, [onLog, onSuccess]);

  return {
    isErasing,
    eraseResult,
    injectRamLoaderAndErase,
  };
}
