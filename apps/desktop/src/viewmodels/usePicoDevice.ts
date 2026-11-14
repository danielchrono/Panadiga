import { useState, useCallback, useEffect, useRef } from 'react';
import {
  ColdBreakinMode,
  DigaModelProfile,
  FlashReport,
  OcdCoreState,
  PicoDeviceStatus,
  PicoDiagnosticResult,
  PicoTransportMode,
  PicoUf2Variant,
  SerialLogEntry,
} from '@/models';
import {
  PICO_MAX_VDD_CEILING_VOLTS,
  PICO_MIN_VDD_THRESHOLD_VOLTS,
  PICO_SAFE_OPERATING_MIN_VOLTS,
} from '@/constants';
import {
  isTargetPowered,
  isTargetVoltageNominal,
} from '@/utils/formatters';
import { emitLocalEvent, invokeCommand, listenEvent } from '@/services/ipc';
import { bleService } from '@/services/bleService';
import {
  connectAndroidUsbOtg,
  disconnectAndroidUsbOtg,
  isAndroidNativeBridgeAvailable,
  listAndroidUsbOtgDevices,
  sendAndroidUsbOtgCommand,
} from '@/services/androidBridge';
import { parseDiagnosticOutput } from '@/services/diagnosticParser';
import { useToast } from '@/context/ToastContext';
import { logger } from '@/utils/logger';

export interface UsePicoDeviceOptions {
  onLog?: (
    level: SerialLogEntry['level'],
    source: SerialLogEntry['source'],
    message: string
  ) => void;
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
  firmware_version: 'P7501 UART / Panadiga Master',
  chip_id: '',
  total_probes: 0,
  hits_detected: 0,
  open_bus_high: 0,
  open_bus_low: 0,
  ram_loader_injected: false,
  last_keepalive: '',
};

const PICO_PORT_PATTERN = /VID:2E8A|RP2040|RP2350|Pico|PANADIGA|P7501/i;

function parseVddFromStatusRaw(rawStatus: string, fallback = 0): number {
  const parsed = parseDiagnosticOutput('STATUS', 'STATUS', rawStatus, 25);
  return parsed.vdd_volts ?? fallback;
}

function deriveCoreStateFromVoltage(
  prevState: OcdCoreState,
  vddVolts: number,
  commandId?: string,
  commandSuccess?: boolean
): OcdCoreState {
  if (commandId === 'HANDSHAKE' && commandSuccess) {
    return 'HALTED';
  }
  if (isTargetPowered(vddVolts, PICO_MIN_VDD_THRESHOLD_VOLTS)) {
    return prevState === 'DISCONNECTED' ? 'RUNNING' : prevState;
  }
  return 'RESET';
}

function buildMobileDeviceStatus(
  prev: PicoDeviceStatus | null,
  portName: string,
  vdd: number,
  chipId?: string
): PicoDeviceStatus {
  const powered = isTargetPowered(vdd, PICO_MIN_VDD_THRESHOLD_VOLTS);
  return {
    ...(prev || DEFAULT_DISCONNECTED_STATUS),
    connected: true,
    port: portName,
    vdd_volts: vdd,
    target_powered: powered,
    core_state: powered ? 'HALTED' : 'RESET',
    ...(chipId ? { chip_id: chipId } : {}),
    last_keepalive: 'OK',
  };
}

async function isGdbBridgeRunning(): Promise<boolean> {
  try {
    const status = await invokeCommand<{ running?: boolean }>('get_gdb_bridge_status_command');
    return Boolean(status?.running);
  } catch {
    return false;
  }
}

export function resolveTransportMode(portName: string): PicoTransportMode {
  if (portName.startsWith('ANDROID-USB-OTG')) return 'usb-otg';
  if (portName.startsWith('BLE_')) return 'ble';
  return 'usb';
}

function createPicoSessionStore() {
  let status: PicoDeviceStatus | null = DEFAULT_DISCONNECTED_STATUS;
  let port = '';
  let model = 'AUTO';

  return Object.freeze({
    getSnapshot: () => ({ status, port, model }),
    setStatus: (nextStatus: PicoDeviceStatus | null) => {
      status = nextStatus;
    },
    setPort: (nextPort: string) => {
      port = nextPort;
    },
    setModel: (nextModel: string) => {
      model = nextModel;
    },
  });
}

