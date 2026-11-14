import { useCallback, useEffect, useState } from 'react';
import type {
  HardwareSsotSpec,
  PicoCommandCategory,
  PicoDiagnosticResult,
  SerialLogEntry,
} from '@/models';
import { invokeCommand, listenEvent } from '@/services/ipc';
import { bleService } from '@/services/bleService';
import { sendAndroidUsbOtgCommand } from '@/services/androidBridge';
import { parseDiagnosticOutput } from '@/services/diagnosticParser';
import { ingestP7501DumpOutput } from '@/services/liveNvramCaptureStore';

export interface UsePicoDiagnosticsOptions {
  selectedPort: string;
  ssot: HardwareSsotSpec | null;
  onLog?: (
    level: SerialLogEntry['level'],
    source: SerialLogEntry['source'],
    message: string
  ) => void;
  onTelemetryUpdate?: (result: PicoDiagnosticResult) => void;
}

/**
 * Granular Diagnostic Studio ViewModel:
 * Executes any firmware CLI command (`PINCHECK`, `BUSDIAG`, `SNIFF`, `DUMP`, `CFI`, `BOOTCAP`, `FMU*`)
 * transparently across Desktop USB CDC, Android USB-C OTG, and Bluetooth LE NUS.
 */
export function usePicoDiagnostics({
  selectedPort,
  ssot,
  onLog,
  onTelemetryUpdate,
}: UsePicoDiagnosticsOptions) {
  const [history, setHistory] = useState<PicoDiagnosticResult[]>([]);
  const [latestResult, setLatestResult] = useState<PicoDiagnosticResult | null>(null);
  const [runningCommandId, setRunningCommandId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<PicoCommandCategory>('electrical');

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listenEvent<PicoDiagnosticResult>('pico://diagnostic-result', payload => {
      if (!payload || !payload.command_id) return;
      setLatestResult(payload);
      setHistory(prev => [payload, ...prev].slice(0, 30));
      onTelemetryUpdate?.(payload);
    }).then(fn => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [onTelemetryUpdate]);

  const commitDiagnosticResult = useCallback(
    (result: PicoDiagnosticResult, logReply = false): PicoDiagnosticResult => {
      setLatestResult(result);
      setHistory(prev => {
        if (prev[0]?.raw_command === result.raw_command && prev[0]?.elapsed_ms === result.elapsed_ms) {
          return prev;
        }
        return [result, ...prev].slice(0, 30);
      });
      if (logReply) {
        onLog?.(result.success ? 'rx' : 'error', 'PICO', result.summary);
      }
      onTelemetryUpdate?.(result);
      return result;
    },
    [onLog, onTelemetryUpdate]
  );

  const executeCommand = useCallback(
    async (commandId: string, arg?: string): Promise<PicoDiagnosticResult | null> => {
      const norm = commandId.trim().toUpperCase();
      const spec = ssot?.pico_commands.find(c => c.id === norm);
      const cleanArg = arg?.trim() ? arg.trim() : undefined;
      const rawCommand = cleanArg ? `${norm} ${cleanArg}` : norm;
      const timeoutMs = spec?.timeout_ms ?? 4000;

      setRunningCommandId(norm);
      setError(null);

      try {
        const executeDirect = async (
          transportTag: string,
          runTransport: () => Promise<{ ok: boolean; output: string }>
        ): Promise<PicoDiagnosticResult> => {
          onLog?.('tx', 'HOST', `> [${transportTag}] ${rawCommand}`);
          const started = performance.now();
          const reply = await runTransport();
          const elapsed = Math.round(performance.now() - started);
          const parsed = parseDiagnosticOutput(norm, rawCommand, reply.output, elapsed);
          if (!reply.ok) parsed.success = false;
          if (parsed.success && norm === 'DUMP') {
            void ingestP7501DumpOutput(reply.output, transportTag, selectedPort).catch(() => {});
          }
          return commitDiagnosticResult(parsed, true);
        };

        if (selectedPort.startsWith('ANDROID-USB-OTG')) {
          return await executeDirect('USB-OTG', async () =>
            sendAndroidUsbOtgCommand(rawCommand, timeoutMs)
          );
        }

        if (selectedPort.startsWith('BLE_') && bleService.isConnected()) {
          return await executeDirect('BLE NUS', async () => ({
            ok: true,
            output: await bleService.sendAndAwaitReply(rawCommand, timeoutMs),
          }));
        }

        if (!selectedPort) {
          const msg =
            'Selecione ou conecte uma sonda Pico real antes de executar diagnósticos P7501 UART / OCD.';
          setError(msg);
          throw new Error(msg);
        }
        const portToUse = selectedPort;
        const result = await invokeCommand<PicoDiagnosticResult>('pico_execute_diagnostic', {
          portName: portToUse,
          port_name: portToUse,
          commandId: norm,
          command_id: norm,
          arg: cleanArg ?? null,
        });
        if (norm === 'DUMP' && result.success && result.raw_output) {
          await ingestP7501DumpOutput(result.raw_output, 'DESKTOP_CDC', portToUse);
        }
        return commitDiagnosticResult(result);
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setError(msg);
        onLog?.('error', 'HOST', `Falha em ${rawCommand}: ${msg}`);
        return null;
      } finally {
        setRunningCommandId(null);
      }
    },
    [selectedPort, ssot?.pico_commands, onLog, commitDiagnosticResult]
  );

  const clearHistory = useCallback(() => {
    setHistory([]);
    setLatestResult(null);
    setError(null);
  }, []);

  return {
    history,
    latestResult,
    runningCommandId,
    isExecuting: runningCommandId !== null,
    isLoading: runningCommandId !== null,
    error,
    selectedCategory,
    setSelectedCategory,
    executeCommand,
    clearHistory,
  };
}
