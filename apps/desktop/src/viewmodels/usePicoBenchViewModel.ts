import { useState, useEffect, useCallback, useRef } from 'react';
import {
  PicoDeviceStatus,
  SerialLogEntry,
  InCircuitNvramEraseResult,
  ColdBreakinMode,
} from '@/models';
import { invokeCommand } from '@/services/ipc';
import { mockInitialSerialLogs } from '@/services/mockData';

export function usePicoBenchViewModel() {
  const [deviceStatus, setDeviceStatus] = useState<PicoDeviceStatus | null>(null);
  const [availablePorts] = useState<string[]>([
    'COM4 (Raspberry Pi Pico RP2040 DWire32A)',
    'COM3 (Silicon Labs CP210x USB to UART)',
    'COM1 (Standard Communications Port)',
  ]);
  const [selectedPort, setSelectedPort] = useState<string>('COM4 (Raspberry Pi Pico RP2040 DWire32A)');
  const [logs, setLogs] = useState<SerialLogEntry[]>(mockInitialSerialLogs);
  const [isErasing, setIsErasing] = useState<boolean>(false);
  const [eraseResult, setEraseResult] = useState<InCircuitNvramEraseResult | null>(null);
  const [commandInput, setCommandInput] = useState<string>('');
  const logContainerRef = useRef<HTMLDivElement>(null);

  // Poll device status
  const pollStatus = useCallback(async () => {
    try {
      const status = await invokeCommand<PicoDeviceStatus>('pico_get_status');
      setDeviceStatus(status);
    } catch (err) {
      console.error('Failed querying Pico status:', err);
    }
  }, []);

  useEffect(() => {
    pollStatus();
    const interval = setInterval(pollStatus, 3000);
    return () => clearInterval(interval);
  }, [pollStatus]);

  const addLog = useCallback((level: SerialLogEntry['level'], source: SerialLogEntry['source'], message: string, hexData?: string) => {
    const newEntry: SerialLogEntry = {
      id: Math.random().toString(36).substring(2, 9),
      timestamp: new Date().toTimeString().split(' ')[0] + '.' + Math.floor(Math.random() * 900 + 100),
      level,
      source,
      message,
      hexData,
    };
    setLogs(prev => [...prev, newEntry]);
  }, []);

  const changeBreakinMode = useCallback(async (mode: ColdBreakinMode) => {
    try {
      const updated = await invokeCommand<PicoDeviceStatus>('pico_set_breakin_mode', { mode });
      setDeviceStatus(updated);
      addLog('info', 'HOST', `Modo de Cold Break-In alterado para: Modo ${mode}`);
    } catch (err) {
      addLog('error', 'HOST', `Erro ao alterar modo: ${err}`);
    }
  }, [addLog]);

  const toggleConnection = useCallback(() => {
    if (!deviceStatus) return;
    const nextConnected = !deviceStatus.connected;
    setDeviceStatus(prev => prev ? { ...prev, connected: nextConnected, core_state: nextConnected ? 'HALTED' : 'DISCONNECTED' } : null);
    addLog(nextConnected ? 'success' : 'warn', 'HOST', nextConnected ? `Conectado com sucesso em ${selectedPort}` : 'Desconectado da interface DWire32A');
  }, [deviceStatus, selectedPort, addLog]);

  const injectRamLoaderAndErase = useCallback(async () => {
    setIsErasing(true);
    addLog('info', 'HOST', 'Disparando sequência cirúrgica: Injeção de RAM Loader + Apagamento de NVRAM In-Circuit...');
    try {
      const result = await invokeCommand<InCircuitNvramEraseResult>('pico_inject_ram_loader');
      setEraseResult(result);
      result.log_lines.forEach((line, index) => {
        setTimeout(() => {
          addLog('rx', 'PICO', line);
        }, index * 100);
      });
      setDeviceStatus(prev => prev ? { ...prev, ram_loader_injected: true } : null);
    } catch (err) {
      addLog('error', 'HOST', `Falha no procedimento in-circuit: ${err}`);
    } finally {
      setIsErasing(false);
    }
  }, [addLog]);

  const sendCommand = useCallback(() => {
    if (!commandInput.trim()) return;
    const cmd = commandInput.trim().toUpperCase();
    addLog('tx', 'HOST', `> ${cmd}`);
    setCommandInput('');

    // Simulate response based on command
    setTimeout(() => {
      if (cmd === 'STATUS') {
        addLog('rx', 'PICO', 'VDD=3.28V | OCD=HALTED | PC=0x80000000 | MODE=2');
      } else if (cmd === 'RESET') {
        addLog('rx', 'TARGET', 'D_RESET pulsed low for 10ms. Target reset asserted.');
      } else if (cmd === 'ADC') {
        addLog('rx', 'PICO', 'ADC0 Raw: 4072 (3.281 V). Rail within tolerance (+-5%).');
      } else {
        addLog('rx', 'PICO', `ACK: ${cmd} (Instrução executada no PIO0)`);
      }
    }, 120);
  }, [commandInput, addLog]);

  const clearLogs = useCallback(() => {
    setLogs([]);
  }, []);

  return {
    deviceStatus,
    availablePorts,
    selectedPort,
    setSelectedPort,
    logs,
    logContainerRef,
    isErasing,
    eraseResult,
    commandInput,
    setCommandInput,
    changeBreakinMode,
    toggleConnection,
    injectRamLoaderAndErase,
    sendCommand,
    clearLogs,
    refreshStatus: pollStatus,
  };
}