const picoSessionStore = createPicoSessionStore();

/**
 * Single Responsibility: Manages Pico hardware link (Desktop USB CDC, Android USB-C OTG,
 * Bluetooth LE NUS), BOOTSEL flashing, SSOT voltage safety checks, and reactive telemetry streams.
 */
export function usePicoDevice({ onLog }: UsePicoDeviceOptions = {}) {
  const { showToast } = useToast();
  const initialSession = picoSessionStore.getSnapshot();
  const [deviceStatus, setDeviceStatus] = useState<PicoDeviceStatus | null>(initialSession.status);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [isFlashing, setIsFlashing] = useState<boolean>(false);
  const [availablePorts, setAvailablePorts] = useState<string[]>([]);
  const [selectedPort, setSelectedPort] = useState<string>(initialSession.port);
  const [modelCatalog, setModelCatalog] = useState<DigaModelProfile[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>(initialSession.model);

  const isConnectingRef = useRef<boolean>(false);
  const lastAlertRef = useRef<'ok' | 'low' | 'over'>('ok');

  const applyMeasuredVoltage = useCallback(
    (vddVolts: number, commandId?: string, commandSuccess?: boolean) => {
      if (typeof vddVolts !== 'number' || isNaN(vddVolts)) return;
      const powered = isTargetPowered(vddVolts, PICO_MIN_VDD_THRESHOLD_VOLTS);
      setDeviceStatus(prev => {
        if (!prev) return prev;
        const nextVdd =
          vddVolts > 0
            ? vddVolts
            : commandId === 'STATUS' || !prev.connected
              ? vddVolts
              : prev.vdd_volts;
        const nextPowered = isTargetPowered(nextVdd, PICO_MIN_VDD_THRESHOLD_VOLTS);
        const updated: PicoDeviceStatus = {
          ...prev,
          vdd_volts: nextVdd,
          target_powered: nextPowered,
          core_state: deriveCoreStateFromVoltage(
            prev.core_state,
            nextVdd,
            commandId,
            commandSuccess
          ),
        };
        if (nextPowered || !prev.connected) {
          picoSessionStore.setStatus(updated);
        } else {
          picoSessionStore.setStatus({ ...updated, vdd_volts: vddVolts, target_powered: powered });
        }
        return updated;
      });
    },
    []
  );

  const sendDirectRawCommand = useCallback(
    async (command: string): Promise<string> => {
      if (selectedPort.startsWith('ANDROID-USB-OTG')) {
        const res = sendAndroidUsbOtgCommand(command, 3000);
        if (!res.ok) throw new Error(res.output);
        return res.output;
      }
      if (selectedPort.startsWith('BLE_') && bleService.isConnected()) {
        return await bleService.sendAndAwaitReply(command, 3500);
      }
      return await invokeCommand<string>('pico_send_serial_command', {
        portName: selectedPort,
        port_name: selectedPort,
        command,
      });
    },
    [selectedPort]
  );

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
          const resp = await sendDirectRawCommand('AUTODETECT');
          onLog?.('info', 'PICO', `Modo AUTO ativado (Address Aliasing Probe): ${resp}`);
        } catch (err) {
          logger.warn('PicoDevice', 'AUTODETECT command failed', err);
        }
      } else if (profile) {
        try {
          const resp = await sendDirectRawCommand(`FLASH_SIZE ${profile.flash_size_mb}`);
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
    [modelCatalog, sendDirectRawCommand, onLog]
  );

  const refreshPorts = useCallback(async () => {
    setIsLoading(true);
    try {
      const ports = await invokeCommand<string[]>('list_pico_serial_ports');
      const allPorts = [...(ports || [])];

      if (isAndroidNativeBridgeAvailable()) {
        const otgDevices = listAndroidUsbOtgDevices();
        otgDevices.forEach(dev => {
          const label = `ANDROID-USB-OTG [${dev.deviceName} - ${dev.productName}${dev.isBootsel ? ' BOOTSEL' : ''}]`;
          if (!allPorts.includes(label)) {
            allPorts.unshift(label);
          }
        });
      }

      if (bleService.isSupported() && !allPorts.some(p => p.includes('BLE_PICO_W'))) {
        allPorts.push('BLE_PICO_W [Bluetooth LE - Nordic UART Service (NUS)]');
      }
      setAvailablePorts(allPorts);

      const realPorts = allPorts.filter(
        p =>
          !p.startsWith('BLE') &&
          !p.includes('BOOTSEL Mode') &&
          !p.includes('BOOTSEL') &&
          !p.includes('PID:0003')
      );

      const preferredPicoPort = realPorts.find(
        p =>
          p.includes('VID:2E8A') ||
          p.startsWith('ANDROID-USB-OTG') ||
          p.includes('RP2040') ||
          p.includes('RP2350') ||
          p.includes('Pico') ||
          p.includes('PANADIGA') ||
          p.includes('P7501')
      );
      const targetDefault = preferredPicoPort || realPorts[0];

      if (targetDefault) {
        if (!selectedPort || !allPorts.includes(selectedPort)) {
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
    if (!selectedPort || isConnectingRef.current) return;
    if (await isGdbBridgeRunning()) return;
    if (selectedPort.startsWith('ANDROID-USB-OTG') || selectedPort.startsWith('BLE_')) {
      try {
        const raw = await sendDirectRawCommand('STATUS');
        const vdd = parseVddFromStatusRaw(raw);
        setDeviceStatus(prev => buildMobileDeviceStatus(prev, selectedPort, vdd));
      } catch (err) {
        logger.warn('PicoDevice', 'Failed mobile STATUS telemetry poll', err);
      }
      return;
    }

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
  }, [selectedPort, sendDirectRawCommand]);

  const changeBreakinMode = useCallback(
    async (mode: ColdBreakinMode) => {
      try {
        if (selectedPort.startsWith('ANDROID-USB-OTG') || selectedPort.startsWith('BLE_')) {
          await sendDirectRawCommand(`BREAK_IN ${mode}`);
          setDeviceStatus(prev => (prev ? { ...prev, breakin_mode: mode } : prev));
          onLog?.('info', 'HOST', `Modo de Cold Break-In alterado para: Modo ${mode}`);
          return;
        }
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
    [selectedPort, sendDirectRawCommand, onLog]
  );

  const toggleConnection = useCallback(async (overridePort?: unknown) => {
    const targetPort =
      typeof overridePort === 'string' && overridePort.trim() ? overridePort : selectedPort;
    if (!targetPort) {
      onLog?.('warn', 'HOST', 'Nenhuma porta selecionada para conexão.');
      return;
    }

    if (isConnectingRef.current) {
      onLog?.('warn', 'HOST', 'Conexão já em andamento — aguarde antes de tentar novamente.');
      return;
    }

    if (deviceStatus?.connected) {
      try {
        await invokeCommand('set_pico_telemetry_stream', { active: false }).catch(() => {});
        if (targetPort.startsWith('ANDROID-USB-OTG')) {
          await sendDirectRawCommand('ISOLATE').catch(() => {});
          disconnectAndroidUsbOtg();
        } else if (targetPort.startsWith('BLE_') && bleService.isConnected()) {
          bleService.disconnect();
        } else {
          await invokeCommand<string>('pico_send_serial_command', {
            portName: targetPort,
            port_name: targetPort,
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
          port: targetPort,
        }));
        onLog?.(
          'warn',
          'HOST',
          `Desconectado da interface P7501 UART / OCD (${targetPort}). Pinos colocados em Hi-Z.`
        );
      }
      return;
    }

    if (await isGdbBridgeRunning()) {
      onLog?.(
        'warn',
        'HOST',
        `${targetPort} está em uso pela Ponte GDB RSP (a sessão de depuração já usa o Pico). Pare a ponte para enviar comandos CDC diretos.`
      );
      return;
    }

    setIsLoading(true);
    isConnectingRef.current = true;
    onLog?.('info', 'HOST', `Estabelecendo conexão física com ${targetPort} (115200 bps)...`);

    try {
      if (targetPort.startsWith('ANDROID-USB-OTG')) {
        const devMatch = targetPort.match(/\[([^\s\]]+)/);
        const otgRes = connectAndroidUsbOtg(devMatch?.[1] || '');
        if (!otgRes.connected) {
          throw new Error(otgRes.message || 'Dispositivo USB-OTG não conectado.');
        }
        const vdd = parseVddFromStatusRaw(sendAndroidUsbOtgCommand('STATUS', 2000).output);
        setDeviceStatus(prev => buildMobileDeviceStatus(prev, targetPort, vdd, 'RP2040/RP2350 OTG'));
        onLog?.('success', 'HOST', `${otgRes.message} | VDD: ${vdd.toFixed(2)}V`);
        return;
      }

      if (targetPort.startsWith('BLE_')) {
        const bleDev = await bleService.connect();
        let measuredVdd = 3.3;
        try {
          const statusOut = await bleService.sendAndAwaitReply('STATUS', 2500);
          measuredVdd = parseVddFromStatusRaw(statusOut, 3.3);
        } catch {
          // Keep default BLE handshake if status reply times out on first packet
        }
        onLog?.('success', 'HOST', `Conectado via Bluetooth LE NUS: ${bleDev.name} (VDD: ${measuredVdd.toFixed(2)}V)`);
        setDeviceStatus(prev => buildMobileDeviceStatus(prev, targetPort, measuredVdd, 'Pico W (BLE)'));
        return;
      }

      const result = await invokeCommand<{
        connected: boolean;
        port_name: string;
        baud_rate: number;
        vdd_voltage: number;
        target_powered: boolean;
        firmware_banner: string;
        message: string;
      }>('pico_connect_and_test', {
        portName: targetPort,
        port_name: targetPort,
        baudRate: 115200,
        baud_rate: 115200,
      });

      if (result && result.connected) {
        const isRp2350 = result.port_name.includes('2350') || result.firmware_banner.includes('2350');
        setDeviceStatus(prev => ({
          ...(prev || DEFAULT_DISCONNECTED_STATUS),
          connected: true,
          port: targetPort,
          baudRate: result.baud_rate,
          vdd_volts: result.vdd_voltage,
          target_powered: result.target_powered,
          core_state: result.target_powered ? 'HALTED' : 'RUNNING',
          firmware_version: result.firmware_banner || 'P7501 UART / Panadiga Master',
          chip_id: isRp2350 ? 'RP2350-A2' : 'RP2040-B2',
          last_keepalive: 'OK',
        }));

        onLog?.('success', 'PICO', result.firmware_banner);
        onLog?.(
          'success',
          'HOST',
          `Conexão estabelecida com sucesso em ${targetPort}! VDD: ${result.vdd_voltage.toFixed(2)}V`
        );

        await invokeCommand('set_pico_telemetry_stream', {
          active: true,
          portName: targetPort,
          port_name: targetPort,
        }).catch(() => {});
      } else {
        throw new Error(result?.message || 'Dispositivo não respondeu ao handshake inicial.');
      }
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      onLog?.('error', 'HOST', `Falha ao conectar em ${targetPort}: ${errMsg}`);
      setDeviceStatus(prev => ({
        ...(prev || DEFAULT_DISCONNECTED_STATUS),
        connected: false,
        port: targetPort,
        core_state: 'DISCONNECTED',
        vdd_volts: 0.0,
        target_powered: false,
      }));
    } finally {
      setIsLoading(false);
      isConnectingRef.current = false;
    }
  }, [deviceStatus, selectedPort, sendDirectRawCommand, onLog]);

  const triggerBootselReboot = useCallback(async () => {
    if (!selectedPort) {
      showToast({
        type: 'warning',
        title: 'Porta Não Selecionada',
        message: 'Selecione a porta serial do Pico antes de solicitar o reset para BOOTSEL.',
      });
      return;
    }
    try {
      const msg = await invokeCommand<string>('pico_trigger_bootsel_reboot', {
        portName: selectedPort,
        port_name: selectedPort,
      });
      showToast({
        type: 'success',
        title: 'BOOTSEL Reset Iniciado',
        message: `${msg}. Aguarde 2 segundos para o drive RPI-RP2 / RP2350 montar e clique em "Flash Pico (BOOTSEL)".`,
      });
      onLog?.('info', 'HOST', `BOOTSEL reset iniciado em ${selectedPort}: ${msg}`);
    } catch (err) {
      showToast({
        type: 'error',
        title: 'Falha no Reset BOOTSEL',
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }, [selectedPort, showToast, onLog]);

  const flashPicoFirmware = useCallback(
    async (uf2Variant: PicoUf2Variant) => {
      setIsFlashing(true);
      try {
        let drives = await invokeCommand<string[]>('pico_detect_bootsel_drives').catch(() => []);
        if ((!drives || drives.length === 0) && selectedPort && PICO_PORT_PATTERN.test(selectedPort)) {
          showToast({
            type: 'info',
            title: 'Reiniciando Pico em BOOTSEL',
            message: 'Enviando pulso 1200-baud para reiniciar a placa em modo de gravação...',
          });
          await invokeCommand('pico_trigger_bootsel_reboot', {
            portName: selectedPort,
            port_name: selectedPort,
          }).catch(() => {});
          await new Promise(resolve => setTimeout(resolve, 1600));
          drives = await invokeCommand<string[]>('pico_detect_bootsel_drives').catch(() => []);
        }

        if (!drives || drives.length === 0) {
          showToast({
            type: 'warning',
            title: 'Nenhum Drive BOOTSEL Detectado',
            message:
              'Nenhum drive RPI-RP2 / RP2350 encontrado. Clique em "BOOTSEL Reset" ou segure BOOTSEL ao conectar.',
          });
          return;
        }

        const report = await invokeCommand<FlashReport>('pico_flash_firmware', {
          targetFamily: uf2Variant,
          target_family: uf2Variant,
        });
        const sizeStr = report.uf2_size_bytes ? report.uf2_size_bytes.toLocaleString() : 'UF2';
        showToast({
          type: 'success',
          title: 'Firmware Pico Gravado',
          message: `${report.message} (${report.board_family || uf2Variant}, ${sizeStr} bytes). Reconectando USB...`,
        });

        let detectedPort = '';
        for (let i = 0; i < 12; i++) {
          await new Promise(resolve => setTimeout(resolve, 500));
          const ports = await invokeCommand<string[]>('list_pico_serial_ports').catch(() => []);
          const found = ports?.find(p => PICO_PORT_PATTERN.test(p));
          if (found) {
            detectedPort = found;
            break;
          }
        }

        await refreshPorts();
        if (detectedPort) {
          setSelectedPort(detectedPort);
          showToast({
            type: 'success',
            title: 'Pico Detectado',
            message: `Reconectando automaticamente em ${detectedPort}...`,
          });
          setTimeout(() => {
            void toggleConnection(detectedPort);
          }, 400);
        }
      } catch (err) {
        showToast({
          type: 'error',
          title: 'Falha ao Gravar Firmware',
          message: err instanceof Error ? err.message : String(err),
        });
      } finally {
        setIsFlashing(false);
      }
    },
    [selectedPort, refreshPorts, showToast, toggleConnection]
  );

  useEffect(() => {
    refreshPorts();
    refreshModelCatalog();
  }, [refreshPorts, refreshModelCatalog]);

  // Subscribe to backend reactive telemetry pushes (`pico://telemetry`) and diagnostic voltage updates
  useEffect(() => {
    let unlistenTelemetry: (() => void) | undefined;
    let unlistenDiag: (() => void) | undefined;

    listenEvent<PicoDeviceStatus>('pico://telemetry', incoming => {
      if (incoming && typeof incoming.vdd_volts === 'number') {
        setDeviceStatus(prev => {
          if (!prev?.connected && !incoming.connected) return prev;
          const preservedVdd =
            incoming.vdd_volts > 0
              ? incoming.vdd_volts
              : prev?.connected && incoming.connected && prev.vdd_volts > 0
                ? prev.vdd_volts
                : incoming.vdd_volts;
          const powered = isTargetPowered(preservedVdd, PICO_MIN_VDD_THRESHOLD_VOLTS);
          const updated: PicoDeviceStatus = {
            ...(prev || DEFAULT_DISCONNECTED_STATUS),
            ...incoming,
            vdd_volts: preservedVdd,
            target_powered: powered,
          };
          picoSessionStore.setStatus(updated);
          return updated;
        });
      }
    }).then(fn => {
      unlistenTelemetry = fn;
    });

    listenEvent<PicoDiagnosticResult>('pico://diagnostic-result', diag => {
      if (diag && typeof diag.vdd_volts === 'number' && diag.vdd_volts > 0) {
        applyMeasuredVoltage(diag.vdd_volts, diag.command_id, diag.success);
      }
    }).then(fn => {
      unlistenDiag = fn;
    });

    return () => {
      unlistenTelemetry?.();
      unlistenDiag?.();
    };
  }, [applyMeasuredVoltage]);

  useEffect(() => {
    picoSessionStore.setStatus(deviceStatus);
    if (deviceStatus) {
      if (deviceStatus.vdd_volts > PICO_MAX_VDD_CEILING_VOLTS) {
        if (lastAlertRef.current !== 'over') {
          lastAlertRef.current = 'over';
          onLog?.(
            'error',
            'PICO',
            `PERIGO CRÍTICO: Sobretensão detectada em VDD (${deviceStatus.vdd_volts.toFixed(2)}V > ${PICO_MAX_VDD_CEILING_VOLTS.toFixed(2)}V)! Desconecte imediatamente!`
          );
        }
      } else if (
        deviceStatus.vdd_volts > 0.2 &&
        deviceStatus.vdd_volts < PICO_MIN_VDD_THRESHOLD_VOLTS
      ) {
        if (lastAlertRef.current !== 'low') {
          lastAlertRef.current = 'low';
          onLog?.(
            'warn',
            'PICO',
            `ALERTA REATIVO: Subtensão em VDD (${deviceStatus.vdd_volts.toFixed(2)}V < ${PICO_MIN_VDD_THRESHOLD_VOLTS.toFixed(2)}V). Alvo desligado ou pino VDD desconectado.`
          );
        }
      } else {
        lastAlertRef.current = 'ok';
      }
      emitLocalEvent('pico_telemetry', deviceStatus);
    }
  }, [deviceStatus, onLog]);

  useEffect(() => {
    picoSessionStore.setPort(selectedPort);
  }, [selectedPort]);

  useEffect(() => {
    picoSessionStore.setModel(selectedModel);
  }, [selectedModel]);

  useEffect(() => {
    const snap = picoSessionStore.getSnapshot();
    if (snap.status?.connected && snap.port) {
      void pollDeviceStatus();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const isVoltageSafe = deviceStatus
    ? isTargetVoltageNominal(
        deviceStatus.vdd_volts,
        PICO_SAFE_OPERATING_MIN_VOLTS,
        PICO_MAX_VDD_CEILING_VOLTS
      )
    : false;
  const isHardwareReady = Boolean(
    deviceStatus?.connected &&
      isTargetPowered(deviceStatus.vdd_volts, PICO_MIN_VDD_THRESHOLD_VOLTS) &&
      deviceStatus.vdd_volts <= PICO_MAX_VDD_CEILING_VOLTS
  );

  return {
    deviceStatus,
    setDeviceStatus,
    applyMeasuredVoltage,
    isLoading,
    isFlashing,
    availablePorts,
    selectedPort,
    setSelectedPort,
    transportMode: resolveTransportMode(selectedPort),
    modelCatalog,
    selectedModel,
    selectTargetModel,
    refreshPorts,
    pollDeviceStatus,
    changeBreakinMode,
    toggleConnection,
    triggerBootselReboot,
    flashPicoFirmware,
    isVoltageSafe,
    isHardwareReady,
  };
}
