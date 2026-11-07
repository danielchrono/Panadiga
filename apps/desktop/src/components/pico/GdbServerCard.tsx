import React, { useState } from 'react';
import { GdbBridgeStatus, Am34RegisterState } from '@/models';
import { StatusBadge } from '@/components/StatusBadge';
import { Am34RegisterGrid } from './Am34RegisterGrid';
import { Server, Play, RefreshCw, Copy, Check } from 'lucide-react';

export interface GdbServerCardProps {
  gdbStatus: GdbBridgeStatus | null;
  registers: Am34RegisterState | null;
  isStarting: boolean;
  onStart: (listenAddr?: string) => Promise<void>;
  onStop: () => Promise<void>;
}

/**
 * Presentation Component: Manages GDB Remote Serial Protocol (RSP) server controls and AM34 live registers.
 */
export const GdbServerCard: React.FC<GdbServerCardProps> = ({
  gdbStatus,
  registers,
  isStarting,
  onStart,
  onStop,
}) => {
  const [copied, setCopied] = useState<boolean>(false);
  const connectionCommand = `target remote ${gdbStatus?.listen_addr || '127.0.0.1:3333'}`;

  const handleCopy = () => {
    navigator.clipboard.writeText(connectionCommand);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
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

        <div className="flex items-center gap-2.5 shrink-0">
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
            className={`px-4 py-2 rounded-xl font-bold text-xs flex items-center gap-2 transition-all ${
              gdbStatus?.running
                ? 'bg-rose-500/10 hover:bg-rose-500/20 text-rose-300 border border-rose-500/30'
                : 'bg-purple-600 hover:bg-purple-500 text-white shadow-lg shadow-purple-950/50'
            }`}
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

      {/* AM34 Register Live Grid */}
      <Am34RegisterGrid registers={registers} />
    </div>
  );
};
