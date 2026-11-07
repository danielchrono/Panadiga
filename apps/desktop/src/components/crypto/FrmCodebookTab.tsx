import React from 'react';
import { KeyRound, Search } from 'lucide-react';
import { CodebookEntry } from '@/models';
import { useTranslation } from '@/i18n/useTranslation';

interface FrmCodebookTabProps {
  filteredCodebook: CodebookEntry[];
  codebookFilter: string;
  setCodebookFilter: (v: string) => void;
}

export const FrmCodebookTab: React.FC<FrmCodebookTabProps> = ({
  filteredCodebook,
  codebookFilter,
  setCodebookFilter,
}) => {
  const { t } = useTranslation();

  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4 font-mono">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <KeyRound className="w-4 h-4 text-purple-400" />
            <span>{t('crypto.codebookSection.title')}</span>
          </h3>
          <p className="text-xs text-slate-400 mt-0.5">
            {t('crypto.codebookSection.desc')}
          </p>
        </div>

        <div className="relative">
          <Search className="w-3.5 h-3.5 absolute left-2.5 top-2.5 text-slate-500" />
          <input
            type="text"
            aria-label={t('crypto.codebookSection.searchPlaceholder')}
            placeholder={t('crypto.codebookSection.searchPlaceholder')}
            value={codebookFilter}
            onChange={e => setCodebookFilter(e.target.value)}
            className="pl-8 pr-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-xs text-slate-200 focus:outline-none focus:border-purple-500 focus:ring-1 focus:ring-purple-500/50"
          />
        </div>
      </div>

      <div className="overflow-x-auto">
        {filteredCodebook.length === 0 ? (
          <div className="p-8 text-center text-xs text-slate-500 font-mono rounded-xl bg-slate-950 border border-slate-800">
            {t('crypto.codebookSection.empty')}
          </div>
        ) : (
          <table className="w-full text-left text-xs font-mono">
            <thead>
              <tr className="border-b border-slate-800 text-slate-500 uppercase text-[10px]">
                <th className="py-2.5 px-3">Rank</th>
                <th className="py-2.5 px-3">Bloco Cifrado C_i (64b)</th>
                <th className="py-2.5 px-3">Texto Claro P_i (KPA)</th>
                <th className="py-2.5 px-3">Ocorrências (.FRM)</th>
                <th className="py-2.5 px-3">Ocorrências (HD)</th>
                <th className="py-2.5 px-3">Interpretação Semântica</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60">
              {filteredCodebook.map(entry => (
                <tr key={entry.block_hex} className="hover:bg-slate-800/40 transition-colors">
                  <td className="py-2.5 px-3 font-bold text-slate-400">
                    #{entry.rank}
                  </td>
                  <td className="py-2.5 px-3 font-bold text-purple-300">
                    {entry.block_hex}
                  </td>
                  <td className="py-2.5 px-3 text-emerald-400 font-bold">
                    {entry.known_plaintext_hex ?? t('crypto.codebookSection.waiting')}
                  </td>
                  <td className="py-2.5 px-3 text-slate-300">
                    {entry.frm_occurrences.toLocaleString()}x ({entry.frm_total_bytes} B)
                  </td>
                  <td className="py-2.5 px-3 text-cyan-400">
                    {entry.hdd_occurrences.toLocaleString()}x
                  </td>
                  <td className="py-2.5 px-3 text-slate-400 text-[11px]">
                    {entry.interpretation}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
};
