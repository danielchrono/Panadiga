import { useState, useCallback, useRef } from 'react';
import { SerialLogEntry } from '@/models';
import { mockInitialSerialLogs } from '@/services/mockData';

/**
 * Single Responsibility: Manages interactive serial CDC console logs and command dispatching.
 */
export function useSerialConsole() {
  const [logs, setLogs] = useState<SerialLogEntry[]>(mockInitialSerialLogs);
  const [commandInput, setCommandInput] = useState<string>('');
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

  const sendCommand = useCallback(() => {
    if (!commandInput.trim()) return;
    const cmd = commandInput.trim().toUpperCase();
    addLog('tx', 'HOST', `> ${cmd}`);
    setCommandInput('');

    setTimeout(() => {
      if (cmd === 'STATUS') {
        addLog('rx', 'PICO', 'VDD=3.28V | OCD=HALTED | PC=0x80000000 | MODE=2');
      } else if (cmd === 'RESET') {
        addLog('rx', 'TARGET', 'D_RESET pulsed low for 10ms. Target reset asserted.');
      } else if (cmd === 'ADC') {
        addLog('rx', 'PICO', 'ADC0 Raw: 4072 (3.281 V). Rail within tolerance (+-5%).');
      } else if (cmd === 'GDB') {
        addLog('rx', 'PICO', 'GDB Remote Serial Protocol mode active.');
      } else {
        addLog('rx', 'PICO', `ACK: ${cmd} (Instrução executada no PIO0)`);
      }
    }, 120);
  }, [commandInput, addLog]);

  const clearLogs = useCallback(() => {
    setLogs([]);
  }, []);

  return {
    logs,
    addLog,
    clearLogs,
    commandInput,
    setCommandInput,
    sendCommand,
    logContainerRef,
  };
}
