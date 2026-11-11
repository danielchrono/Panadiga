import { useState, useCallback, useEffect } from 'react';
import { PicoDeviceStatus, ColdBreakinMode, DigaModelProfile } from '@/models';
import { invokeCommand, listenEvent } from '@/services/ipc';
import { bleService } from '@/services/bleService';
import { logger } from '@/utils/logger';

export interface UsePicoDeviceOptions {
  onLog?: (level: 'info' | 'warn' | 'error' | 'success', source: 'HOST' | 'PICO', message: string) => void;
}

export const DEFAULT_DISCONNECTED_STATUS: PicoDeviceStatus = {
  connected: false,
  port: '',
  baudRate: 115200,
  vdd_volts: 0.0,
  target_powered: false,
  breakin_mode: 2,
  core_state: 'DISCONNECTED',
  bus_freq_hz: 0,
  firmware_version: 'DWire32A Bus Master',
  chip_id: '',
  total_probes: 0,
  hits_detected: 0,
  open_bus_high: 0,
  open_bus_low: 0,
  ram_loader_injected: false,
  last_keepalive: '',
};

/**
 * Single Responsibility: Manages Pico USB hardware connection, port enumeration,
 * reactive telemetry streams, and break-in modes.
 */
export function usePicoDevice({ onLog }: UsePicoDeviceOptions = {}) {
  const [deviceStatus, setDeviceStatus] = useState<PicoDeviceStatus | null>(DEFAULT_DISCONNECTED_STATUS);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [availablePorts, setAvailablePorts] = useState<string[]>([]);
  const [selectedPort, setSelectedPort] = useState<string>('');
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
      const isBleAvailable =
        typeof navigator !== 'undefined' && 'bluetooth' in navigator;
      const allPorts = [...(ports || [])];
      if (isBleAvailable && !allPorts.some(p => p.includes('BLE_PICO_W'))) {
        allPorts.push('BLE_PICO_W [Bluetooth LE - Nordic UART Service (NUS)]');
      }
      setAvailablePorts(allPorts);

      const realPorts = allPorts.filter(p => !p.startsWith('MOCK') && !p.startsWith('BLE'));
      if (realPorts.length > 0) {
        if (!selectedPort || selectedPort.startsWith('MOCK') || !allPorts.includes(selectedPort)) {
          setSelectedPort(realPorts[0]);
        }
      } else if (allPorts.length > 0 && !selectedPort) {
        setSelectedPort(allPorts[0]);
      } else if (allPorts.length === 0) {
        setSelectedPort('');
      }
    } catch (err) {
      logger.warn('PicoDevice', 'Failed loading system serial ports', err);
    } finally {
      setIsLoading(false);
    }
  }, [selectedPort]);

  const pollDeviceStatus = useCallback(async () => {
    if (!selectedPort) return;
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
    if (!selectedPort) {
      onLog?.('warn', 'HOST', 'Nenhuma porta selecionada para conexão.');
      return;
    }

    if (deviceStatus?.connected) {
      // Disconnect
      try {
        if (selectedPort.startsWith('BLE_') && bleService.isConnected()) {
          bleService.disconnect();
        } else {
          await invokeCommand<string>('pico_send_serial_command', {
            port_name: selectedPort,
            command: 'ISOLATE',
          }).catch(() => {});
        }
      } finally {
        setDeviceStatus(prev => ({
          ...(prev || DEFAULT_DISCONNECTED_STATUS),
          connected: false,
          core_state: 'DISCONNECTED',
          vdd_volts: 0.0,
          target_powered: false,
          port: selectedPort,
        }));
        onLog?.('warn', 'HOST', `Desconectado da interface DWire32A (${selectedPort}). Pinos colocados em Hi-Z.`);
      }
      return;
    }

    // Connect
    setIsLoading(true);
    onLog?.('info', 'HOST', `Estabelecendo conexão física com ${selectedPort} (115200 bps)...`);

    try {
      if (selectedPort.startsWith('BLE_')) {
        const bleDev = await bleService.connect();
        onLog?.('success', 'HOST', `Conectado via Bluetooth LE NUS: ${bleDev.name}`);
        setDeviceStatus(prev => ({
          ...(prev || DEFAULT_DISCONNECTED_STATUS),
          connected: true,
          port: selectedPort,
          core_state: 'HALTED',
          vdd_volts: 3.3,
          target_powered: true,
        }));
        return;
      }

      // Real physical connection test via Tauri IPC
      const result = await invokeCommand<{
        connected: boolean;
        port_name: string;
        baud_rate: number;
        vdd_voltage: number;
        target_powered: boolean;
        firmware_banner: string;
        is_mock: boolean;
        message: string;
      }>('pico_connect_and_test', {
        port_name: selectedPort,
        baud_rate: 115200,
      });

      if (result && result.connected) {
        const isRp2350 = result.port_name.includes('2350') || result.firmware_banner.includes('2350');
        setDeviceStatus(prev => ({
          ...(prev || DEFAULT_DISCONNECTED_STATUS),
          connected: true,
          port: selectedPort,
          baudRate: result.baud_rate,
          vdd_volts: result.vdd_voltage,
          target_powered: result.target_powered,
          core_state: result.target_powered ? 'HALTED' : 'RUNNING',
          firmware_version: result.firmware_banner || 'v2.0-PIO-DWire32A',
          chip_id: isRp2350 ? 'RP2350-A2' : 'RP2040-B2',
          last_keepalive: 'OK',
        }));

        onLog?.('success', 'PICO', result.firmware_banner);
        onLog?.(
          'success',
          'HOST',
          `Conexão estabelecida com sucesso em ${selectedPort}! VDD: ${result.vdd_voltage.toFixed(2)}V`
        );
      } else {
        throw new Error(result?.message || 'Dispositivo não respondeu ao handshake inicial.');
      }
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      onLog?.('error', 'HOST', `Falha ao conectar em ${selectedPort}: ${errMsg}`);
      setDeviceStatus(prev => ({
        ...(prev || DEFAULT_DISCONNECTED_STATUS),
        connected: false,
        port: selectedPort,
        core_state: 'DISCONNECTED',
        vdd_volts: 0.0,
        target_powered: false,
      }));
    } finally {
      setIsLoading(false);
    }
  }, [deviceStatus, selectedPort, onLog]);

  // Reactive Push-Based Event Stream Subscription
  useEffect(() => {
    refreshPorts();
    refreshModelCatalog();

    let unlisten: (() => void) | undefined;
    let active = true;

    listenEvent<PicoDeviceStatus>('pico_telemetry', incoming => {
      if (!active) return;
      setDeviceStatus(prev => {
        // Instantaneous Reactive Safety Guard: Voltage boundary verification (2.8V - 3.55V)
        if (incoming.vdd_volts < 2.8 || incoming.vdd_volts > 3.55) {
          onLog?.(
            'warn',
            'PICO',
            `ALERTA REATIVO: Tensão VDD fora da faixa nominal (2.8V-3.55V): ${incoming.vdd_volts.toFixed(2)}V`
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
  }, [refreshPorts, refreshModelCatalog, onLog]);

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
