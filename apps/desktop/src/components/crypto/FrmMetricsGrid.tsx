import React from 'react';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { CryptanalysisReport, FrmTocDumpReport } from '@/models';
import { useTranslation } from '@/i18n/useTranslation';

interface FrmMetricsGridProps {
  cryptReport: CryptanalysisReport;
  tocReport?: FrmTocDumpReport | null;
}

export const FrmMetricsGrid: React.FC<FrmMetricsGridProps> = ({ cryptReport, tocReport }) => {
  const { t } = useTranslation();
  const topBlockCount = cryptReport.top_blocks?.[0]?.count ?? 0;
  const containerMb = (cryptReport.file_size_bytes / (1024 * 1024)).toFixed(2);
  const slotCount =
    tocReport?.decrypted_toc && tocReport.decrypted_toc.length > 0
      ? tocReport.decrypted_toc.length
      : (tocReport?.toc_entries?.length ?? 0);
  const tocBadge =
    tocReport?.toc_format === 'sd-plaintext'
      ? '64B SD TOC'
      : tocReport?.toc_format === 'dvd-feistel'
        ? '48B FEISTEL'
        : t('crypto.metrics.cpusBadge');

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
        value={topBlockCount.toLocaleString()}
        unit="blocos"
        subtitle={t('crypto.metrics.ecbLeaksSub')}
        variant={cryptReport.is_ecb_leak_confirmed ? 'amber' : 'slate'}
        badge={
          <StatusBadge
            variant={cryptReport.is_ecb_leak_confirmed ? 'warning' : 'neutral'}
            label={
              cryptReport.is_ecb_leak_confirmed
                ? t('crypto.metrics.ecbBadge')
                : t('crypto.metrics.ecbCleanBadge')
            }
          />
        }
      />
      <MetricCard
        title={t('crypto.metrics.size')}
        value={cryptReport.file_size_bytes.toLocaleString()}
        unit="bytes"
        subtitle={`${cryptReport.frm_file} (${containerMb} MB)`}
        variant="slate"
      />
      <MetricCard
        title={t('crypto.metrics.cpus')}
        value={String(slotCount)}
        unit="módulos"
        subtitle={t('crypto.metrics.cpusSub')}
        variant="cyan"
        badge={<StatusBadge variant="success" label={tocBadge} />}
      />
    </div>
  );
};
