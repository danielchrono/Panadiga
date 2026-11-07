import React from 'react';
import { Lock } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { useTranslation } from '@/i18n/useTranslation';

export const FrmTopHeader: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl font-mono">
      <div className="flex items-center gap-3.5">
        <div className="p-3 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/30">
          <Lock className="w-6 h-6" />
        </div>
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-base font-bold text-slate-100">
              {t('crypto.title')}
            </h2>
            <StatusBadge variant="info" label={t('crypto.badge')} />
          </div>
          <p className="text-xs text-slate-400 mt-0.5">
            {t('crypto.subtitle')}
          </p>
        </div>
      </div>

      <div className="flex items-center gap-2 shrink-0">
        <span className="text-xs text-slate-400">
          {t('crypto.signature')}{' '}
          <span className="text-purple-400 font-bold">50 41 4E 41 5F 46 52 4D</span>
        </span>
      </div>
    </div>
  );
};
