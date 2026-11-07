import { useState, useCallback, useEffect } from 'react';
import { GdbBridgeStatus, Am34RegisterState } from '@/models';
import { invokeCommand } from '@/services/ipc';

export interface UseGdbBridgeOptions {
  selectedPort: string;
  onLog?: (level: 'info' | 'warn' | 'error' | 'success', source: 'HOST' | 'PICO', message: string) => void;
}

/**
 * Single Responsibility: Manages the GDB RSP TCP server bridge lifecycle and AM34 register telemetry.
 */
export function useGdbBridge({ selectedPort, onLog }: UseGdbBridgeOptions) {
  const [gdbStatus, setGdbStatus] = useState<GdbBridgeStatus | null>(null);
  const [am34Registers, setAm34Registers] = useState<Am34RegisterState | null>(null);
  const [isGdbStarting, setIsGdbStarting] = useState<boolean>(false);

  const pollGdbState = useCallback(async () => {
    try {
      const gdb = await invokeCommand<GdbBridgeStatus>('get_gdb_bridge_status_command');
      setGdbStatus(gdb);
      if (gdb?.running) {
        const regs = await invokeCommand<Am34RegisterState>('get_am34_registers_command');
        setAm34Registers(regs);
      }
    } catch (err) {
      console.warn('Failed querying GDB status:', err);
    }
  }, []);

  const startGdbServer = useCallback(async (listenAddr = '127.0.0.1:3333') => {
    setIsGdbStarting(true);
    onLog?.('info', 'HOST', `Iniciando Servidor Ponte GDB RSP em ${listenAddr} (Porta: ${selectedPort})...`);
    try {
      const status = await invokeCommand<GdbBridgeStatus>('start_gdb_bridge_command', {
        port_name: selectedPort,
        listen_addr: listenAddr,
      });
      setGdbStatus(status);
      onLog?.('success', 'HOST', `Ponte GDB RSP ativa! Conecte seu depurador com: (gdb) target remote ${listenAddr}`);
    } catch (err) {
      onLog?.('error', 'HOST', `Falha ao iniciar ponte GDB: ${err}`);
    } finally {
      setIsGdbStarting(false);
    }
  }, [selectedPort, onLog]);

  const stopGdbServer = useCallback(async () => {
    onLog?.('warn', 'HOST', 'Finalizando Servidor Ponte GDB RSP...');
    try {
      const status = await invokeCommand<GdbBridgeStatus>('stop_gdb_bridge_command');
      setGdbStatus(status);
      onLog?.('info', 'HOST', 'Servidor Ponte GDB desativado.');
    } catch (err) {
      onLog?.('error', 'HOST', `Falha ao parar ponte GDB: ${err}`);
    }
  }, [onLog]);

  const launchTool = useCallback(
    async (tool: 'gdb' | 'openocd' | 'ghidra' | 'hxd' | 'terminal') => {
      onLog?.('info', 'HOST', `Disparando ferramenta externa: ${tool.toUpperCase()}...`);
      try {
        const msg = await invokeCommand<string>('launch_external_tool_command', { tool });
        onLog?.('success', 'HOST', msg);
      } catch (err) {
        onLog?.('error', 'HOST', `Erro ao abrir ${tool}: ${err}`);
      }
    },
    [onLog]
  );

  useEffect(() => {
    pollGdbState();
    const interval = setInterval(pollGdbState, 2000);
    return () => clearInterval(interval);
  }, [pollGdbState]);

  return {
    gdbStatus,
    am34Registers,
    isGdbStarting,
    startGdbServer,
    stopGdbServer,
    pollGdbState,
    launchTool,
  };
}
