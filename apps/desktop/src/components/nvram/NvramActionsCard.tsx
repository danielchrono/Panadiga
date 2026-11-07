import React from 'react';
import { Cpu, ShieldCheck, RefreshCw, FileCode } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { Button } from '@/components/ui/button';

interface NvramActionsCardProps {
  cleaning: boolean;
  cleanSuccess: boolean;
  onExecuteClean: () => void;
  onRefresh: () => void;
}

export const NvramActionsCard: React.FC<NvramActionsCardProps> = ({
  cleaning,
  cleanSuccess,
  onExecuteClean,
  onRefresh,
}) => (
  <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
    <div className="flex items-center gap-3.5">
      <div className="p-3 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/30">
        <Cpu className="w-6 h-6" />
      </div>
      <div>
        <div className="flex items-center gap-2">
          <h2 className="text-base font-bold text-slate-100">
            Spansion S29JL064H NOR Flash (8 MB / 64 Mbit, TSOP-48)
          </h2>
          <StatusBadge variant="info" label="TOP-BOOT" />
          <StatusBadge variant="neutral" label="TSOP-48" />
        </div>
        <p className="text-xs text-slate-400 mt-1">
          Offset NVRAM: <code className="text-cyan-400">0x007F_0000..0x007F_FFFF</code> (64 KB). Partição de microsetores de 8 KB (SA134..SA141).
        </p>
      </div>
    </div>

    <div className="flex items-center gap-3 shrink-0">
      <Button
        variant="ghost"
        size="icon"
        onClick={onRefresh}
        title="Recarregar análise da flash"
        aria-label="Recarregar análise da flash"
      >
        <RefreshCw className="w-4 h-4 text-slate-400" />
      </Button>

      <Button
        variant="emerald"
        onClick={onExecuteClean}
        disabled={cleaning || cleanSuccess}
        className="gap-2"
      >
        {cleaning ? (
          <>
            <RefreshCw className="w-4 h-4 animate-spin" />
            <span>Higienizando Microsetores...</span>
          </>
        ) : cleanSuccess ? (
          <>
            <ShieldCheck className="w-4 h-4" />
            <span>Flash Higienizada (0xFF)</span>
          </>
        ) : (
          <>
            <FileCode className="w-4 h-4" />
            <span>Executar Cirurgia no Dump</span>
          </>
        )}
      </Button>
    </div>
  </div>
);
