import React from 'react';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { ShieldCheck, AlertOctagon, Layers, ArrowRight } from 'lucide-react';
import { NvramCleanReport, FlashSectorInfo } from '@/models';

interface NvramMetricsGridProps {
  report: NvramCleanReport | null;
  selectedSector: FlashSectorInfo | null;
  cleanSuccess: boolean;
}

export const NvramMetricsGrid: React.FC<NvramMetricsGridProps> = ({
  report,
  selectedSector,
  cleanSuccess,
}) => {
  const flashBytes = report?.total_flash_bytes ?? 0;
  const flashKb = Math.round(flashBytes / 1024);
  const flashMb = (flashBytes / (1024 * 1024)).toFixed(0);
  const flashMbit = flashBytes > 0 ? (flashBytes * 8) / (1024 * 1024) : 0;

  const totalMicrosectors = report?.microsectors.length ?? 8;
  const corruptedCount = cleanSuccess
    ? 0
    : (report?.microsectors.filter(m => m.active_bytes > 0).length ?? 0);

  const isCorrupted = !cleanSuccess && corruptedCount > 0;

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <MetricCard
        title="Status da NVRAM"
        value={
          cleanSuccess
            ? 'VIRGEM'
            : report
              ? isCorrupted
                ? 'CORROMPIDA'
                : 'LIMPO'
              : 'AGUARDANDO DUMP'
        }
        subtitle={
          cleanSuccess
            ? 'Pronto para regravar na flash'
            : report
              ? isCorrupted
                ? `Erro U99 ativo (${corruptedCount} microsetores afetados)`
                : 'NVRAM já limpa (0xFF)'
              : 'Nenhum dump carregado'
        }
        variant={cleanSuccess ? 'emerald' : isCorrupted ? 'rose' : 'cyan'}
        icon={
          cleanSuccess ? (
            <ShieldCheck className="w-5 h-5 text-emerald-400" />
          ) : (
            <AlertOctagon
              className={`w-5 h-5 ${isCorrupted ? 'text-rose-400' : 'text-cyan-400'}`}
            />
          )
        }
        badge={
          <StatusBadge
            variant={cleanSuccess ? 'success' : isCorrupted ? 'error' : 'neutral'}
            label={cleanSuccess ? 'LIMPO 0xFF' : isCorrupted ? 'U99 LOOP' : 'STATUS OK'}
          />
        }
      />

      <MetricCard
        title="Tamanho da Flash NOR"
        value={flashKb > 0 ? flashKb.toLocaleString() : '---'}
        unit={flashKb > 0 ? 'KB' : undefined}
        subtitle={
          flashBytes > 0
            ? `${flashMb} MB Flash NOR (Base EEMUL @ 0x${(report?.nvram_offset_start ?? 0).toString(16).toUpperCase()})`
            : 'Aguardando detecção'
        }
        variant="amber"
        icon={<Layers className="w-5 h-5 text-amber-400" />}
        badge={
          <StatusBadge
            variant="warning"
            label={flashMbit > 0 ? `${flashMbit} MBIT` : 'DETECTANDO'}
          />
        }
      />

      <MetricCard
        title="Setores Corrompidos"
        value={report ? corruptedCount : '---'}
        unit={`/ ${totalMicrosectors} microsetores`}
        subtitle={
          cleanSuccess
            ? 'Área 100% preenchida com 0xFF'
            : report
              ? isCorrupted
                ? `${report.nvram_non_ff_bytes.toLocaleString()} bytes corrompidos serão purgados`
                : 'Área EEMUL já limpa (0xFF)'
              : 'Aguardando análise'
        }
        variant={cleanSuccess ? 'emerald' : isCorrupted ? 'rose' : 'cyan'}
        badge={
          <StatusBadge
            variant={cleanSuccess ? 'success' : isCorrupted ? 'error' : 'neutral'}
            label={
              cleanSuccess
                ? '0 B CORROMPIDOS'
                : isCorrupted
                  ? `${report?.nvram_non_ff_bytes ?? 0} B U99`
                  : 'SEM CORRUPÇÃO'
            }
          />
        }
      />

      <MetricCard
        title="Setor em Inspeção"
        value={selectedSector?.name ?? '---'}
        subtitle={
          selectedSector
            ? `0x${selectedSector.start_offset.toString(16).toUpperCase().padStart(8, '0')} (${(selectedSector.size_bytes / 1024).toFixed(0)} KB)`
            : 'Selecione um setor no mapa'
        }
        variant="cyan"
        icon={<ArrowRight className="w-5 h-5 text-cyan-400" />}
        badge={
          <StatusBadge
            variant={selectedSector?.is_corrupted_area ? 'warning' : 'neutral'}
            label={selectedSector?.sector_type.toUpperCase() ?? 'NENHUM'}
          />
        }
      />
    </div>
  );
};
