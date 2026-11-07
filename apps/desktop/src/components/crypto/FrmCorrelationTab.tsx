import React from 'react';
import { FileCheck2 } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { PartitionCorrelation } from '@/models';
import { useTranslation } from '@/i18n/useTranslation';

interface FrmCorrelationTabProps {
  correlations: PartitionCorrelation[];
}

export const FrmCorrelationTab: React.FC<FrmCorrelationTabProps> = ({ correlations }) => {
  const { t } = useTranslation();

  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4 font-mono">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <FileCheck2 className="w-4 h-4 text-emerald-400" />
            <span>{t('crypto.correlationSection.title')}</span>
          </h3>
          <p className="text-xs text-slate-400 mt-0.5">
            {t('crypto.correlationSection.desc')}
          </p>
        </div>
        <StatusBadge variant="success" label={t('crypto.correlationSection.badge')} />
      </div>

      <div className="space-y-3">
        {correlations.length === 0 ? (
          <div className="p-8 text-center text-xs text-slate-500 font-mono rounded-xl bg-slate-950 border border-slate-800">
            {t('crypto.correlationSection.empty')}
          </div>
        ) : (
          correlations.map(corr => (
            <div
              key={corr.offset_hex}
              className="p-4 rounded-xl bg-slate-950 border border-slate-800 text-xs font-mono space-y-2"
            >
              <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                <span className="font-bold text-slate-200">{corr.partition_name}</span>
                <span className="text-cyan-400">{corr.offset_hex}</span>
              </div>
              <div className="space-y-1 text-slate-400">
                <div className="flex justify-between">
                  <span>Bloco Cifrado Identificado:</span>
                  <span className="text-purple-300 font-bold">{corr.block_hex}</span>
                </div>
                <div className="flex justify-between">
                  <span>Frequência no PANA_DVD.FRM:</span>
                  <span className="text-emerald-400 font-bold">{corr.frm_occurrences} repetições</span>
                </div>
                <div className="flex justify-between">
                  <span>Interpretação:</span>
                  <span className="text-slate-300">{corr.interpretation}</span>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
