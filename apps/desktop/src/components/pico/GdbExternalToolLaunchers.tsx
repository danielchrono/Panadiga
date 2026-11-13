import React, { useMemo, useState } from 'react';
import { Bug, FileCode, Binary, Terminal, type LucideIcon } from 'lucide-react';
import { Button } from '@/components/ui';
import { isAndroidNativeBridgeAvailable } from '@/services/androidBridge';

export type ExternalToolName = 'gdb' | 'ghidra' | 'hxd' | 'terminal';

export interface GdbExternalToolLaunchersProps {
  onLaunchTool?: (tool: ExternalToolName) => Promise<void>;
}

const TOOL_LAUNCHERS: Array<{
  id: ExternalToolName;
  label: string;
  title: string;
  className: string;
  Icon: LucideIcon;
}> = [
  {
    id: 'gdb',
    label: 'GDB Multiarch',
    title: 'Abre terminal interativo com gdb-multiarch conectado na porta 3333',
    className: 'bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-300 border-cyan-500/30',
    Icon: Bug,
  },
  {
    id: 'ghidra',
    label: 'Scripts Ghidra',
    title: 'Inicia o Ghidra / exportação de mapa de memória e símbolos AM34',
    className: 'bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-300 border-emerald-500/30',
    Icon: FileCode,
  },
  {
    id: 'hxd',
    label: 'Editor Hexadecimal',
    title: 'Abre o editor hexadecimal configurado no sistema com os binários da bancada',
    className: 'bg-slate-800 hover:bg-slate-700 text-slate-200 border-slate-700',
    Icon: Binary,
  },
  {
    id: 'terminal',
    label: 'Terminal Host',
    title: 'Abre o shell nativo do sistema operacional no diretório do projeto',
    className: 'bg-slate-800 hover:bg-slate-700 text-slate-300 border-slate-700',
    Icon: Terminal,
  },
];

export const GdbExternalToolLaunchers: React.FC<GdbExternalToolLaunchersProps> = ({
  onLaunchTool,
}) => {
  const [activeTool, setActiveTool] = useState<string | null>(null);
  const androidHost = useMemo(() => isAndroidNativeBridgeAvailable(), []);

  if (androidHost) {
    return (
      <div className="bg-slate-950/60 p-3 rounded-xl border border-slate-800/80 text-xs font-mono text-slate-400">
        Ferramentas desktop externas (GDB CLI / Ghidra GUI) ficam disponíveis no host Desktop (Windows / Linux / macOS).
      </div>
    );
  }

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
        {TOOL_LAUNCHERS.length === 0 ? (
          <span className="text-xs text-slate-500">Nenhuma ferramenta externa configurada.</span>
        ) : (
          TOOL_LAUNCHERS.map(({ id, label, title, className, Icon }) => (
            <Button
              key={id}
              type="button"
              variant="outline"
              size="sm"
              onClick={() => handleLaunch(id)}
              disabled={activeTool === id}
              title={title}
              className={`font-mono font-semibold text-xs gap-1.5 h-8 ${className}`}
            >
              <Icon className="w-3.5 h-3.5" />
              <span>{label}</span>
            </Button>
          ))
        )}
      </div>
    </div>
  );
};
