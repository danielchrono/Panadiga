import { useCallback, useEffect, useState } from 'react';
import type {
  PicoDiagnosticResult,
  Sc0BridgeDataEvent,
  Sc0BridgeInputMode,
  Sc0BridgeStatus,
  SerialLogEntry,
} from '@/models';
import { invokeCommand, listenEvent } from '@/services/ipc';

export interface BridgeStreamLine extends Sc0BridgeDataEvent {
  readonly id: string;
}

export interface UseSc0BridgeOptions {
  selectedPort: string;
  onExecuteDiagnostic: (commandId: string, arg?: string) => Promise<PicoDiagnosticResult | null>;
  onLog?: (
    level: SerialLogEntry['level'],
    source: SerialLogEntry['source'],
    message: string
  ) => void;
}

const DEFAULT_BRIDGE_STATUS: Sc0BridgeStatus = {
  running: false,
  port_name: '',
  baud_rate: 115200,
  boot_mode: 'JIG:350',
  bytes_rx: 0,
  bytes_tx: 0,
  last_event: 'Standby',
};

const MAX_BRIDGE_LINES = 150;

function formatLogId(prefix: string): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
}

/**
 * MVVM ViewModel for Interactive SC0 Bridge (`BRIDGE`, exit `'~~~'`),
 * `WAITTX` triggers, `PANA_RSP` quick probes (`0x15`, `0E0402`, `$g#67`, `$Z0#8A`),
 * and optical drive serial commands (`PASS`, `START`, `LDD`, `DSCREG`, `FEP`).
 */
export function useSc0Bridge({
  selectedPort,
  onExecuteDiagnostic,
  onLog,
}: UseSc0BridgeOptions) {
  const [bridgeStatus, setBridgeStatus] = useState<Sc0BridgeStatus>(DEFAULT_BRIDGE_STATUS);
  const [bridgeLogs, setBridgeLogs] = useState<BridgeStreamLine[]>([]);
  const [baudRate, setBaudRate] = useState<string>('115200');
  const [bootMode, setBootMode] = useState<string>('JIG:350');
  const [inputMode, setInputMode] = useState<Sc0BridgeInputMode>('line');
  const [txInput, setTxInput] = useState<string>('');
  const [isBusy, setIsBusy] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listenEvent<Sc0BridgeDataEvent>('pico://bridge-data', chunk => {
      if (!chunk) return;
      const line: BridgeStreamLine = { ...chunk, id: formatLogId(chunk.stream) };
      setBridgeLogs(prev => [...prev, line].slice(-MAX_BRIDGE_LINES));
      if (chunk.stream === 'rx') {
        setBridgeStatus(prev => ({ ...prev, bytes_rx: prev.bytes_rx + chunk.bytes.length }));
      } else if (chunk.stream === 'tx') {
        setBridgeStatus(prev => ({ ...prev, bytes_tx: prev.bytes_tx + chunk.bytes.length }));
      } else if (chunk.text.includes('finalizada') || chunk.text.includes('BRIDGE: OK')) {
        setBridgeStatus(prev => ({ ...prev, running: false, last_event: chunk.text }));
      }
    }).then(fn => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  const refreshBridgeStatus = useCallback(async (): Promise<void> => {
    try {
      const status = await invokeCommand<Sc0BridgeStatus>('pico_get_bridge_status');
      if (status && typeof status.running === 'boolean') {
        setBridgeStatus(status);
      }
    } catch {
      /* Ignore transient poll errors in browser/test harness */
    }
  }, []);

  const startBridge = useCallback(async (): Promise<void> => {
    if (!selectedPort) {
      setError('Selecione uma porta serial COM antes de iniciar a SC0 Bridge.');
      return;
    }
    setIsBusy(true);
    setError(null);
    try {
      const parsedBaud = Number.parseInt(baudRate, 10) || 115200;
      const status = await invokeCommand<Sc0BridgeStatus>('pico_start_sc0_bridge', {
        portName: selectedPort,
        baudRate: parsedBaud,
        bootMode,
      });
      setBridgeStatus(status);
      onLog?.('info', 'HOST', `SC0 BRIDGE ${parsedBaud} 8N1 iniciada (${bootMode})`);
    } catch (err) {
      const msg = String(err);
      setError(msg);
      onLog?.('error', 'HOST', `Falha SC0 BRIDGE: ${msg}`);
    } finally {
      setIsBusy(false);
    }
  }, [selectedPort, baudRate, bootMode, onLog]);

  const stopBridge = useCallback(async (): Promise<void> => {
    setIsBusy(true);
    setError(null);
    try {
      const status = await invokeCommand<Sc0BridgeStatus>('pico_stop_sc0_bridge');
      setBridgeStatus(status);
      onLog?.('info', 'HOST', "SC0 BRIDGE encerrada enviando '~~~'");
    } catch (err) {
      const msg = String(err);
      setError(msg);
      onLog?.('error', 'HOST', `Erro ao encerrar SC0 BRIDGE: ${msg}`);
    } finally {
      setIsBusy(false);
    }
  }, [onLog]);

  const sendBridgePayload = useCallback(
    async (mode: Sc0BridgeInputMode, payload: string): Promise<void> => {
      if (!payload.trim()) return;
      setError(null);
      try {
        await invokeCommand<number>('pico_send_bridge_input', {
          inputMode: mode,
          payload,
        });
        await refreshBridgeStatus();
      } catch (err) {
        setError(String(err));
      }
    },
    [refreshBridgeStatus]
  );

  const submitTxInput = useCallback(async (): Promise<void> => {
    const trimmed = txInput.trim();
    if (!trimmed) return;
    await sendBridgePayload(inputMode, trimmed);
    setTxInput('');
  }, [txInput, inputMode, sendBridgePayload]);

  const clearBridgeLogs = useCallback((): void => {
    setBridgeLogs([]);
    setError(null);
  }, []);

  const triggerInteractiveWaittx = useCallback(
    async (spec: string): Promise<void> => {
      setError(null);
      await onExecuteDiagnostic('WAITTX', spec);
    },
    [onExecuteDiagnostic]
  );

  const sendPanaRspPreset = useCallback(
    async (
      mode: Sc0BridgeInputMode,
      bridgePayload: string,
      standalonePanarspArg: string
    ): Promise<void> => {
      if (bridgeStatus.running) {
        await sendBridgePayload(mode, bridgePayload);
      } else {
        await onExecuteDiagnostic('PANARSP', standalonePanarspArg);
      }
    },
    [bridgeStatus.running, sendBridgePayload, onExecuteDiagnostic]
  );

  const sendDriveCommand = useCallback(
    async (driveCmd: string): Promise<void> => {
      if (bridgeStatus.running) {
        await sendBridgePayload('line', driveCmd);
      } else {
        await onExecuteDiagnostic('UARTSEND', driveCmd);
      }
    },
    [bridgeStatus.running, sendBridgePayload, onExecuteDiagnostic]
  );

  return {
    bridgeStatus,
    bridgeLogs,
    baudRate,
    setBaudRate,
    bootMode,
    setBootMode,
    inputMode,
    setInputMode,
    txInput,
    setTxInput,
    isBusy,
    error,
    startBridge,
    stopBridge,
    submitTxInput,
    clearBridgeLogs,
    triggerInteractiveWaittx,
    sendPanaRspPreset,
    sendDriveCommand,
  };
}
