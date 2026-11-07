import React, { useState } from 'react';
import { GdbBridgeStatus, Am34RegisterState } from '@/models';
import { StatusBadge } from '@/components/StatusBadge';
import { Am34RegisterGrid } from './Am34RegisterGrid';
import {
  Server,
  Play,
  RefreshCw,
  Copy,
  Check,
  Bug,
  Zap,
  FileCode,
  Binary,
  Terminal,
} from 'lucide-react';
import { cn } from '@/lib/utils';

export interface GdbServerCardProps {
  gdbStatus: GdbBridgeStatus | null;
  registers: Am34RegisterState | null;
  isStarting: boolean;
  onStart: (listenAddr?: string) => Promise<void>;
  onStop: () => Promise<void>;
  onLaunchTool?: (tool: 'gdb' | 'openocd' | 'ghidra' | 'hxd' | 'terminal') => Promise<void>;
}

/**
 * Presentation Component: Manages GDB Remote Serial Protocol (RSP) server controls,
 * 1-click external tool launchers (GDB, OpenOCD, Ghidra, HxD), and AM34 live registers.
 */
export const GdbServerCard: React.FC<GdbServerCardProps> = ({
  gdbStatus,
  registers,
  isStarting,
  onStart,
  onStop,
  onLaunchTool,
}) => {
  const [copied, setCopied] = useState<boolean>(false);
  const [activeTool, setActiveTool] = useState<string | null>(null);
  const connectionCommand = `target remote ${gdbStatus?.listen_addr || '127.0.0.1:3333'}`;

  const handleCopy = () => {
    navigator.clipboard.writeText(connectionCommand);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleLaunch = async (tool: 'gdb' | 'openocd' | 'ghidra' | 'hxd' | 'terminal') => {
    if (!onLaunchTool) return;
    setActiveTool(tool);
    try {
      await onLaunchTool(tool);
    } finally {
      setTimeout(() => setActiveTool(null), 1200);
    }
  };

  return (
    <div className="rounded-2xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
      {/* Header & Controls */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 border-b border-slate-800 pb-4">
        <div className="flex items-center gap-3">
          <div className="p-2.5 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/30">
            <Server className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="text-sm font-bold text-slate-100">
                Servidor GDB Remote RSP (Porta TCP 3333)
              </h3>
              {gdbStatus?.running ? (
                (gdbStatus.connected_clients ?? 0) > 0 ? (
                  <StatusBadge
                    variant="success"
                    label={`CLIENTE ATIVO (${gdbStatus.connected_clients})`}
                  />
                ) : (
                  <StatusBadge variant="warning" label="OUVINDO (3333)" />
                )
              ) : (
                <StatusBadge variant="neutral" label="OFFLINE" />
              )}
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Conecte ferramentas de engenharia reversa (Ghidra Debugger, IDA Pro, gdb-multiarch) ao vivo no AM34.
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2.5 shrink-0 flex-wrap">
          <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-[11px] text-slate-300">
            <code>(gdb) {connectionCommand}</code>
            <button
              onClick={handleCopy}
              className="text-slate-400 hover:text-cyan-400 p-1 transition-colors"
              title="Copiar comando GDB"
            >
              {copied ? (
                <Check className="w-3.5 h-3.5 text-emerald-400" />
              ) : (
                <Copy className="w-3.5 h-3.5" />
              )}
            </button>
          </div>

          <button
            onClick={() => (gdbStatus?.running ? onStop() : onStart())}
            disabled={isStarting}
            className={cn(
              'px-4 py-2 rounded-xl font-bold text-xs flex items-center gap-2 transition-all',
              gdbStatus?.running
                ? 'bg-rose-500/10 hover:bg-rose-500/20 text-rose-300 border border-rose-500/30'
                : 'bg-purple-600 hover:bg-purple-500 text-white shadow-lg shadow-purple-950/50'
            )}
          >
            {isStarting ? (
              <>
                <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                <span>Iniciando...</span>
              </>
            ) : gdbStatus?.running ? (
              <span>Parar Servidor GDB</span>
            ) : (
              <>
                <Play className="w-3.5 h-3.5" />
                <span>Iniciar Servidor GDB</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* 1-Click Ecosystem External Tools Launcher */}
      <div className="p-3 rounded-xl bg-slate-950/70 border border-slate-800/80 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <span className="text-[11px] font-mono font-medium text-slate-400 uppercase tracking-wider">
            Ecossistema & Ferramentas Externas:
          </span>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <button
            onClick={() => handleLaunch('gdb')}
            disabled={activeTool === 'gdb'}
            className={cn(
              'px-3 py-1.5 rounded-lg text-xs font-mono font-semibold flex items-center gap-1.5 border transition-all',
              'bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-300 border-cyan-500/30 shadow-sm'
            )}
            title="Abre terminal interativo com gdb-multiarch conectado na porta 3333"
          >
            <Bug className="w-3.5 h-3.5" />
            <span>GDB Multiarch</span>
          </button>

          <button
            onClick={() => handleLaunch('openocd')}
            disabled={activeTool === 'openocd'}
            className={cn(
              'px-3 py-1.5 rounded-lg text-xs font-mono font-semibold flex items-center gap-1.5 border transition-all',
              'bg-amber-500/10 hover:bg-amber-500/20 text-amber-300 border-amber-500/30 shadow-sm'
            )}
            title="Inicia OpenOCD carregando a configuração da placa Panasonic AM34"
          >
            <Zap className="w-3.5 h-3.5" />
            <span>OpenOCD AM34</span>
          </button>

          <button
            onClick={() => handleLaunch('ghidra')}
            disabled={activeTool === 'ghidra'}
            className={cn(
              'px-3 py-1.5 rounded-lg text-xs font-mono font-semibold flex items-center gap-1.5 border transition-all',
              'bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-300 border-emerald-500/30 shadow-sm'
            )}
            title="Abre a pasta contendo os scripts Ghidra: mapa de memória, símbolos e debugger"
          >
            <FileCode className="w-3.5 h-3.5" />
            <span>Scripts Ghidra</span>
          </button>

          <button
            onClick={() => handleLaunch('hxd')}
            disabled={activeTool === 'hxd'}
            className={cn(
              'px-3 py-1.5 rounded-lg text-xs font-mono font-semibold flex items-center gap-1.5 border transition-all',
              'bg-slate-800 hover:bg-slate-700 text-slate-200 border-slate-700 shadow-sm'
            )}
            title="Abre o editor hexadecimal HxD com o dump da memória"
          >
            <Binary className="w-3.5 h-3.5" />
            <span>HxD Editor</span>
          </button>

          <button
            onClick={() => handleLaunch('terminal')}
            disabled={activeTool === 'terminal'}
            className={cn(
              'px-3 py-1.5 rounded-lg text-xs font-mono font-semibold flex items-center gap-1.5 border transition-all',
              'bg-slate-800 hover:bg-slate-700 text-slate-300 border-slate-700 shadow-sm'
            )}
            title="Abre o terminal PowerShell no diretório do projeto"
          >
            <Terminal className="w-3.5 h-3.5" />
            <span>PowerShell</span>
          </button>
        </div>
      </div>

      {/* AM34 Register Live Grid */}
      <Am34RegisterGrid registers={registers} />
    </div>
  );
};
