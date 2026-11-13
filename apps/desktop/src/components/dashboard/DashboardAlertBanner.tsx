import React from 'react';
import { ShieldAlert, ArrowRight } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { Button } from '@/components/ui';
import { useTranslation } from '@/i18n/useTranslation';

interface DashboardAlertBannerProps {
  errorCode: string;
  errorDescription: string;
  onNavigateNvram: () => void;
}

export const DashboardAlertBanner: React.FC<DashboardAlertBannerProps> = ({
  errorCode,
  errorDescription,
  onNavigateNvram,
}) => {
  const { t } = useTranslation();

  return (
    <div className="rounded-2xl bg-gradient-to-r from-amber-500/10 via-amber-500/5 to-transparent border border-amber-500/30 p-5 shadow-xl relative overflow-hidden font-mono">
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
        <div className="flex items-start gap-3.5">
          <div className="p-2.5 rounded-xl bg-amber-500/20 text-amber-400 border border-amber-500/30 shrink-0">
            <ShieldAlert className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-sm font-bold font-mono text-amber-300">
                {t('dashboard.alertTitle', { errorCode })}
              </h2>
              <StatusBadge variant="warning" label={t('dashboard.alertBadge')} />
            </div>
            <p className="text-xs text-slate-300 mt-1 max-w-3xl leading-relaxed font-sans">
              {errorDescription}
            </p>
          </div>
        </div>

        <Button
          onClick={onNavigateNvram}
          aria-label={t('dashboard.cleanButton')}
          className="bg-amber-500 hover:bg-amber-400 text-slate-950 font-mono font-bold text-xs gap-2 shadow-lg shadow-amber-950/40 shrink-0"
        >
          <span>{t('dashboard.cleanButton')}</span>
          <ArrowRight className="w-4 h-4" />
        </Button>
      </div>
    </div>
  );
};
