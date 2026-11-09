import { useState, useCallback, useEffect } from 'react';
import { PicoDeviceStatus, ColdBreakinMode, DigaModelProfile } from '@/models';
import { invokeCommand, listenEvent } from '@/services/ipc';
import { bleService } from '@/services/bleService';
import { logger } from '@/utils/logger';

export interface UsePicoDeviceOptions {
  onLog?: (level: 'info' | 'warn' | 'error' | 'success', source: 'HOST' | 'PICO', message: string) => void;
}

/**
 * Single Responsibility: Manages Pico USB hardware connection, port enumeration,
 * reactive telemetry streams, and break-in modes.
 */
export function usePicoDevice({ onLog }: UsePicoDeviceOptions = {}) {
  const [deviceStatus, setDeviceStatus] = useState<PicoDeviceStatus | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [availablePorts, setAvailablePorts] = useState<string[]>([
    'MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]',
    'MOCK_RP2350 [RP2350 Pico 2 DWire32A Bench Simulator - 520KB SRAM]',
    'BLE_PICO_W [Bluetooth LE - Nordic UART Service (NUS)]',
  ]);
  const [selectedPort, setSelectedPort] = useState<string>(
    'MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]'
  );
  const [modelCatalog, setModelCatalog] = useState<DigaModelProfile[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>('AUTO');

  const refreshModelCatalog = useCallback(async () => {
    try {
      const catalog = await invokeCommand<DigaModelProfile[]>('get_diga_model_catalog');
      if (catalog && catalog.length > 0) {
        setModelCatalog(catalog);
      }
    } catch (err) {
      logger.warn('PicoDevice', 'Failed loading DIGA model catalog', err);
    }
  }, []);

  const selectTargetModel = useCallback(
    async (modelId: string) => {
      setSelectedModel(modelId);
      const profile = modelCatalog.find(m => m.model_id === modelId);
      if (modelId === 'AUTO') {
        try {
          const resp = await invokeCommand<string>('pico_send_serial_command', {
            port_name: selectedPort,
            command: 'AUTODETECT',
          });
          onLog?.('info', 'PICO', `Modo AUTO ativado (Address Aliasing Probe): ${resp}`);
        } catch (err) {
          logger.warn('PicoDevice', 'AUTODETECT command failed', err);
        }
      } else if (profile) {
        try {
          const resp = await invokeCommand<string>('pico_send_serial_command', {
            port_name: selectedPort,
            command: `FLASH_SIZE ${profile.flash_size_mb}`,
          });
          onLog?.(
            'info',
            'HOST',
            `Perfil ${profile.model_id} selecionado (${profile.flash_size_mb}MB | EEMUL ${profile.nvram_base_hex}): ${resp}`
          );
        } catch (err) {
          logger.warn('PicoDevice', 'FLASH_SIZE command failed', err);
        }
      }
    },
    [modelCatalog, selectedPort, onLog]
  );

  const refreshPorts = useCallback(async () => {
    setIsLoading(true);
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
    } finally {
      setIsLoading(false);
    }
  }, [selectedPort]);

  const pollDeviceStatus = useCallback(async () => {
    setIsLoading(true);
    try {
      const status = await invokeCommand<PicoDeviceStatus>('pico_get_status', {
        port_name: selectedPort,
      });
      setDeviceStatus(status);
    } catch (err) {
      logger.warn('PicoDevice', 'Failed querying Pico device telemetry', err);
    } finally {
      setIsLoading(false);
    }
  }, [selectedPort]);

  const changeBreakinMode = useCallback(
    async (mode: ColdBreakinMode) => {
      try {
        const updated = await invokeCommand<PicoDeviceStatus>('pico_set_breakin_mode', {
          port_name: selectedPort,
          mode,
        });
        setDeviceStatus(updated);
        onLog?.('info', 'HOST', `Modo de Cold Break-In alterado para: Modo ${mode}`);
      } catch (err) {
        onLog?.('error', 'HOST', `Erro ao alterar modo: ${err}`);
      }
    },
    [selectedPort, onLog]
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

  // Reactive Push-Based Event Stream Subscription
  useEffect(() => {
    refreshPorts();
    refreshModelCatalog();
    pollDeviceStatus();

    let unlisten: (() => void) | undefined;
    let active = true;

    listenEvent<PicoDeviceStatus>('pico_telemetry', incoming => {
      if (!active) return;
      setDeviceStatus(prev => {
        // Instantaneous Reactive Safety Guard: Voltage boundary verification (3.0V - 3.35V)
        if (incoming.vdd_volts < 3.0 || incoming.vdd_volts > 3.35) {
          onLog?.(
            'warn',
            'PICO',
            `ALERTA REATIVO: Tensão VDD fora da faixa nominal (3.0V-3.35V): ${incoming.vdd_volts.toFixed(2)}V`
          );
        }
        return {
          ...incoming,
          // Preserve client-side state flags if not populated by event
          ram_loader_injected: prev?.ram_loader_injected ?? incoming.ram_loader_injected,
        };
      });
    }).then(cleanup => {
      if (active) {
        unlisten = cleanup;
      } else {
        cleanup();
      }
    });

    return () => {
      active = false;
      unlisten?.();
    };
  }, [refreshPorts, refreshModelCatalog, pollDeviceStatus, onLog]);

  // Derived Instantaneous Safety State
  const isVoltageSafe = deviceStatus
    ? deviceStatus.vdd_volts >= 3.0 && deviceStatus.vdd_volts <= 3.35
    : false;
  const isHardwareReady = Boolean(deviceStatus?.connected && isVoltageSafe);

  return {
    deviceStatus,
    setDeviceStatus,
    isLoading,
    availablePorts,
    selectedPort,
    setSelectedPort,
    modelCatalog,
    selectedModel,
    selectTargetModel,
    refreshPorts,
    pollDeviceStatus,
    changeBreakinMode,
    toggleConnection,
    isVoltageSafe,
    isHardwareReady,
  };
}
