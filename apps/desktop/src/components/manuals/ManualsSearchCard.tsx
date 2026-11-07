import React from 'react';
import { Search, Sparkles } from 'lucide-react';
import { useTranslation } from '@/i18n/useTranslation';

interface ManualsSearchCardProps {
  searchQuery: string;
  setSearchQuery: (q: string) => void;
  activeTerms: string[];
  executeSearch: (q: string) => void;
  selectTerm: (term: string) => void;
}

export const ManualsSearchCard: React.FC<ManualsSearchCardProps> = ({
  searchQuery,
  setSearchQuery,
  activeTerms,
  executeSearch,
  selectTerm,
}) => {
  const { t } = useTranslation();

  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4 font-mono">
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
        <div className="relative flex-1">
          <Search className="w-4 h-4 absolute left-3 top-3 text-slate-400" />
          <input
            type="text"
            placeholder={t('manuals.search.placeholder')}
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && executeSearch(searchQuery)}
            aria-label={t('manuals.search.placeholder')}
            className="w-full pl-9 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-100 placeholder-slate-500 focus:outline-none focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500/50 font-mono"
          />
        </div>

        <button
          onClick={() => executeSearch(searchQuery)}
          aria-label={t('manuals.search.button')}
          className="px-5 py-2.5 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold text-xs flex items-center justify-center gap-2 transition-colors shrink-0 focus:outline-none focus:ring-2 focus:ring-cyan-400"
        >
          <Search className="w-4 h-4" />
          <span>{t('manuals.search.button')}</span>
        </button>
      </div>

      {/* Diagnostic Terms Quick Pills */}
      <div className="flex flex-wrap items-center gap-1.5 pt-1">
        <span className="text-[11px] text-slate-500 mr-1 flex items-center gap-1">
          <Sparkles className="w-3 h-3 text-amber-400" /> {t('manuals.terms.label')}
        </span>
        {activeTerms.length === 0 ? (
          <span className="text-xs text-slate-600">Nenhum termo ativo</span>
        ) : (
          activeTerms.map(term => (
            <button
              key={term}
              onClick={() => selectTerm(term)}
              aria-label={`Buscar termo ${term}`}
              className={`px-2.5 py-1 rounded-lg text-xs transition-colors border focus:outline-none focus:ring-1 focus:ring-cyan-500/50 ${
                searchQuery.toUpperCase() === term.toUpperCase()
                  ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/40 font-bold'
                  : 'bg-slate-950 text-slate-400 border-slate-800 hover:border-slate-700 hover:text-slate-200'
              }`}
            >
              {term}
            </button>
          ))
        )}
      </div>
    </div>
  );
};
