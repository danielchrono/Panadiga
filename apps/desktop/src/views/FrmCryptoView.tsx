import React from 'react';
import { useFrmCryptoViewModel } from '@/viewmodels/useFrmCryptoViewModel';
import { useTranslation } from '@/i18n/useTranslation';
import {
  FrmTopHeader,
  FrmMetricsGrid,
  FrmEntropyTab,
  FrmTocTab,
  FrmCodebookTab,
  FrmCorrelationTab,
} from '@/components/crypto';
import { BarChart2, Layers, KeyRound, FileCheck2 } from 'lucide-react';

export const FrmCryptoView: React.FC = () => {
  const { t } = useTranslation();
  const {
    cryptReport,
    tocReport,
    codebookReport,
    filteredCodebook,
    activeTab,
    setActiveTab,
    loading,
    codebookFilter,
    setCodebookFilter,
  } = useFrmCryptoViewModel();

  if (loading || !cryptReport) {
    return (
      <div className="flex items-center justify-center h-full text-slate-500 font-mono text-xs">
        {t('crypto.loading')}
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      <FrmTopHeader />

      <FrmMetricsGrid cryptReport={cryptReport} />

      {/* Navigation Tabs */}
      <div className="flex items-center gap-2 border-b border-slate-800 pb-2 text-xs overflow-x-auto">
        <button
          onClick={() => setActiveTab('entropy')}
          aria-current={activeTab === 'entropy' ? 'page' : undefined}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-purple-500/50 ${
            activeTab === 'entropy'
              ? 'bg-purple-500/10 text-purple-300 border border-purple-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <BarChart2 className="w-3.5 h-3.5" />
          <span>{t('crypto.tabs.entropy')}</span>
        </button>

        <button
          onClick={() => setActiveTab('toc')}
          aria-current={activeTab === 'toc' ? 'page' : undefined}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-purple-500/50 ${
            activeTab === 'toc'
              ? 'bg-purple-500/10 text-purple-300 border border-purple-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Layers className="w-3.5 h-3.5" />
          <span>{t('crypto.tabs.toc')}</span>
        </button>

        <button
          onClick={() => setActiveTab('codebook')}
          aria-current={activeTab === 'codebook' ? 'page' : undefined}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-purple-500/50 ${
            activeTab === 'codebook'
              ? 'bg-purple-500/10 text-purple-300 border border-purple-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <KeyRound className="w-3.5 h-3.5" />
          <span>{t('crypto.tabs.codebook')}</span>
        </button>

        <button
          onClick={() => setActiveTab('correlation')}
          aria-current={activeTab === 'correlation' ? 'page' : undefined}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-purple-500/50 ${
            activeTab === 'correlation'
              ? 'bg-purple-500/10 text-purple-300 border border-purple-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <FileCheck2 className="w-3.5 h-3.5" />
          <span>{t('crypto.tabs.correlation')}</span>
        </button>
      </div>

      {activeTab === 'entropy' && (
        <FrmEntropyTab topBlocks={cryptReport.top_blocks} />
      )}

      {activeTab === 'toc' && tocReport && (
        <FrmTocTab tocReport={tocReport} />
      )}

      {activeTab === 'codebook' && codebookReport && (
        <FrmCodebookTab
          filteredCodebook={filteredCodebook}
          codebookFilter={codebookFilter}
          setCodebookFilter={setCodebookFilter}
        />
      )}

      {activeTab === 'correlation' && (
        <FrmCorrelationTab correlations={cryptReport.hdd_correlations} />
      )}
    </div>
  );
};
