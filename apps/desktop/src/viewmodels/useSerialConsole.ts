import { useState, useCallback, useRef, useEffect } from 'react';
import { SerialLogEntry } from '@/models';
import { invokeCommand, isDemoModeActive, isTauriEnvironment } from '@/services/ipc';
import { bleService } from '@/services/bleService';

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
 * Single Responsibility: Manages interactive serial CDC console logs and command dispatching.
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
      setLogs(prev => [...prev, newEntry]);
    },
    []
  );

  // Subscribe to incoming BLE notifications when active
  useEffect(() => {
    const unsubscribe = bleService.onReceive(data => {
      addLog('rx', 'PICO', `[BLE] ${data.trim()}`);
    });
    return unsubscribe;
  }, [addLog]);

  const sendCommand = useCallback(async (overridePort?: string) => {
    if (!commandInput.trim()) return;
    const cmd = commandInput.trim().toUpperCase();
    setConsoleState({ status: 'sending', command: cmd });
    addLog('tx', 'HOST', `> ${cmd}`);
    setCommandInput('');

    // If BLE link is active, transmit wirelessly over Nordic UART Service
    if (bleService.isConnected()) {
      try {
        await bleService.send(cmd);
      } catch (err) {
        addLog('error', 'HOST', `Falha ao transmitir via BLE: ${err}`);
      } finally {
        setConsoleState({ status: 'idle' });
      }
      return;
    }

    if (isTauriEnvironment() && !isDemoModeActive()) {
      try {
        const targetPort = overridePort || selectedPort;
        const response = await invokeCommand<string>('pico_send_serial_command', {
          portName: targetPort,
          port_name: targetPort,
          command: cmd,
        });
        const source = cmd === 'RESET' ? 'TARGET' : 'PICO';
        addLog('rx', source, response);
      } catch (err) {
        addLog(
          'error',
          'HOST',
          `Erro CDC: ${err instanceof Error ? err.message : String(err)}`
        );
      } finally {
        setConsoleState({ status: 'idle' });
      }
      return;
    }

    setTimeout(() => {
      if (cmd === 'STATUS' || cmd === 'VDD') {
        addLog('rx', 'PICO', 'VDD=3.28V | OCD=HALTED | PC=0x80000000 | MODE=2');
      } else if (cmd === 'RESET') {
        addLog('rx', 'TARGET', 'D_RESET pulsed low for 10ms. Target reset asserted.');
      } else if (cmd === 'ADC') {
        addLog('rx', 'PICO', 'ADC0 Raw: 4072 (3.281 V). Rail within tolerance (+-5%).');
      } else if (cmd === 'GDB') {
        addLog('rx', 'PICO', 'GDB Remote Serial Protocol mode active.');
      } else {
        addLog('rx', 'PICO', `ACK: ${cmd} (Instrução executada no PIO0 @ 0x00004000)`);
      }
      setConsoleState({ status: 'idle' });
    }, 120);
  }, [commandInput, selectedPort, addLog]);

  const clearLogs = useCallback(() => {
    setLogs([]);
  }, []);

  return {
    logs,
    consoleState,
    addLog,
    clearLogs,
    commandInput,
    setCommandInput,
    sendCommand,
    logContainerRef,
  };
}
