import React, { useState } from 'react';
import { Bug, Zap, FileCode, Binary, Terminal } from 'lucide-react';
import { cn } from '@/lib/utils';

export type ExternalToolName = 'gdb' | 'openocd' | 'ghidra' | 'hxd' | 'terminal';

export interface GdbExternalToolLaunchersProps {
  onLaunchTool?: (tool: ExternalToolName) => Promise<void>;
}

export const GdbExternalToolLaunchers: React.FC<GdbExternalToolLaunchersProps> = ({
  onLaunchTool,
}) => {
  const [activeTool, setActiveTool] = useState<string | null>(null);

  const handleLaunch = async (tool: ExternalToolName) => {
    if (!onLaunchTool) return;
    setActiveTool(tool);
    try {
      await onLaunchTool(tool);
    } finally {
      setTimeout(() => setActiveTool(null), 1200);
    }
  };

  return (
    <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 bg-slate-950/60 p-3 rounded-xl border border-slate-800/80">
      <span className="text-xs font-semibold text-slate-300">
        Disparo Rápido de Ferramentas de Engenharia Reversa:
      </span>
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
  );
};
