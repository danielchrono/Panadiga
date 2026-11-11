import { useState, useCallback } from 'react';
import { InCircuitNvramEraseResult, FirstContactBackupResult } from '@/models';
import { invokeCommand } from '@/services/ipc';

export interface UseInCircuitEraseOptions {
  selectedPort?: string;
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
 * Single Responsibility: Manages in-circuit RAM loader injection, first-contact backup,
 * and NVRAM sector erasure lifecycle.
 */
export function useInCircuitErase({ selectedPort, onLog, onSuccess }: UseInCircuitEraseOptions = {}) {
  const [state, setState] = useState<InCircuitEraseState>({ status: 'idle' });
  const [backupState, setBackupState] = useState<{
    isBackingUp: boolean;
    result: FirstContactBackupResult | null;
    error: string | null;
  }>({
    isBackingUp: false,
    result: null,
    error: null,
  });

  const isErasing = state.status === 'erasing';
  const eraseResult = state.status === 'success' ? state.result : null;
  const eraseError = state.status === 'error' ? state.error : null;

  const executeFirstContactBackup = useCallback(async () => {
    setBackupState({ isBackingUp: true, result: null, error: null });
    onLog?.('info', 'HOST', 'Iniciando Procedimento de 1º Contato (Backup Read-Only & Verificação de Integridade)...');
    try {
      const result = await invokeCommand<FirstContactBackupResult>('pico_first_contact_backup', {
        port_name: selectedPort,
      });
      setBackupState({ isBackingUp: false, result, error: null });
      result.log_lines.forEach((line: string, index: number) => {
        setTimeout(() => {
          onLog?.('rx', 'PICO', line);
        }, index * 80);
      });
      onLog?.('success', 'HOST', `Backup de 1º Contato concluído! Modelo: ${result.detected_model}. NVRAM: ${result.nvram_bytes}B.`);
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setBackupState({ isBackingUp: false, result: null, error: errMsg });
      onLog?.('error', 'HOST', `Falha no backup de 1º contato: ${errMsg}`);
    }
  }, [selectedPort, onLog]);

  const injectRamLoaderAndErase = useCallback(async () => {
    setState({ status: 'erasing' });
    onLog?.('info', 'HOST', 'Disparando sequência cirúrgica: Injeção de RAM Loader + Apagamento de NVRAM In-Circuit...');
    try {
      const result = await invokeCommand<InCircuitNvramEraseResult>('pico_inject_ram_loader', {
        port_name: selectedPort,
      });
      setState({ status: 'success', result });
      result.log_lines.forEach((line: string, index: number) => {
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
  }, [selectedPort, onLog, onSuccess]);

  return {
    state,
    isErasing,
    eraseResult,
    eraseError,
    injectRamLoaderAndErase,
    isBackingUp: backupState.isBackingUp,
    backupResult: backupState.result,
    backupError: backupState.error,
    executeFirstContactBackup,
  };
}
