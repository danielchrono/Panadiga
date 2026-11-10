import React from 'react';
import { Cpu, ShieldCheck, RefreshCw, FileCode } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { Button } from '@/components/ui/button';
import { NvramCleanReport } from '@/models';

interface NvramActionsCardProps {
  report: NvramCleanReport | null;
  cleaning: boolean;
  cleanSuccess: boolean;
  onExecuteClean: () => void;
  onRefresh: () => void;
}

export const NvramActionsCard: React.FC<NvramActionsCardProps> = ({
  report,
  cleaning,
  cleanSuccess,
  onExecuteClean,
  onRefresh,
}) => {
  const flashBytes = report?.total_flash_bytes ?? 0;
  const flashMb = flashBytes > 0 ? (flashBytes / (1024 * 1024)).toFixed(0) : '8';
  const flashMbit = flashBytes > 0 ? (flashBytes * 8) / (1024 * 1024) : 64;

  const startHex = report
    ? `0x${report.nvram_offset_start.toString(16).toUpperCase().padStart(8, '0')}`
    : '0x007F0000';
  const endHex = report
    ? `0x${(report.nvram_offset_end - 1).toString(16).toUpperCase().padStart(8, '0')}`
    : '0x007FFFFF';

  const flashName =
    flashBytes === 0x0020_0000
      ? 'Spansion S29AL016J NOR Flash (2 MB / 16 Mbit)'
      : flashBytes === 0x0040_0000
        ? 'Spansion S29JL032H NOR Flash (4 MB / 32 Mbit)'
        : flashBytes === 0x0100_0000
          ? 'Spansion S29GL128P NOR Flash (16 MB / 128 Mbit)'
          : 'Spansion S29JL064H NOR Flash (8 MB / 64 Mbit)';

  return (
    <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
      <div className="flex items-center gap-3.5">
        <div className="p-3 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/30">
          <Cpu className="w-6 h-6" />
        </div>
        <div>
          <div className="flex items-center gap-2 flex-wrap">
            <h2 className="text-base font-bold text-slate-100">{flashName}</h2>
            <StatusBadge variant="info" label="TOP-BOOT" />
            <StatusBadge variant="neutral" label="TSOP-48 / BGA" />
          </div>
          <p className="text-xs text-slate-400 mt-1">
            Offset EEMUL NVRAM: <code className="text-cyan-400">{startHex}..{endHex}</code> (64 KB).{' '}
            {report?.microsectors.length ?? 8} microsetores de 8 KB com checksum e cabeçalhos emulados.
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
          disabled={cleaning || cleanSuccess || !report}
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
              <span>Limpeza Cirúrgica (U99 Reset)</span>
            </>
          )}
        </Button>
      </div>
    </div>
  );
};
