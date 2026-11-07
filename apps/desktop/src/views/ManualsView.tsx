import React, { useState } from 'react';
import { useTranslation } from '@/i18n/useTranslation';
import { useManualsViewModel } from '@/viewmodels/useManualsViewModel';
import { StatusBadge } from '@/components/StatusBadge';
import { digaApi } from '@/services/digaApi';
import { RagQueryResult, ExecutionStatus } from '@/types/domain';
import {
  BookOpen,
  Search,
  FileText,
  Layers,
  Sparkles,
  ShieldCheck,
  ShieldAlert,
  Bot,
  CheckCircle2,
  AlertTriangle,
  ExternalLink,
  Cpu,
} from 'lucide-react';

export const ManualsView: React.FC = () => {
  const { t } = useTranslation();
  const {
    documents,
    searchResults,
    searchQuery,
    setSearchQuery,
    activeTerms,
    selectedDoc,
    setSelectedDoc,
    loading,
    executeSearch,
    selectTerm,
  } = useManualsViewModel();

  // Zero-Error RAG state
  const [ragQuery, setRagQuery] = useState<string>('How to recover error U99 on Spansion Flash?');
  const [ragStatus, setRagStatus] = useState<ExecutionStatus>(ExecutionStatus.IDLE);
  const [ragResult, setRagResult] = useState<RagQueryResult | null>(null);
  const [ragError, setRagError] = useState<string | null>(null);

  const handleRagSubmit = async () => {
    if (!ragQuery.trim()) return;
    setRagStatus(ExecutionStatus.RUNNING);
    setRagError(null);

    const res = await digaApi.queryRag(ragQuery);
    if (res.success) {
      setRagResult(res.data);
      setRagStatus(ExecutionStatus.SUCCESS);
    } else {
      setRagError(res.error);
      setRagStatus(ExecutionStatus.ERROR);
    }
  };

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
                {t('manuals.header.title')}
              </h2>
              <StatusBadge variant="info" label={t('manuals.header.badge')} />
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              {t('manuals.header.subtitle')}
            </p>
          </div>
        </div>

        <div className="text-right text-xs text-slate-400">
          <span>{t('manuals.header.size')}</span>
        </div>
      </div>

      {/* ZERO-ERROR RAG AI ASSISTANT PANEL */}
      <div className="rounded-2xl bg-slate-950 border border-cyan-500/30 p-6 shadow-2xl space-y-5">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 border-b border-slate-800 pb-4">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30">
              <Bot className="w-5 h-5" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h3 className="text-sm font-bold text-slate-100">
                  {t('rag.title')}
                </h3>
                <StatusBadge variant="success" label={t('rag.badge')} />
              </div>
              <p className="text-xs text-slate-400">
                {t('rag.subtitle')}
              </p>
            </div>
          </div>
          <div className="text-xs text-cyan-400 font-mono flex items-center gap-1.5">
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
            <span>{t('rag.groundedBadge')}</span>
          </div>
        </div>

        {/* Input Bar */}
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
          <div className="relative flex-1">
            <input
              type="text"
              placeholder={t('rag.inputPlaceholder')}
              value={ragQuery}
              onChange={e => setRagQuery(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleRagSubmit()}
              className="w-full pl-4 pr-4 py-2.5 rounded-xl bg-bench-900 border border-slate-800 text-xs text-slate-100 placeholder-slate-500 focus:outline-none focus:border-cyan-500 font-mono"
            />
          </div>

          <button
            onClick={handleRagSubmit}
            disabled={ragStatus === ExecutionStatus.RUNNING}
            className="px-5 py-2.5 rounded-xl bg-cyan-500 hover:bg-cyan-400 disabled:opacity-50 text-slate-950 font-bold text-xs flex items-center justify-center gap-2 transition-colors shrink-0"
          >
            <Sparkles className="w-4 h-4" />
            <span>{ragStatus === ExecutionStatus.RUNNING ? t('rag.asking') : t('rag.askButton')}</span>
          </button>
        </div>

        {/* Error Alert Display */}
        {ragError && (
          <div className="p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs flex items-center gap-2">
            <ShieldAlert className="w-4 h-4 text-rose-400 shrink-0" />
            <span>{ragError}</span>
          </div>
        )}

        {/* RAG Answer Display */}
        {ragResult && (
          <div className="space-y-4 pt-2">
            {/* Safety & Invariant Status Bar */}
            <div className="flex flex-wrap items-center justify-between gap-3 p-3 rounded-xl bg-bench-900 border border-slate-800 text-xs">
              <div className="flex items-center gap-2">
                {ragResult.safety.is_safe ? (
                  <span className="flex items-center gap-1.5 text-emerald-400 font-bold">
                    <CheckCircle2 className="w-4 h-4" /> {t('rag.safetyPassed')}
                  </span>
                ) : (
                  <span className="flex items-center gap-1.5 text-rose-400 font-bold">
                    <ShieldAlert className="w-4 h-4" /> {t('rag.safetyBlocked')}
                  </span>
                )}
              </div>
              <div className="flex items-center gap-4 text-slate-400 text-[11px]">
                <span>{t('rag.confidence')} <strong className="text-cyan-300">{(ragResult.confidence * 100).toFixed(0)}%</strong></span>
                <span>{t('rag.latency')} <strong className="text-slate-200">{ragResult.execution_time_ms.toFixed(1)}ms</strong></span>
                <span>{t('rag.chunksCount')} <strong className="text-slate-200">{ragResult.retrieved_chunks_count}</strong></span>
              </div>
            </div>

            {/* Answer Text Card */}
            <div className="p-4 rounded-xl bg-slate-900/80 border border-slate-800 text-xs text-slate-200 leading-relaxed space-y-3 whitespace-pre-line font-sans">
              {ragResult.answer}
            </div>

            {/* FactCritic Audit Report */}
            {ragResult.critic && (
              <div className="p-3.5 rounded-xl bg-bench-900 border border-slate-800 text-xs space-y-2">
                <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                  <span className="font-bold text-slate-300 flex items-center gap-1.5">
                    <ShieldCheck className="w-4 h-4 text-cyan-400" />
                    {t('rag.criticReportTitle')}
                  </span>
                  <StatusBadge
                    variant={ragResult.critic.is_faithful ? 'success' : 'warning'}
                    label={ragResult.critic.is_faithful ? t('rag.groundedBadge') : t('rag.hallucinationWarning')}
                  />
                </div>
                {ragResult.critic.ungrounded_entities.length > 0 && (
                  <div className="text-rose-400 text-[11px] space-y-1">
                    <span className="font-bold">{t('rag.ungroundedEntities')}</span>
                    {ragResult.critic.ungrounded_entities.map((ue, idx) => (
                      <div key={idx} className="font-mono pl-2">• {ue}</div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Citations List */}
            {ragResult.citations.length > 0 && (
              <div className="space-y-2 pt-1">
                <h4 className="text-xs font-bold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
                  <FileText className="w-4 h-4 text-amber-400" />
                  {t('rag.citationsTitle')}
                </h4>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                  {ragResult.citations.map((c, idx) => (
                    <div
                      key={idx}
                      className="p-3 rounded-xl bg-bench-900 border border-slate-800/80 text-xs space-y-1"
                    >
                      <div className="flex items-center justify-between text-[11px] font-bold text-cyan-400">
                        <span className="truncate max-w-[200px]">{c.doc}</span>
                        <span className="text-slate-500 font-mono">{t('rag.line')} {c.lines}</span>
                      </div>
                      <div className="text-[11px] text-slate-400 font-mono truncate">{c.section}</div>
                      <p className="text-[11px] text-slate-300 italic pt-1 border-t border-slate-800/60 font-sans">
                        "{c.excerpt}"
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Quick Keyword Pills & Search Input Bar */}
      <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
          <div className="relative flex-1">
            <Search className="w-4 h-4 absolute left-3 top-3 text-slate-400" />
            <input
              type="text"
              placeholder={t('manuals.search.placeholder')}
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
            <span>{t('manuals.search.button')}</span>
          </button>
        </div>

        {/* Diagnostic Terms Quick Pills */}
        <div className="flex flex-wrap items-center gap-1.5 pt-1">
          <span className="text-[11px] text-slate-500 mr-1 flex items-center gap-1">
            <Sparkles className="w-3 h-3 text-amber-400" /> {t('manuals.terms.label')}
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
                  {Object.entries(result.matches).map(([term, pages]) => (
                    <span
                      key={term}
                      className="px-2 py-0.5 rounded bg-slate-950 border border-slate-800 text-[11px] text-slate-300 font-mono"
                    >
                      <span className="text-amber-400 font-bold">{term}</span>: {t('manuals.results.pages')} {pages.slice(0, 5).join(', ')}
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
                        <span className="text-cyan-400 font-bold">{t('rag.line')} #{snip.page}</span>
                        <span className="text-amber-400 font-mono">{snip.term}</span>
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
              <span>{t('manuals.meta.title')}</span>
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
                        {doc.page_count} {t('manuals.results.pages')}
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
