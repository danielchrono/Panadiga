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
}) => (
  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
    <MetricCard
      title="Status da NVRAM"
      value={cleanSuccess ? 'VIRGEM' : (report ? 'CORROMPIDA' : 'DESCONHECIDO')}
      subtitle={cleanSuccess ? 'Pronto para regravar na flash' : 'Erro U99 ativo no setor SA137'}
      variant={cleanSuccess ? 'emerald' : 'rose'}
      icon={
        cleanSuccess ? (
          <ShieldCheck className="w-5 h-5 text-emerald-400" />
        ) : (
          <AlertOctagon className="w-5 h-5 text-rose-400" />
        )
      }
      badge={
        <StatusBadge
          variant={cleanSuccess ? 'success' : 'error'}
          label={cleanSuccess ? 'LIMPO 0xFF' : 'U99 LOOP'}
        />
      }
    />

    <MetricCard
      title="Tamanho da Flash NOR"
      value="8.192"
      unit="KB"
      subtitle="71 setores totais (SA0 a SA141)"
      variant="amber"
      icon={<Layers className="w-5 h-5 text-amber-400" />}
      badge={<StatusBadge variant="warning" label="64 MBIT" />}
    />

    <MetricCard
      title="Setores Corrompidos"
      value={cleanSuccess ? '0' : 8}
      unit="/ 8 microsetores"
      subtitle={cleanSuccess ? 'Área 100% preenchida com 0xFF' : '65.536 bytes serão purgados'}
      variant={cleanSuccess ? 'emerald' : 'rose'}
      badge={
        <StatusBadge
          variant={cleanSuccess ? 'success' : 'error'}
          label={cleanSuccess ? '0 B CORROMPIDOS' : '64 KB U99'}
        />
      }
    />

    <MetricCard
      title="Setor em Inspeção"
      value={selectedSector?.name ?? 'SA137'}
      subtitle={`0x${selectedSector?.start_offset.toString(16).toUpperCase()} (${(selectedSector?.size_bytes ?? 8192) / 1024} KB)`}
      variant="cyan"
      icon={<ArrowRight className="w-5 h-5 text-cyan-400" />}
      badge={
        <StatusBadge
          variant={selectedSector?.is_corrupted_area ? 'warning' : 'neutral'}
          label={selectedSector?.sector_type.toUpperCase() ?? 'NVRAM'}
        />
      }
    />
  </div>
);
