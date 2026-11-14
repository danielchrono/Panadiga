import React from 'react';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { useTranslation } from '@/i18n/useTranslation';
import { HardDrive, Cpu, Radio, Lock } from 'lucide-react';
import { SystemOverview } from '@/models';

export interface DashboardTelemetryGridProps {
  overview?: SystemOverview | null;
}

export const DashboardTelemetryGrid: React.FC<DashboardTelemetryGridProps> = ({ overview }) => {
  const { t } = useTranslation();

  const flashBin = overview?.binaries?.find(b => b.id.includes('flash') || b.name.toLowerCase().includes('flash'));
  const hddBin = overview?.binaries?.find(b => b.id.includes('hdd') || b.name.toLowerCase().includes('hdd'));
  const frmBin = overview?.binaries?.find(b => b.id.includes('frm') || b.name.toLowerCase().includes('frm'));

  const flashMb = flashBin?.size_bytes ? (flashBin.size_bytes / (1024 * 1024)).toFixed(2) : '--';
  const hddMb = hddBin?.size_bytes ? (hddBin.size_bytes / (1024 * 1024)).toFixed(1) : '--';
  const frmMb = frmBin?.size_bytes ? (frmBin.size_bytes / (1024 * 1024)).toFixed(2) : '--';

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 font-mono">
      <MetricCard
        title={t('dashboard.metricSpansion', 'Flash NOR Spansion')}
        value={flashMb}
        unit="MB"
        subtitle={flashBin?.exists ? flashBin.name : 'Dump de Flash NOR'}
        variant={flashBin?.exists ? 'amber' : 'slate'}
        icon={<Cpu className="w-4 h-4" />}
        badge={
          <StatusBadge
            variant={flashBin?.exists ? 'warning' : 'neutral'}
            label={flashBin?.exists ? 'MICROSETORES' : 'AUSENTE'}
          />
        }
      />
      <MetricCard
        title={t('dashboard.metricHdd', 'Imagem HDD Forense')}
        value={hddMb}
        unit="MB"
        subtitle={hddBin?.exists ? hddBin.name : 'Nenhuma imagem física'}
        variant={hddBin?.exists ? 'cyan' : 'slate'}
        icon={<HardDrive className="w-4 h-4" />}
        badge={
          <StatusBadge
            variant={hddBin?.exists ? 'success' : 'neutral'}
            label={hddBin?.exists ? 'MEIHDFS OK' : 'OFFLINE'}
          />
        }
      />
      <MetricCard
        title={t('dashboard.metricFrm', 'Firmware PANA_DVD.FRM')}
        value={frmMb}
        unit={frmBin?.exists ? 'MB' : ''}
        subtitle={frmBin?.exists ? frmBin.name : 'Container oficial de update'}
        variant={frmBin?.exists ? 'slate' : 'slate'}
        icon={<Lock className="w-4 h-4" />}
        badge={
          <StatusBadge
            variant={frmBin?.exists ? 'info' : 'neutral'}
            label={frmBin?.exists ? 'ENCRIPTADO' : 'AUSENTE'}
          />
        }
      />
      <MetricCard
        title={overview?.debug_port || 'P7501 UART 115200 8N1'}
        value="115.2"
        unit="kbps"
        subtitle="P7501 Async UART (Legacy PIO 1 MHz)"
        variant="emerald"
        icon={<Radio className="w-4 h-4" />}
        badge={<StatusBadge variant="neutral" label="STANDBY" />}
      />
    </div>
  );
};
