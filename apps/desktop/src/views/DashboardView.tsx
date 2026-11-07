import React from 'react';
import { useDashboardViewModel } from '@/viewmodels/useDashboardViewModel';
import { ViewKey } from '@/components/Sidebar';
import { useTranslation } from '@/i18n/useTranslation';
import {
  DashboardAlertBanner,
  DashboardTelemetryGrid,
  DashboardInstruments,
  DashboardArchitectureCard,
  DashboardBinariesTable,
} from '@/components/dashboard';

interface DashboardViewProps {
  onNavigate: (view: ViewKey) => void;
  onQuickClean?: () => void;
}

export const DashboardView: React.FC<DashboardViewProps> = ({ onNavigate }) => {
  const { overview, loading } = useDashboardViewModel();
  const { t } = useTranslation();

  if (loading || !overview) {
    return (
      <div className="flex items-center justify-center h-full text-slate-500 font-mono text-xs">
        {t('dashboard.loading')}
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Alert Banner: Root cause analysis of U99 */}
      <DashboardAlertBanner
        errorCode={overview.error_code}
        errorDescription={overview.error_description}
        onNavigateNvram={() => onNavigate('nvram')}
      />

      {/* Top Telemetry Metric Cards */}
      <DashboardTelemetryGrid />

      {/* Quick Launch Instruments */}
      <DashboardInstruments onNavigate={onNavigate} />

      {/* Binary Status & Hardware Table */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <DashboardArchitectureCard overview={overview} />
        <DashboardBinariesTable binaries={overview.binaries} />
      </div>
    </div>
  );
};
