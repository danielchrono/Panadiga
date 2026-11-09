import { useCallback } from 'react';
import { usePicoDevice } from './usePicoDevice';
import { useGdbBridge } from './useGdbBridge';
import { useInCircuitErase } from './useInCircuitErase';
import { useSerialConsole } from './useSerialConsole';

/**
 * Coordinating ViewModel: Composes specialized domain hooks
 * following MVVM and Single Responsibility principles.
 */
export function usePicoBenchViewModel() {
  const console = useSerialConsole();

  const device = usePicoDevice({
    onLog: console.addLog,
  });

  const gdb = useGdbBridge({
    selectedPort: device.selectedPort,
    onLog: console.addLog,
  });

  const erase = useInCircuitErase({
    onLog: console.addLog,
    onSuccess: useCallback(() => {
      device.setDeviceStatus(prev => (prev ? { ...prev, ram_loader_injected: true } : null));
    }, [device]),
  });

  return {
    // Device & Connection Subsystem
    deviceStatus: device.deviceStatus,
    availablePorts: device.availablePorts,
    selectedPort: device.selectedPort,
    setSelectedPort: device.setSelectedPort,
    refreshPorts: device.refreshPorts,
    changeBreakinMode: device.changeBreakinMode,
    toggleConnection: device.toggleConnection,
    refreshStatus: device.pollDeviceStatus,
    isVoltageSafe: device.isVoltageSafe,
    isHardwareReady: device.isHardwareReady,

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

    // Interactive Serial Console Subsystem
    logs: console.logs,
    logContainerRef: console.logContainerRef,
    commandInput: console.commandInput,
    setCommandInput: console.setCommandInput,
    sendCommand: console.sendCommand,
    clearLogs: console.clearLogs,
  };
}
