import { useCallback, useRef } from 'react';
import type { PicoDiagnosticResult } from '@/models';
import { PICO_MIN_VDD_THRESHOLD_VOLTS } from '@/constants';
import { isTargetPowered } from '@/utils/formatters';
import { useHardwareSsot } from './useHardwareSsot';
import { usePicoDevice } from './usePicoDevice';
import { usePicoDiagnostics } from './usePicoDiagnostics';
import { useGdbBridge } from './useGdbBridge';
import { useInCircuitErase } from './useInCircuitErase';
import { useSerialConsole } from './useSerialConsole';

/**
 * Coordinating ViewModel: Composes SSOT spec, Pico connection/transport,
 * Granular Diagnostic Studio, GDB RSP Bridge, In-Circuit Erase, and Reactive Console.
 */
export function usePicoBenchViewModel() {
  const {
    ssot,
    isLoaded: isSsotLoaded,
    isLoading: isSsotLoading,
    error: ssotError,
    reload: reloadSsot,
  } = useHardwareSsot();
  const telemetryRef = useRef<((result: PicoDiagnosticResult) => void) | undefined>(undefined);
  const console = useSerialConsole({
    onCommandTelemetry: useCallback((result: PicoDiagnosticResult) => {
      telemetryRef.current?.(result);
    }, []),
  });

  const device = usePicoDevice({
    onLog: console.addLog,
  });
  const { setDeviceStatus } = device;

  const handleDiagnosticTelemetry = useCallback(
    (result: PicoDiagnosticResult) => {
      if (result.vdd_volts !== null && result.vdd_volts !== undefined) {
        const vdd = result.vdd_volts;
        const powered = isTargetPowered(vdd, PICO_MIN_VDD_THRESHOLD_VOLTS);
        setDeviceStatus(prev =>
          prev
            ? {
                ...prev,
                vdd_volts: vdd,
                target_powered: powered,
                core_state:
                  result.command_id === 'HANDSHAKE' && result.success
                    ? 'HALTED'
                    : powered
                      ? prev.core_state === 'DISCONNECTED'
                        ? 'RUNNING'
                        : prev.core_state
                      : 'RESET',
              }
            : prev
        );
      }
    },
    [setDeviceStatus]
  );
  telemetryRef.current = handleDiagnosticTelemetry;

  const diagnostics = usePicoDiagnostics({
    selectedPort: device.selectedPort,
    ssot,
    onLog: console.addLog,
    onTelemetryUpdate: handleDiagnosticTelemetry,
  });

  const gdb = useGdbBridge({
    selectedPort: device.selectedPort,
    onLog: console.addLog,
  });

  const erase = useInCircuitErase({
    selectedPort: device.selectedPort,
    onLog: console.addLog,
    onSuccess: useCallback(() => {
      setDeviceStatus(prev => (prev ? { ...prev, ram_loader_injected: true } : null));
    }, [setDeviceStatus]),
  });

  const handleSendCommand = useCallback(() => {
    return console.sendCommand(device.selectedPort);
  }, [console, device.selectedPort]);

  return {
    // Hardware/Firmware SSOT Spec
    ssot,
    isSsotLoaded,
    isSsotLoading,
    ssotError,
    reloadSsot,

    // Device & Connection Subsystem
    deviceStatus: device.deviceStatus,
    availablePorts: device.availablePorts,
    selectedPort: device.selectedPort,
    setSelectedPort: device.setSelectedPort,
    transportMode: device.transportMode,
    modelCatalog: device.modelCatalog,
    selectedModel: device.selectedModel,
    selectTargetModel: device.selectTargetModel,
    refreshPorts: device.refreshPorts,
    changeBreakinMode: device.changeBreakinMode,
    toggleConnection: device.toggleConnection,
    refreshStatus: device.pollDeviceStatus,
    isVoltageSafe: device.isVoltageSafe,
    isHardwareReady: device.isHardwareReady,

    // Granular Pico Diagnostic Studio
    diagnosticHistory: diagnostics.history,
    latestDiagnostic: diagnostics.latestResult,
    runningCommandId: diagnostics.runningCommandId,
    isDiagnosticRunning: diagnostics.isExecuting,
    diagnosticError: diagnostics.error,
    diagnosticCategory: diagnostics.selectedCategory,
    setDiagnosticCategory: diagnostics.setSelectedCategory,
    executeDiagnostic: diagnostics.executeCommand,
    clearDiagnosticHistory: diagnostics.clearHistory,

    // GDB Remote RSP Server Subsystem
    gdbStatus: gdb.gdbStatus,
    am34Registers: gdb.am34Registers,
    isGdbStarting: gdb.isGdbStarting,
    startGdbServer: gdb.startGdbServer,
    stopGdbServer: gdb.stopGdbServer,
    launchExternalTool: gdb.launchTool,

    // In-Circuit NVRAM Erase Subsystem
    isErasing: erase.isErasing,
    eraseResult: erase.eraseResult,
    injectRamLoaderAndErase: erase.injectRamLoaderAndErase,
    isBackingUp: erase.isBackingUp,
    backupResult: erase.backupResult,
    executeFirstContactBackup: erase.executeFirstContactBackup,

    // Interactive Serial Console Subsystem
    logs: console.logs,
    logContainerRef: console.logContainerRef,
    commandInput: console.commandInput,
    setCommandInput: console.setCommandInput,
    sendCommand: handleSendCommand,
    clearLogs: console.clearLogs,
  };
}
