import React from 'react';
import { FileText } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { ManualSearchResult } from '@/models';
import { useTranslation } from '@/i18n/useTranslation';

interface ManualsResultsListProps {
  searchResults: ManualSearchResult[];
  loading: boolean;
}

export const ManualsResultsList: React.FC<ManualsResultsListProps> = ({
  searchResults,
  loading,
}) => {
  const { t } = useTranslation();

  return (
    <div className="space-y-4 font-mono">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
          <FileText className="w-4 h-4 text-cyan-400" />
          <span>{t('manuals.results.title')} ({searchResults.length})</span>
        </h3>
      </div>

      {loading ? (
        <div className="p-8 text-center text-slate-500 text-xs font-mono">
          {t('manuals.results.loading')}
        </div>
      ) : searchResults.length === 0 ? (
        <div className="p-8 rounded-xl bg-bench-900 border border-slate-800 text-center text-slate-500 text-xs">
          {t('manuals.results.empty')}
        </div>
      ) : (
        searchResults.map(result => (
          <div
            key={result.file_name}
            className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-3"
          >
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2 border-b border-slate-800 pb-3">
              <div>
                <h4 className="text-sm font-bold text-slate-200 flex items-center gap-2">
                  <FileText className="w-4 h-4 text-blue-400 shrink-0" />
                  <span>{result.title}</span>
                </h4>
                <span className="text-[11px] text-slate-500 block font-mono">
                  {result.file_name} • {result.total_pages} {t('manuals.results.pages')}
                </span>
              </div>
              <StatusBadge
                variant="info"
                label={`${result.total_match_count} ${t('manuals.results.occurrences')}`}
              />
            </div>

            {/* Match badges */}
            <div className="flex flex-wrap gap-1.5">
              {Object.keys(result.matches).length === 0 ? (
                <span className="text-[11px] text-slate-600">Sem termos associados</span>
              ) : (
                Object.entries(result.matches).map(([term, pages]) => (
                  <span
                    key={term}
                    className="px-2 py-0.5 rounded bg-slate-950 border border-slate-800 text-[11px] text-slate-300 font-mono"
                  >
                    <span className="text-amber-400 font-bold">{term}</span>: {t('manuals.results.pages')} {pages.slice(0, 5).join(', ')}
                    {pages.length > 5 ? '...' : ''}
                  </span>
                ))
              )}
            </div>

            {/* Snippets */}
            <div className="space-y-2 pt-1">
              {result.snippets.length === 0 ? (
                <div className="text-[11px] text-slate-600 italic">Nenhum trecho de texto disponível</div>
              ) : (
                result.snippets.map((snip, idx) => (
                  <div
                    key={idx}
                    className="p-3 rounded-lg bg-slate-950 border border-slate-800/80 text-xs text-slate-300 space-y-1"
                  >
                    <div className="flex items-center justify-between text-[10px] text-slate-500">
                      <span className="text-cyan-400 font-bold">{t('rag.line')} #{snip.page}</span>
                      <span className="text-amber-400 font-mono">{snip.term}</span>
                    </div>
                    <p className="text-[11px] text-slate-300 font-sans leading-relaxed">
                      "{snip.context}"
                    </p>
                  </div>
                ))
              )}
            </div>
          </div>
        ))
      )}
    </div>
  );
};
