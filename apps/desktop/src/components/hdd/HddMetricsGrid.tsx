import React from 'react';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { useTranslation } from '@/i18n/useTranslation';

export const HddMetricsGrid: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <MetricCard
        title={t('hdd.metrics.imageSize', 'Tamanho da Imagem')}
        value="100.00"
        unit="MB"
        subtitle="104.857.600 bytes exatos"
        variant="cyan"
      />
      <MetricCard
        title={t('hdd.metrics.mirrorMbr', 'Validação MBR / Espelho')}
        value="LBA 57.344"
        unit="(28 MB)"
        subtitle="Identical Bit-for-Bit Replica"
        variant="emerald"
        badge={<StatusBadge variant="success" label="MATCH 100%" />}
      />
      <MetricCard
        title="Partição 1 (AM34 OS)"
        value="6.173.264"
        unit="bytes"
        subtitle="LBA 65.536 (32 MB) • Entropia 7.994b"
        variant="slate"
      />
      <MetricCard
        title="Partição 2 (Microcódigo)"
        value="720.896"
        unit="bytes"
        subtitle="LBA 131.072 (64 MB) • 1.408 setores"
        variant="slate"
      />
    </div>
  );
};
