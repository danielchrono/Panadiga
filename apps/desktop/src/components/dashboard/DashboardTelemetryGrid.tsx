import React from 'react';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { useTranslation } from '@/i18n/useTranslation';
import { HardDrive, Cpu, Radio, Lock } from 'lucide-react';

export const DashboardTelemetryGrid: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 font-mono">
      <MetricCard
        title={t('dashboard.metricSpansion')}
        value="8.00"
        unit="MB"
        subtitle={t('dashboard.metricSpansionSub')}
        variant="amber"
        icon={<Cpu className="w-4 h-4" />}
        badge={<StatusBadge variant="warning" label={t('dashboard.metricSpansionBadge')} />}
      />
      <MetricCard
        title={t('dashboard.metricHdd')}
        value="100.0"
        unit="MB"
        subtitle={t('dashboard.metricHddSub')}
        variant="cyan"
        icon={<HardDrive className="w-4 h-4" />}
        badge={<StatusBadge variant="success" label={t('dashboard.metricHddBadge')} />}
      />
      <MetricCard
        title={t('dashboard.metricFrm')}
        value="7.99958"
        unit="bits/byte"
        subtitle={t('dashboard.metricFrmSub')}
        variant="slate"
        icon={<Lock className="w-4 h-4" />}
        badge={<StatusBadge variant="info" label={t('dashboard.metricFrmBadge')} />}
      />
      <MetricCard
        title={t('dashboard.metricDwire')}
        value="3.28"
        unit="V (ADC0)"
        subtitle={t('dashboard.metricDwireSub')}
        variant="emerald"
        icon={<Radio className="w-4 h-4" />}
        badge={<StatusBadge variant="success" label={t('dashboard.metricDwireBadge')} />}
      />
    </div>
  );
};
