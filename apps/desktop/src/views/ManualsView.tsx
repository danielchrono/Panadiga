import React from 'react';
import { useManualsViewModel } from '@/viewmodels/useManualsViewModel';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import {
  BookOpen,
  Search,
  FileText,
  Bookmark,
  ExternalLink,
  Cpu,
  Layers,
  Sparkles,
} from 'lucide-react';

export const ManualsView: React.FC = () => {
  const {
    documents,
    searchResults,
    searchQuery,
    setSearchQuery,
    activeTerms,
    selectedDoc,
    setSelectedDoc,
    loading,
    categoryFilter,
    setCategoryFilter,
    executeSearch,
    selectTerm,
  } = useManualsViewModel();

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Top Header */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
        <div className="flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-blue-500/10 text-blue-400 border border-blue-500/30">
            <BookOpen className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-base font-bold text-slate-100">
                Biblioteca Técnica & Esquemas Elétricos (15 Manuais em PDF)
              </h2>
              <StatusBadge variant="info" label="INDEXAÇÃO LOCAL RÁPIDA" />
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Busca instantânea de CIs, conectores de teste, rotinas de auto-teste e códigos de erro de bancada
            </p>
          </div>
        </div>

        <div className="text-right text-xs text-slate-400">
          <span>62.1 MB de Documentação Indexada</span>
        </div>
      </div>

      {/* Quick Keyword Pills & Search Input Bar */}
      <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
          <div className="relative flex-1">
            <Search className="w-4 h-4 absolute left-3 top-3 text-slate-400" />
            <input
              type="text"
              placeholder="Pesquisar por código (U99, S29JL064, CN55001, IC5001)..."
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && executeSearch(searchQuery)}
              className="w-full pl-9 pr-4 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-100 placeholder-slate-500 focus:outline-none focus:border-cyan-500 font-mono"
            />
          </div>

          <button
            onClick={() => executeSearch(searchQuery)}
            className="px-5 py-2.5 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold text-xs flex items-center justify-center gap-2 transition-colors shrink-0"
          >
            <Search className="w-4 h-4" />
            <span>Pesquisar Manuais</span>
          </button>
        </div>

        {/* Diagnostic Terms Quick Pills */}
        <div className="flex flex-wrap items-center gap-1.5 pt-1">
          <span className="text-[11px] text-slate-500 mr-1 flex items-center gap-1">
            <Sparkles className="w-3 h-3 text-amber-400" /> Termos Críticos:
          </span>
          {activeTerms.map(term => (
            <button
              key={term}
              onClick={() => selectTerm(term)}
              className={`px-2.5 py-1 rounded-lg text-xs transition-colors border ${
                searchQuery.toUpperCase() === term.toUpperCase()
                  ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/40 font-bold'
                  : 'bg-slate-950 text-slate-400 border-slate-800 hover:border-slate-700 hover:text-slate-200'
              }`}
            >
              {term}
            </button>
          ))}
        </div>
      </div>

      {/* Main Grid: Search Results & Document Metadata */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Search Results & Snippets */}
        <div className="lg:col-span-8 space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
              <FileText className="w-4 h-4 text-cyan-400" />
              <span>Ocorrências Encontradas para "{searchQuery}" ({searchResults.length} manuais)</span>
            </h3>
          </div>

          {loading ? (
            <div className="p-8 text-center text-slate-500 text-xs font-mono">
              Indexando fluxos de texto dos PDFs...
            </div>
          ) : searchResults.length === 0 ? (
            <div className="p-8 rounded-xl bg-bench-900 border border-slate-800 text-center text-slate-500 text-xs">
              Nenhuma referência encontrada para o termo pesquisado. Tente outro código da lista rápida acima.
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
                      {result.file_name} • {result.total_pages} páginas
                    </span>
                  </div>
                  <StatusBadge
                    variant="info"
                    label={`${result.total_match_count} OCORRÊNCIAS`}
                  />
                </div>

                {/* Match badges */}
                <div className="flex flex-wrap gap-1.5">
                  {Object.entries(result.matches).map(([term, pages]) => (
                    <span
                      key={term}
                      className="px-2 py-0.5 rounded bg-slate-950 border border-slate-800 text-[11px] text-slate-300 font-mono"
                    >
                      <span className="text-amber-400 font-bold">{term}</span>: páginas {pages.slice(0, 5).join(', ')}
                      {pages.length > 5 ? '...' : ''}
                    </span>
                  ))}
                </div>

                {/* Snippets */}
                <div className="space-y-2 pt-1">
                  {result.snippets.map((snip, idx) => (
                    <div
                      key={idx}
                      className="p-3 rounded-lg bg-slate-950 border border-slate-800/80 text-xs text-slate-300 space-y-1"
                    >
                      <div className="flex items-center justify-between text-[10px] text-slate-500">
                        <span className="text-cyan-400 font-bold">PÁGINA #{snip.page}</span>
                        <span className="text-amber-400 font-mono">Termo: {snip.term}</span>
                      </div>
                      <p className="text-[11px] text-slate-300 font-sans leading-relaxed">
                        "{snip.context}"
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            ))
          )}
        </div>

        {/* Right: Document Catalog & Selected Doc Details */}
        <div className="lg:col-span-4 space-y-4">
          <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
              <Layers className="w-4 h-4 text-cyan-400" />
              <span>Catálogo de Manuais DMR-EH55</span>
            </h3>

            {/* Document list */}
            <div className="space-y-2 max-h-[500px] overflow-y-auto pr-1 scrollbar-thin scrollbar-thumb-slate-800">
              {documents.map(doc => {
                const isSelected = selectedDoc?.id === doc.id;
                return (
                  <div
                    key={doc.id}
                    onClick={() => setSelectedDoc(doc)}
                    className={`p-3 rounded-xl border cursor-pointer transition-all ${
                      isSelected
                        ? 'bg-blue-500/10 border-blue-500/40 text-blue-200'
                        : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700 hover:text-slate-200'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-xs font-bold truncate max-w-[200px]">
                        {doc.title}
                      </span>
                      <span className="text-[10px] text-slate-500 shrink-0">
                        {doc.page_count}p
                      </span>
                    </div>
                    <p className="text-[10px] text-slate-500 line-clamp-2">
                      {doc.description}
                    </p>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
