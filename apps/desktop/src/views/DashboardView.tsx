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
import { AlertCircle, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface DashboardViewProps {
  onNavigate: (view: ViewKey) => void;
  onQuickClean?: () => void;
}

export const DashboardView: React.FC<DashboardViewProps> = ({ onNavigate }) => {
  const { overview, loading, error, refresh } = useDashboardViewModel();
  const { t } = useTranslation();

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-96 gap-3 text-slate-500 font-mono text-xs">
        <RefreshCw className="w-6 h-6 animate-spin text-cyan-400" />
        <span>{t('dashboard.loading')}</span>
      </div>
    );
  }

  if (!overview) {
    return (
      <div className="max-w-2xl mx-auto my-16 p-6 rounded-2xl bg-bench-900 border border-slate-800 text-center space-y-4 font-mono">
        <div className="w-12 h-12 rounded-full bg-rose-500/10 text-rose-400 border border-rose-500/30 flex items-center justify-center mx-auto">
          <AlertCircle className="w-6 h-6" />
        </div>
        <div>
          <h3 className="text-sm font-bold text-slate-100">
            Falha na Conexão com o Kernel do Sistema
          </h3>
          <p className="text-xs text-slate-400 mt-1">
            {error || 'Não foi possível carregar o resumo da bancada do dispositivo.'}
          </p>
        </div>
        <Button variant="emerald" onClick={refresh} className="gap-2 mx-auto">
          <RefreshCw className="w-4 h-4" />
          <span>Tentar Novamente</span>
        </Button>
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
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 items-stretch">
        <DashboardArchitectureCard overview={overview} />
        <DashboardBinariesTable binaries={overview.binaries} />
      </div>
    </div>
  );
};
