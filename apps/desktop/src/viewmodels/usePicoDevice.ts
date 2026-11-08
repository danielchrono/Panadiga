import { useState, useCallback, useEffect } from 'react';
import { PicoDeviceStatus, ColdBreakinMode } from '@/models';
import { invokeCommand } from '@/services/ipc';
import { bleService } from '@/services/bleService';
import { logger } from '@/utils/logger';

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
    'BLE_PICO_W [Bluetooth LE - Nordic UART Service (NUS)]',
  ]);
  const [selectedPort, setSelectedPort] = useState<string>(
    'MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]'
  );

  const refreshPorts = useCallback(async () => {
    try {
      const ports = await invokeCommand<string[]>('list_pico_serial_ports');
      const allPorts = [...(ports || [])];
      if (!allPorts.some(p => p.includes('BLE_PICO_W'))) {
        allPorts.push('BLE_PICO_W [Bluetooth LE - Nordic UART Service (NUS)]');
      }
      setAvailablePorts(allPorts);
      if (!allPorts.includes(selectedPort)) {
        setSelectedPort(allPorts[0]);
      }
    } catch (err) {
      logger.warn('PicoDevice', 'Failed loading system serial ports', err);
    }
  }, [selectedPort]);

  const pollDeviceStatus = useCallback(async () => {
    try {
      const status = await invokeCommand<PicoDeviceStatus>('pico_get_status');
      setDeviceStatus(status);
    } catch (err) {
      logger.warn('PicoDevice', 'Failed querying Pico device telemetry', err);
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

  const toggleConnection = useCallback(async () => {
    if (!deviceStatus) return;
    const nextConnected = !deviceStatus.connected;

    if (nextConnected && selectedPort.startsWith('BLE_')) {
      try {
        const bleDev = await bleService.connect();
        onLog?.('success', 'HOST', `Conectado via Bluetooth LE NUS: ${bleDev.name}`);
        setDeviceStatus(prev =>
          prev
            ? {
                ...prev,
                connected: true,
                core_state: 'HALTED',
                port: selectedPort,
              }
            : null
        );
        return;
      } catch (err) {
        onLog?.(
          'error',
          'HOST',
          `Falha ao parear Bluetooth LE: ${err instanceof Error ? err.message : String(err)}`
        );
        return;
      }
    } else if (!nextConnected && bleService.isConnected()) {
      bleService.disconnect();
      onLog?.('warn', 'HOST', 'Desconectado do link Bluetooth LE');
    }

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
