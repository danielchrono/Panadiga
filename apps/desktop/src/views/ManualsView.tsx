import React from 'react';
import { useTranslation } from '@/i18n/useTranslation';
import { useManualsViewModel } from '@/viewmodels/useManualsViewModel';
import { StatusBadge } from '@/components/StatusBadge';
import { RagAssistantPanel } from '@/components/rag';
import {
  ManualsSearchCard,
  ManualsResultsList,
  ManualsDocCatalog,
} from '@/components/manuals';
import { BookOpen } from 'lucide-react';

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
      <RagAssistantPanel />

      {/* Quick Keyword Pills & Search Input Bar */}
      <ManualsSearchCard
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
        activeTerms={activeTerms}
        executeSearch={executeSearch}
        selectTerm={selectTerm}
      />

      {/* Main Grid: Search Results & Document Metadata */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Search Results & Snippets */}
        <div className="lg:col-span-8 space-y-4">
          <ManualsResultsList
            searchResults={searchResults}
            loading={loading}
          />
        </div>

        {/* Right: Document Catalog & Selected Doc Details */}
        <div className="lg:col-span-4 space-y-4">
          <ManualsDocCatalog
            documents={documents}
            selectedDoc={selectedDoc}
            setSelectedDoc={setSelectedDoc}
          />
        </div>
      </div>
    </div>
  );
};
