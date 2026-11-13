import { useState, useCallback, useRef, useEffect } from 'react';
import type { ReactiveSerialEvent, SerialLogEntry } from '@/models';
import { invokeCommand, isTauriEnvironment, listenEvent } from '@/services/ipc';
import { bleService } from '@/services/bleService';
import { sendAndroidUsbOtgCommand } from '@/services/androidBridge';
import { parseDiagnosticOutput } from '@/services/diagnosticParser';
import { ingestP7501DumpOutput } from '@/services/liveNvramCaptureStore';

/**
 * Closed algebraic state machine for Serial Console transmission lifecycle.
 */
export type SerialConsoleState =
  | { status: 'idle' }
  | { status: 'sending'; command: string };

export interface UseSerialConsoleOptions {
  selectedPort?: string;
  initialLogs?: SerialLogEntry[];
}

/**
 * Single Responsibility: Manages interactive serial CDC / BLE / USB-OTG console logs
 * and reactive `pico://serial-log` streaming events.
 */
export function useSerialConsole({ selectedPort, initialLogs = [] }: UseSerialConsoleOptions = {}) {
  const [logs, setLogs] = useState<SerialLogEntry[]>(initialLogs);
  const [commandInput, setCommandInput] = useState<string>('');
  const [consoleState, setConsoleState] = useState<SerialConsoleState>({ status: 'idle' });
  const logContainerRef = useRef<HTMLDivElement>(null);

  const addLog = useCallback(
    (
      level: SerialLogEntry['level'],
      source: SerialLogEntry['source'],
      message: string,
      hexData?: string
    ) => {
      const newEntry: SerialLogEntry = {
        id: Math.random().toString(36).substring(2, 9),
        timestamp:
          new Date().toTimeString().split(' ')[0] +
          '.' +
          Math.floor(Math.random() * 900 + 100),
        level,
        source,
        message,
        hexData,
      };
      setLogs(prev => [...prev.slice(-240), newEntry]);
    },
    []
  );

  // Subscribe to incoming BLE notifications when active
  useEffect(() => {
    const unsubscribe = bleService.onReceive(data => {
      const clean = data.trim();
      if (clean) {
        addLog('rx', 'PICO', `[BLE] ${clean}`);
      }
    });
    return unsubscribe;
  }, [addLog]);

  // Subscribe to reactive backend diagnostic log pushes
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listenEvent<ReactiveSerialEvent>('pico://serial-log', evt => {
      if (!evt?.message) return;
      setLogs(prev => {
        if (prev.some(item => item.id === evt.id)) return prev;
        return [
          ...prev.slice(-240),
          {
            id: evt.id || Math.random().toString(36).substring(2, 9),
            timestamp: evt.timestamp || new Date().toTimeString().split(' ')[0],
            level: evt.level || 'info',
            source: evt.source || 'PICO',
            message: evt.message,
          },
        ];
      });
    }).then(fn => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  const sendCommand = useCallback(
    async (overridePort?: string) => {
      if (!commandInput.trim()) return;
      const cmd = commandInput.trim().toUpperCase();
      const targetPort = overridePort || selectedPort || '';
      setConsoleState({ status: 'sending', command: cmd });
      addLog('tx', 'HOST', `> ${cmd}`);
      setCommandInput('');

      if (targetPort.startsWith('ANDROID-USB-OTG')) {
        try {
          const res = sendAndroidUsbOtgCommand(cmd, 4000);
          addLog(res.ok ? 'rx' : 'error', 'PICO', res.output);
          if (res.ok && cmd.startsWith('DUMP')) {
            void ingestP7501DumpOutput(res.output, 'ANDROID_USB_OTG', targetPort).catch(() => {});
          }
        } catch (err) {
          addLog('error', 'HOST', `Erro USB-OTG: ${err instanceof Error ? err.message : String(err)}`);
        } finally {
          setConsoleState({ status: 'idle' });
        }
        return;
      }

      if (bleService.isConnected()) {
        try {
          if (cmd.startsWith('DUMP')) {
            const reply = await bleService.sendAndAwaitReply(cmd, 5000);
            addLog('rx', 'PICO', reply);
            void ingestP7501DumpOutput(reply, 'BLE_NUS', targetPort || 'BLE_PICO_W').catch(() => {});
          } else {
            await bleService.send(cmd);
          }
        } catch (err) {
          addLog('error', 'HOST', `Falha ao transmitir via BLE: ${err}`);
        } finally {
          setConsoleState({ status: 'idle' });
        }
        return;
      }

      if (!targetPort || targetPort.startsWith('MOCK')) {
        addLog('error', 'HOST', 'Nenhuma porta Pico USB CDC / OTG / BLE selecionada para transmissão.');
        setConsoleState({ status: 'idle' });
        return;
      }

      if (!isTauriEnvironment() && typeof window !== 'undefined' && !window.__PANADIGA_TEST_IPC__) {
        addLog('error', 'HOST', 'Runtime nativo Tauri indisponível para acesso serial ao Pico.');
        setConsoleState({ status: 'idle' });
        return;
      }

      try {
        const response = await invokeCommand<string>('pico_send_serial_command', {
          portName: targetPort,
          port_name: targetPort,
          command: cmd,
        });
        const source = cmd === 'RESET' ? 'TARGET' : 'PICO';
        addLog('rx', source, response);
        if (cmd.startsWith('DUMP')) {
          void ingestP7501DumpOutput(response, 'USB_CDC', targetPort).catch(() => {});
        }
      } catch (err) {
        addLog(
          'error',
          'HOST',
          `Erro CDC: ${err instanceof Error ? err.message : String(err)}`
        );
      } finally {
        setConsoleState({ status: 'idle' });
      }
    },
    [commandInput, selectedPort, addLog]
  );

  const clearLogs = useCallback(() => {
    setLogs([]);
  }, []);

  return {
    logs,
    consoleState,
    isLoading: consoleState.status === 'sending',
    addLog,
    clearLogs,
    commandInput,
    setCommandInput,
    sendCommand,
    logContainerRef,
  };
}
