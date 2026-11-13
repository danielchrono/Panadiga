import React, { useState } from 'react';
import type { GdbBridgeStatus } from '@/models';
import { CLIPBOARD_FEEDBACK_MS, GDB_DEFAULT_LISTEN_ADDR } from '@/constants';
import { StatusBadge } from '@/components/StatusBadge';
import { Button } from '@/components/ui';
import { Server, Play, RefreshCw, Copy, Check } from 'lucide-react';

export interface GdbServerControlsProps {
  gdbStatus: GdbBridgeStatus | null;
  isStarting: boolean;
  onStart: (listenAddr?: string) => Promise<void>;
  onStop: () => Promise<void>;
}

export const GdbServerControls: React.FC<GdbServerControlsProps> = ({
  gdbStatus,
  isStarting,
  onStart,
  onStop,
}) => {
  const [copied, setCopied] = useState<boolean>(false);
  const listenAddr = gdbStatus?.listen_addr || GDB_DEFAULT_LISTEN_ADDR;
  const connectionCommand = `target remote ${listenAddr}`;

  const handleCopy = () => {
    navigator.clipboard?.writeText(connectionCommand);
    setCopied(true);
    setTimeout(() => setCopied(false), CLIPBOARD_FEEDBACK_MS);
  };

  return (
    <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 border-b border-slate-800 pb-4">
      <div className="flex items-center gap-3">
        <div className="p-2.5 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/30">
          <Server className="w-5 h-5" />
        </div>
        <div>
          <div className="flex items-center gap-2 flex-wrap">
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
          <p className="text-xs text-slate-400 mt-0.5 font-sans">
            Socket TCP binário GDB RSP (não abre como link HTTP no navegador). Conecte gdb-multiarch ou Ghidra Debugger em{' '}
            <code className="text-purple-300 font-mono">{listenAddr}</code>.
          </p>
        </div>
      </div>

      <div className="flex items-center gap-2.5 shrink-0 flex-wrap">
        <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-[11px] text-slate-300">
          <code>(gdb) {connectionCommand}</code>
          <button
            type="button"
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

        {gdbStatus?.running ? (
          <Button
            type="button"
            variant="destructive"
            size="sm"
            onClick={() => onStop()}
            className="font-bold text-xs"
          >
            Parar GDB
          </Button>
        ) : (
          <Button
            type="button"
            variant="default"
            size="sm"
            onClick={() => onStart(listenAddr)}
            disabled={isStarting}
            className="bg-purple-600 hover:bg-purple-500 text-white font-bold text-xs gap-2 shadow-lg shadow-purple-950/50"
          >
            {isStarting ? (
              <>
                <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                <span>Iniciando...</span>
              </>
            ) : (
              <>
                <Play className="w-3.5 h-3.5 fill-current" />
                <span>Iniciar Servidor GDB</span>
              </>
            )}
          </Button>
        )}
      </div>
    </div>
  );
};
