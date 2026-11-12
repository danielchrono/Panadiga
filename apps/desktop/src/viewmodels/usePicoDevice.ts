import { useState, useCallback, useEffect, useRef } from 'react';
import { PicoDeviceStatus, ColdBreakinMode, DigaModelProfile } from '@/models';
import { invokeCommand, listenEvent, emitLocalEvent } from '@/services/ipc';
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
 * Returns true while the GDB RSP bridge owns the CDC serial port (exclusive access).
 * Query failures are treated as "not running" so normal connection flow is never blocked.
 */
async function isGdbBridgeRunning(): Promise<boolean> {
  try {
    const status = await invokeCommand<{ running?: boolean }>('get_gdb_bridge_status_command');
    return Boolean(status?.running);
  } catch {
    return false;
  }
}

/**
 * Session-scoped snapshot that survives view unmount/remount. The hook lives inside the lazily
 * loaded Pico Bench view; without this, leaving the view and coming back reset the link to
 * "DISCONNECTED" (and broadcast that to the Header/Sidebar) although the Pico was still attached.
 */
const sessionSnapshot: { status: PicoDeviceStatus | null; port: string; model: string } = {
  status: DEFAULT_DISCONNECTED_STATUS,
  port: '',
  model: 'AUTO',
};

/**
 * Single Responsibility: Manages Pico USB hardware connection, port enumeration,
 * reactive telemetry streams, and break-in modes.
 */
export function usePicoDevice({ onLog }: UsePicoDeviceOptions = {}) {
  const [deviceStatus, setDeviceStatus] = useState<PicoDeviceStatus | null>(sessionSnapshot.status);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [availablePorts, setAvailablePorts] = useState<string[]>([]);
  const [selectedPort, setSelectedPort] = useState<string>(sessionSnapshot.port);
  const [modelCatalog, setModelCatalog] = useState<DigaModelProfile[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>(sessionSnapshot.model);

  /**
   * Mutex ref: prevents concurrent pico_connect_and_test calls on different ports.
   * Serial port ownership is exclusive — two simultaneous open() calls → "Acesso Negado".
   */
  const isConnectingRef = useRef<boolean>(false);

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
            portName: selectedPort,
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
            portName: selectedPort,
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

      // Exclude MOCK, BLE, and BOOTSEL-mode drives (Pico in BOOTSEL shows as USB mass storage,
      // NOT as a COM port — it has VID:2E8A PID:0003 and label contains "BOOTSEL Mode").
      // Attempting to open a BOOTSEL-mode port with serialport always fails.
      const realPorts = allPorts.filter(
        p =>
          !p.startsWith('MOCK') &&
          !p.startsWith('BLE') &&
          !p.includes('BOOTSEL Mode') &&
          !p.includes('PID:0003')
      );

      // Prioritize recognized Raspberry Pi Pico / DWire32A controllers (VID:2E8A)
      // over motherboard management ports (e.g. Intel AMT SOL on COM3).
      const preferredPicoPort = realPorts.find(
        p =>
          p.includes('VID:2E8A') ||
          p.includes('RP2040') ||
          p.includes('RP2350') ||
          p.includes('Pico') ||
          p.includes('DWire32A')
      );
      const targetDefault = preferredPicoPort || realPorts[0];

      if (targetDefault) {
        if (!selectedPort || selectedPort.startsWith('MOCK') || !allPorts.includes(selectedPort)) {
          setSelectedPort(targetDefault);
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
    // Don't poll if a connection is already in progress (prevents "Acesso Negado" on concurrent open())
    if (!selectedPort || isConnectingRef.current) return;
    // The GDB RSP bridge owns the CDC port exclusively; polling would only produce "porta ocupada" noise.
    if (await isGdbBridgeRunning()) return;
    setIsLoading(true);
    try {
      const status = await invokeCommand<PicoDeviceStatus>('pico_get_status', {
        portName: selectedPort,
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
          portName: selectedPort,
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

    // Prevent double-connect race: serial port is exclusive, two concurrent open() → "Acesso Negado"
    if (isConnectingRef.current) {
      onLog?.('warn', 'HOST', 'Conexão já em andamento — aguarde antes de tentar novamente.');
      return;
    }

    if (deviceStatus?.connected) {
      // Disconnect
      try {
        if (selectedPort.startsWith('BLE_') && bleService.isConnected()) {
          bleService.disconnect();
        } else {
          await invokeCommand<string>('pico_send_serial_command', {
            portName: selectedPort,
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

    // The GDB RSP bridge holds the serial port exclusively: keep the existing link instead of failing.
    if (await isGdbBridgeRunning()) {
      onLog?.(
        'warn',
        'HOST',
        `${selectedPort} está em uso pela Ponte GDB RSP (a sessão de depuração já usa o Pico). Pare a ponte para enviar comandos CDC diretos.`
      );
      return;
    }

    // Connect
    setIsLoading(true);
    isConnectingRef.current = true;
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
        portName: selectedPort,
        port_name: selectedPort,
        baudRate: 115200,
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
      isConnectingRef.current = false;
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
        // Instantaneous Reactive Safety Guard:
        // When target is connected and powered, guard nominal 2.8V - 3.55V.
        // If > 3.55V, ALWAYS trigger a critical overvoltage warning!
        // If <= 0.2V, target is disconnected or powered off (normal quiescent state, no alarm spam).
        if (incoming.vdd_volts > 3.55) {
          onLog?.(
            'error',
            'PICO',
            `PERIGO CRÍTICO: Sobretensão detectada em VDD (${incoming.vdd_volts.toFixed(2)}V > 3.55V)! Desconecte imediatamente!`
          );
        } else if (incoming.vdd_volts > 0.2 && incoming.vdd_volts < 2.8) {
          onLog?.(
            'warn',
            'PICO',
            `ALERTA REATIVO: Subtensão em VDD (${incoming.vdd_volts.toFixed(2)}V < 2.80V). Alvo pode estar instável.`
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

  // Broadcast device status changes to top-level subscribers (Header, Sidebar)
  // and persist the snapshot so remounting the view keeps the real link state.
  useEffect(() => {
    sessionSnapshot.status = deviceStatus;
    if (deviceStatus) {
      emitLocalEvent('pico_telemetry', deviceStatus);
    }
  }, [deviceStatus]);

  useEffect(() => {
    sessionSnapshot.port = selectedPort;
  }, [selectedPort]);

  useEffect(() => {
    sessionSnapshot.model = selectedModel;
  }, [selectedModel]);

  // Revalidate a restored "connected" snapshot once on mount (cable may have been pulled meanwhile).
  useEffect(() => {
    if (sessionSnapshot.status?.connected && sessionSnapshot.port) {
      void pollDeviceStatus();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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
