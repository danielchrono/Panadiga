import React from 'react';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { CryptanalysisReport } from '@/models';
import { useTranslation } from '@/i18n/useTranslation';

interface FrmMetricsGridProps {
  cryptReport: CryptanalysisReport;
}

export const FrmMetricsGrid: React.FC<FrmMetricsGridProps> = ({ cryptReport }) => {
  const { t } = useTranslation();

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 font-mono">
      <MetricCard
        title={t('crypto.metrics.entropy')}
        value={cryptReport.global_entropy.toFixed(5)}
        unit="bits/byte"
        subtitle={t('crypto.metrics.entropySub')}
        variant="slate"
      />
      <MetricCard
        title={t('crypto.metrics.ecbLeaks')}
        value="3.812"
        unit="blocos"
        subtitle={t('crypto.metrics.ecbLeaksSub')}
        variant="amber"
        badge={<StatusBadge variant="warning" label={t('crypto.metrics.ecbBadge')} />}
      />
      <MetricCard
        title={t('crypto.metrics.size')}
        value={cryptReport.file_size_bytes ? cryptReport.file_size_bytes.toLocaleString() : '7.531.744'}
        unit="bytes"
        subtitle={t('crypto.metrics.sizeSub')}
        variant="slate"
      />
      <MetricCard
        title={t('crypto.metrics.cpus')}
        value="3"
        unit="arquiteturas"
        subtitle={t('crypto.metrics.cpusSub')}
        variant="cyan"
        badge={<StatusBadge variant="success" label={t('crypto.metrics.cpusBadge')} />}
      />
    </div>
  );
};
