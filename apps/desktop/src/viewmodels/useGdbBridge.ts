import { useState, useCallback, useEffect } from 'react';
import { GdbBridgeStatus, Am34RegisterState, SerialLogEntry } from '@/models';
import { GDB_DEFAULT_LISTEN_ADDR } from '@/constants';
import { invokeCommand } from '@/services/ipc';
import { logger } from '@/utils/logger';

export interface UseGdbBridgeOptions {
  selectedPort: string;
  onLog?: (level: SerialLogEntry['level'], source: SerialLogEntry['source'], message: string) => void;
}

/**
 * Closed algebraic state machine for GDB RSP server bridge lifecycle.
 * Eliminates illegal states at compile time (ADTs / Discriminated Union).
 */
export type GdbBridgeState =
  | { status: 'stopped' }
  | { status: 'starting'; listenAddr: string }
  | { status: 'running'; listenAddr: string; clientCount: number }
  | { status: 'error'; message: string };

/**
 * Single Responsibility: Manages the GDB RSP TCP server bridge lifecycle and AM34 register telemetry.
 */
export function useGdbBridge({ selectedPort, onLog }: UseGdbBridgeOptions) {
  const [gdbStatus, setGdbStatus] = useState<GdbBridgeStatus | null>(null);
  const [am34Registers, setAm34Registers] = useState<Am34RegisterState | null>(null);
  const [state, setState] = useState<GdbBridgeState>({ status: 'stopped' });

  const isGdbStarting = state.status === 'starting';

  const pollGdbState = useCallback(async () => {
    try {
      const gdb = await invokeCommand<GdbBridgeStatus>('get_gdb_bridge_status_command');
      setGdbStatus(gdb);
      if (gdb?.running) {
        setState({
          status: 'running',
          listenAddr: gdb.listen_addr,
          clientCount: gdb.connected_clients,
        });
        const regs = await invokeCommand<Am34RegisterState>('get_am34_registers_command');
        setAm34Registers(regs);
      } else {
        setState(prev => (prev.status === 'starting' ? prev : { status: 'stopped' }));
      }
    } catch (err) {
      logger.warn('GdbBridge', 'Failed querying GDB status', err);
    }
  }, []);

  const startGdbServer = useCallback(async (listenAddr = GDB_DEFAULT_LISTEN_ADDR) => {
    setState({ status: 'starting', listenAddr });
    onLog?.('info', 'HOST', `Iniciando Servidor Ponte GDB RSP em ${listenAddr} (Porta: ${selectedPort})...`);
    try {
      const status = await invokeCommand<GdbBridgeStatus>('start_gdb_bridge_command', {
        portName: selectedPort,
        port_name: selectedPort,
        listenAddr,
        listen_addr: listenAddr,
      });
      setGdbStatus(status);
      setState({
        status: 'running',
        listenAddr: status.listen_addr,
        clientCount: status.connected_clients,
      });
      onLog?.('success', 'HOST', `Ponte GDB RSP ativa! Conecte seu depurador com: (gdb) target remote ${listenAddr}`);
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setState({ status: 'error', message: errMsg });
      onLog?.('error', 'HOST', `Falha ao iniciar ponte GDB: ${errMsg}`);
    }
  }, [selectedPort, onLog]);

  const stopGdbServer = useCallback(async () => {
    onLog?.('warn', 'HOST', 'Finalizando Servidor Ponte GDB RSP...');
    try {
      const status = await invokeCommand<GdbBridgeStatus>('stop_gdb_bridge_command');
      setGdbStatus(status);
      setState({ status: 'stopped' });
      onLog?.('info', 'HOST', 'Servidor Ponte GDB desativado.');
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setState({ status: 'error', message: errMsg });
      onLog?.('error', 'HOST', `Falha ao parar ponte GDB: ${errMsg}`);
    }
  }, [onLog]);

  const launchTool = useCallback(
    async (tool: 'gdb' | 'ghidra' | 'hxd' | 'terminal') => {
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
    state,
    gdbStatus,
    am34Registers,
    isGdbStarting,
    startGdbServer,
    stopGdbServer,
    pollGdbState,
    launchTool,
  };
}
