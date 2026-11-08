import { useState, useCallback } from 'react';
import { InCircuitNvramEraseResult } from '@/models';
import { invokeCommand } from '@/services/ipc';

export interface UseInCircuitEraseOptions {
  onLog?: (level: 'info' | 'warn' | 'error' | 'success' | 'rx', source: 'HOST' | 'PICO', message: string) => void;
  onSuccess?: () => void;
}

/**
 * Closed algebraic state machine for In-Circuit Erasure.
 * Eliminates illegal states at compile time (ADTs / Discriminated Union).
 */
export type InCircuitEraseState =
  | { status: 'idle' }
  | { status: 'erasing' }
  | { status: 'success'; result: InCircuitNvramEraseResult }
  | { status: 'error'; error: string };

/**
 * Single Responsibility: Manages in-circuit RAM loader injection and NVRAM sector erasure lifecycle.
 */
export function useInCircuitErase({ onLog, onSuccess }: UseInCircuitEraseOptions = {}) {
  const [state, setState] = useState<InCircuitEraseState>({ status: 'idle' });

  const isErasing = state.status === 'erasing';
  const eraseResult = state.status === 'success' ? state.result : null;
  const eraseError = state.status === 'error' ? state.error : null;

  const injectRamLoaderAndErase = useCallback(async () => {
    setState({ status: 'erasing' });
    onLog?.('info', 'HOST', 'Disparando sequência cirúrgica: Injeção de RAM Loader + Apagamento de NVRAM In-Circuit...');
    try {
      const result = await invokeCommand<InCircuitNvramEraseResult>('pico_inject_ram_loader');
      setState({ status: 'success', result });
      result.log_lines.forEach((line, index) => {
        setTimeout(() => {
          onLog?.('rx', 'PICO', line);
        }, index * 80);
      });
      onSuccess?.();
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setState({ status: 'error', error: errMsg });
      onLog?.('error', 'HOST', `Falha no procedimento in-circuit: ${errMsg}`);
    }
  }, [onLog, onSuccess]);

  return {
    state,
    isErasing,
    eraseResult,
    eraseError,
    injectRamLoaderAndErase,
  };
}
