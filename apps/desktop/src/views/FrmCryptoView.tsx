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
import { BarChart2, Layers, KeyRound, FileCheck2, RefreshCw, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';

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
    error,
    codebookFilter,
    setCodebookFilter,
    refresh,
  } = useFrmCryptoViewModel();

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-96 gap-3 text-slate-500 font-mono text-xs">
        <RefreshCw className="w-6 h-6 animate-spin text-purple-400" />
        <span>{t('crypto.loading')}</span>
      </div>
    );
  }

  if (!cryptReport) {
    return (
      <div className="max-w-2xl mx-auto my-16 p-6 rounded-2xl bg-bench-900 border border-slate-800 text-center space-y-4 font-mono">
        <div className="w-12 h-12 rounded-full bg-rose-500/10 text-rose-400 border border-rose-500/30 flex items-center justify-center mx-auto">
          <AlertCircle className="w-6 h-6" />
        </div>
        <div>
          <h3 className="text-sm font-bold text-slate-100">
            Contêiner Criptográfico PANA_DVD.FRM Não Encontrado
          </h3>
          <p className="text-xs text-slate-400 mt-1">
            {error ||
              'Não foi possível carregar ou analisar o contêiner PANA_DVD.FRM. Verifique se o arquivo está na pasta firmware_and_images/.'}
          </p>
        </div>
        <Button variant="emerald" onClick={refresh} className="gap-2 mx-auto">
          <RefreshCw className="w-4 h-4" />
          <span>Tentar Novamente</span>
        </Button>
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

      {activeTab === 'toc' && (
        tocReport ? (
          <FrmTocTab tocReport={tocReport} />
        ) : (
          <div className="p-8 text-center text-slate-500 font-mono text-xs rounded-xl bg-bench-900 border border-slate-800">
            {t('crypto.tabs.toc')} - {t('common.loading', 'Carregando TOC...')}
          </div>
        )
      )}

      {activeTab === 'codebook' && (
        <FrmCodebookTab
          filteredCodebook={filteredCodebook}
          codebookFilter={codebookFilter}
          setCodebookFilter={setCodebookFilter}
        />
      )}

      {activeTab === 'correlation' && (
        <FrmCorrelationTab correlations={cryptReport.hdd_correlations || []} />
      )}
    </div>
  );
};
