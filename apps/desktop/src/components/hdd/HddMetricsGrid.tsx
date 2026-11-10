import React from 'react';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { useTranslation } from '@/i18n/useTranslation';
import { HddAnalysisReport } from '@/models';

export interface HddMetricsGridProps {
  report?: HddAnalysisReport | null;
}

export const HddMetricsGrid: React.FC<HddMetricsGridProps> = ({ report }) => {
  const { t } = useTranslation();

  if (!report) {
    return (
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title={t('hdd.metrics.imageSize', 'Tamanho da Imagem')}
          value="--"
          unit="MB"
          subtitle="Nenhum arquivo carregado"
          variant="slate"
        />
        <MetricCard
          title={t('hdd.metrics.mirrorMbr', 'Validação MBR / Espelho')}
          value="--"
          unit=""
          subtitle="Aguardando inspeção forense"
          variant="slate"
          badge={<StatusBadge variant="neutral" label="OFFLINE" />}
        />
        <MetricCard
          title="Partição 1"
          value="--"
          unit=""
          subtitle="Estrutura MEIHDFS não analisada"
          variant="slate"
        />
        <MetricCard
          title="Partição 2"
          value="--"
          unit=""
          subtitle="Estrutura MEIHDFS não analisada"
          variant="slate"
        />
      </div>
    );
  }

  const isGb = report.file_size_mb >= 1024;
  const sizeVal = isGb ? (report.file_size_mb / 1024).toFixed(2) : report.file_size_mb.toFixed(2);
  const sizeUnit = isGb ? 'GB' : 'MB';
  const mirrorLbaMb = Math.round((report.mbr_tables.mirror_lba * 512) / (1024 * 1024));
  const isMirrorOk = report.mbr_tables.mirror_is_bit_identical;

  const part1 = report.partitions[0];
  const part2 = report.partitions[1];

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <MetricCard
        title={t('hdd.metrics.imageSize', 'Tamanho da Imagem')}
        value={sizeVal}
        unit={sizeUnit}
        subtitle={`${report.file_size_bytes.toLocaleString()} bytes (${report.total_sectors.toLocaleString()} setores)`}
        variant="cyan"
      />
      <MetricCard
        title={t('hdd.metrics.mirrorMbr', 'Validação MBR / Espelho')}
        value={`LBA ${report.mbr_tables.mirror_lba.toLocaleString()}`}
        unit={`(${mirrorLbaMb} MB)`}
        subtitle={`GUID: ${report.mbr_tables.disk_volume_guid || 'N/A'}`}
        variant={isMirrorOk ? 'emerald' : 'rose'}
        badge={
          isMirrorOk ? (
            <StatusBadge variant="success" label="MATCH 100%" />
          ) : (
            <StatusBadge variant="error" label="DIVERGENTE" />
          )
        }
      />
      <MetricCard
        title={part1 ? part1.name : 'Partição 1'}
        value={part1 ? part1.exact_payload_size.toLocaleString() : '--'}
        unit={part1 ? 'bytes' : ''}
        subtitle={
          part1
            ? `LBA ${part1.lba_start.toLocaleString()} • Entropia ${part1.entropy.toFixed(3)}b`
            : 'Não identificada'
        }
        variant="slate"
      />
      <MetricCard
        title={part2 ? part2.name : 'Partição 2'}
        value={part2 ? part2.exact_payload_size.toLocaleString() : '--'}
        unit={part2 ? 'bytes' : ''}
        subtitle={
          part2
            ? `LBA ${part2.lba_start.toLocaleString()} • ${part2.sectors_count.toLocaleString()} setores`
            : 'Não identificada'
        }
        variant="slate"
      />
    </div>
  );
};
