import React from 'react';
import { BarChart2 } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { BlockFrequency } from '@/models';
import { useTranslation } from '@/i18n/useTranslation';

interface FrmEntropyTabProps {
  topBlocks: BlockFrequency[];
}

export const FrmEntropyTab: React.FC<FrmEntropyTabProps> = ({ topBlocks }) => {
  const { t } = useTranslation();

  return (
    <div className="space-y-6 font-mono">
      <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <BarChart2 className="w-4 h-4 text-purple-400" />
            <span>{t('crypto.entropySection.title')}</span>
          </h3>
          <p className="text-xs text-slate-400 mt-0.5">
            {t('crypto.entropySection.desc')}
          </p>
        </div>

        <div className="space-y-3">
          {topBlocks.length === 0 ? (
            <div className="p-8 text-center text-xs text-slate-500 font-mono rounded-xl bg-slate-950 border border-slate-800">
              {t('crypto.entropySection.empty')}
            </div>
          ) : (
            topBlocks.map((blk, idx) => (
              <div
                key={blk.hex}
                className="p-3.5 rounded-xl bg-slate-950 border border-slate-800 flex flex-col md:flex-row items-start md:items-center justify-between gap-3 text-xs"
              >
                <div className="flex items-center gap-3">
                  <span className="w-6 h-6 rounded-lg bg-slate-800 text-slate-400 font-bold flex items-center justify-center text-xs">
                    #{idx + 1}
                  </span>
                  <div>
                    <span className="text-purple-300 font-bold text-sm tracking-wider">
                      {blk.hex}
                    </span>
                    <p className="text-slate-400 text-[11px] mt-0.5">{blk.interpretation}</p>
                  </div>
                </div>

                <div className="flex items-center gap-4 text-xs font-mono shrink-0">
                  <div className="text-right">
                    <span className="text-slate-200 font-bold">{blk.count.toLocaleString()} ocorrências</span>
                    <span className="text-slate-500 block text-[10px]">
                      ({blk.total_bytes.toLocaleString()} bytes totais)
                    </span>
                  </div>
                  <StatusBadge
                    variant={blk.count > 1000 ? 'warning' : 'neutral'}
                    label={blk.count > 1000 ? t('crypto.entropySection.highRep') : t('crypto.entropySection.frequent')}
                  />
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
