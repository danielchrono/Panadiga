import { useState, useCallback, useEffect } from 'react';
import { PicoDeviceStatus, ColdBreakinMode } from '@/models';
import { invokeCommand } from '@/services/ipc';

export interface UsePicoDeviceOptions {
  onLog?: (level: 'info' | 'warn' | 'error' | 'success', source: 'HOST' | 'PICO', message: string) => void;
}

/**
 * Single Responsibility: Manages Pico USB hardware connection, port enumeration, and break-in modes.
 */
export function usePicoDevice({ onLog }: UsePicoDeviceOptions = {}) {
  const [deviceStatus, setDeviceStatus] = useState<PicoDeviceStatus | null>(null);
  const [availablePorts, setAvailablePorts] = useState<string[]>([
    'MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]',
    'MOCK_RP2350 [RP2350 Pico 2 DWire32A Bench Simulator - 520KB SRAM]',
  ]);
  const [selectedPort, setSelectedPort] = useState<string>(
    'MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]'
  );

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

  const pollDeviceStatus = useCallback(async () => {
    try {
      const status = await invokeCommand<PicoDeviceStatus>('pico_get_status');
      setDeviceStatus(status);
    } catch (err) {
      console.warn('Failed querying Pico device telemetry:', err);
    }
  }, []);

  const changeBreakinMode = useCallback(
    async (mode: ColdBreakinMode) => {
      try {
        const updated = await invokeCommand<PicoDeviceStatus>('pico_set_breakin_mode', { mode });
        setDeviceStatus(updated);
        onLog?.('info', 'HOST', `Modo de Cold Break-In alterado para: Modo ${mode}`);
      } catch (err) {
        onLog?.('error', 'HOST', `Erro ao alterar modo: ${err}`);
      }
    },
    [onLog]
  );

  const toggleConnection = useCallback(() => {
    if (!deviceStatus) return;
    const nextConnected = !deviceStatus.connected;
    setDeviceStatus(prev =>
      prev
        ? {
            ...prev,
            connected: nextConnected,
            core_state: nextConnected ? 'HALTED' : 'DISCONNECTED',
          }
        : null
    );
    onLog?.(
      nextConnected ? 'success' : 'warn',
      'HOST',
      nextConnected
        ? `Conectado com sucesso em ${selectedPort}`
        : 'Desconectado da interface DWire32A'
    );
  }, [deviceStatus, selectedPort, onLog]);

  useEffect(() => {
    refreshPorts();
    pollDeviceStatus();
    const interval = setInterval(pollDeviceStatus, 3000);
    return () => clearInterval(interval);
  }, [refreshPorts, pollDeviceStatus]);

  return {
    deviceStatus,
    setDeviceStatus,
    availablePorts,
    selectedPort,
    setSelectedPort,
    refreshPorts,
    pollDeviceStatus,
    changeBreakinMode,
    toggleConnection,
  };
}
