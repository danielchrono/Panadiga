import { useState, useEffect, useCallback, useRef } from 'react';
import {
  PicoDeviceStatus,
  SerialLogEntry,
  InCircuitNvramEraseResult,
  ColdBreakinMode,
  GdbBridgeStatus,
  Am34RegisterState,
} from '@/models';
import { invokeCommand } from '@/services/ipc';
import { mockInitialSerialLogs } from '@/services/mockData';

export function usePicoBenchViewModel() {
  const [deviceStatus, setDeviceStatus] = useState<PicoDeviceStatus | null>(null);
  const [availablePorts, setAvailablePorts] = useState<string[]>([
    'MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]',
    'MOCK_RP2350 [RP2350 Pico 2 DWire32A Bench Simulator - 520KB SRAM]',
  ]);
  const [selectedPort, setSelectedPort] = useState<string>('MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]');
  const [logs, setLogs] = useState<SerialLogEntry[]>(mockInitialSerialLogs);
  const [isErasing, setIsErasing] = useState<boolean>(false);
  const [eraseResult, setEraseResult] = useState<InCircuitNvramEraseResult | null>(null);
  const [commandInput, setCommandInput] = useState<string>('');
  const [gdbStatus, setGdbStatus] = useState<GdbBridgeStatus | null>(null);
  const [am34Registers, setAm34Registers] = useState<Am34RegisterState | null>(null);
  const [isGdbStarting, setIsGdbStarting] = useState<boolean>(false);
  const logContainerRef = useRef<HTMLDivElement>(null);

  // Load real system serial ports dynamically
  const refreshPorts = useCallback(async () => {
    try {
      const ports = await invokeCommand<string[]>('list_pico_serial_ports');
      if (ports && ports.length > 0) {
        setAvailablePorts(ports);
        if (!ports.includes(selectedPort)) {
          setSelectedPort(ports[0]);
        }
      }
    } catch (err) {
      console.warn('Failed loading system serial ports:', err);
    }
  }, [selectedPort]);

  // Poll device status and GDB bridge state
  const pollStatus = useCallback(async () => {
    try {
      const status = await invokeCommand<PicoDeviceStatus>('pico_get_status');
      setDeviceStatus(status);
      const gdb = await invokeCommand<GdbBridgeStatus>('get_gdb_bridge_status_command');
      setGdbStatus(gdb);
      if (gdb?.running) {
        const regs = await invokeCommand<Am34RegisterState>('get_am34_registers_command');
        setAm34Registers(regs);
      }
    } catch (err) {
      console.error('Failed querying Pico telemetry:', err);
    }
  }, []);

  useEffect(() => {
    refreshPorts();
    pollStatus();
    const interval = setInterval(pollStatus, 2500);
    return () => clearInterval(interval);
  }, [pollStatus, refreshPorts]);

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

  const startGdbServer = useCallback(async (listenAddr: string = '127.0.0.1:3333') => {
    setIsGdbStarting(true);
    addLog('info', 'HOST', `Iniciando Servidor Ponte GDB RSP em ${listenAddr} (Porta: ${selectedPort})...`);
    try {
      const status = await invokeCommand<GdbBridgeStatus>('start_gdb_bridge_command', {
        port_name: selectedPort,
        listen_addr: listenAddr,
      });
      setGdbStatus(status);
      addLog('success', 'HOST', `Ponte GDB RSP ativa! Conecte seu depurador com: (gdb) target remote ${listenAddr}`);
    } catch (err) {
      addLog('error', 'HOST', `Falha ao iniciar ponte GDB: ${err}`);
    } finally {
      setIsGdbStarting(false);
    }
  }, [selectedPort, addLog]);

  const stopGdbServer = useCallback(async () => {
    addLog('warn', 'HOST', 'Finalizando Servidor Ponte GDB RSP...');
    try {
      const status = await invokeCommand<GdbBridgeStatus>('stop_gdb_bridge_command');
      setGdbStatus(status);
      addLog('info', 'HOST', 'Servidor Ponte GDB desativado.');
    } catch (err) {
      addLog('error', 'HOST', `Falha ao parar ponte GDB: ${err}`);
    }
  }, [addLog]);

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
    gdbStatus,
    am34Registers,
    isGdbStarting,
    startGdbServer,
    stopGdbServer,
    refreshPorts,
    changeBreakinMode,
    toggleConnection,
    injectRamLoaderAndErase,
    sendCommand,
    clearLogs,
    refreshStatus: pollStatus,
  };
}
