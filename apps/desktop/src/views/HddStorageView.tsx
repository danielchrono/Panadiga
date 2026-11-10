import React, { useState } from 'react';
import { useHddStorageViewModel, HddTab } from '@/viewmodels/useHddStorageViewModel';
import { useTranslation } from '@/i18n/useTranslation';
import {
  HddTopHeader,
  HddMetricsGrid,
  HddPartitionTable,
  HddMbrComparison,
  HddMeihdfsTable,
  HddJournalTable,
  HddDensityMap,
  HddExtractManifestCard,
  HddWriterCard,
} from '@/components/hdd';
import {
  Layers,
  FileCheck,
  Table,
  BookOpen,
  BarChart3,
  FolderArchive,
  HardDrive,
} from 'lucide-react';

/**
 * Declarative MVVM View: Panasonic MEIHDFS Storage & HDD Forensics Workbench.
 * Decomposed into dedicated domain components under components/hdd/.
 */
export const HddStorageView: React.FC = () => {
  const { t } = useTranslation();
  const {
    hddReport,
    meihdfsReport,
    filteredRecords,
    activeTab,
    setActiveTab,
    loading,
    extracting,
    manifest,
    tagFilter,
    setTagFilter,
    searchQuery,
    setSearchQuery,
    extractPartitions,
  } = useHddStorageViewModel();

  const [copiedSha, setCopiedSha] = useState(false);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedSha(true);
    setTimeout(() => setCopiedSha(false), 2000);
  };

  if (loading || !hddReport) {
    return (
      <div className="flex items-center justify-center h-full text-slate-500 font-mono text-xs">
        {t('hdd.loading', 'Lendo imagem física do HD (104.857.600 bytes) via zero-copy mmap...')}
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Top Header & Fast Partition Extractor */}
      <HddTopHeader
        extracting={extracting}
        onExtractPartitions={extractPartitions}
      />

      {/* Metrics Row */}
      <HddMetricsGrid />

      {/* Navigation Tabs */}
      <div
        className="flex items-center gap-2 border-b border-slate-800 pb-2 text-xs"
        role="tablist"
        aria-label="Navegação de Forense do Disco"
      >
        <button
          onClick={() => setActiveTab('overview')}
          role="tab"
          aria-selected={activeTab === 'overview'}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 ${
            activeTab === 'overview'
              ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Layers className="w-3.5 h-3.5" />
          <span>{t('hdd.tabs.overview', 'Visão Geral do Disco')}</span>
        </button>

        <button
          onClick={() => setActiveTab('mbr')}
          role="tab"
          aria-selected={activeTab === 'mbr'}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 ${
            activeTab === 'mbr'
              ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <FileCheck className="w-3.5 h-3.5" />
          <span>{t('hdd.tabs.mbr', 'MBR vs Espelho LBA 57.344')}</span>
        </button>

        <button
          onClick={() => setActiveTab('meihdfs')}
          role="tab"
          aria-selected={activeTab === 'meihdfs'}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 ${
            activeTab === 'meihdfs'
              ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Table className="w-3.5 h-3.5" />
          <span>{t('hdd.tabs.meihdfs', '256 Registros MEIHDFS (XOR 0xFF)')}</span>
        </button>

        <button
          onClick={() => setActiveTab('journal')}
          role="tab"
          aria-selected={activeTab === 'journal'}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 ${
            activeTab === 'journal'
              ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <BookOpen className="w-3.5 h-3.5" />
          <span>{t('hdd.tabs.journal', 'Journal WAL (10 Blocos)')}</span>
        </button>

        <button
          onClick={() => setActiveTab('density')}
          role="tab"
          aria-selected={activeTab === 'density'}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 ${
            activeTab === 'density'
              ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <BarChart3 className="w-3.5 h-3.5" />
          <span>{t('hdd.tabs.density', 'Mapa de Densidade (100 MB)')}</span>
        </button>

        {manifest && (
          <button
            onClick={() => setActiveTab('extract')}
            role="tab"
            aria-selected={activeTab === 'extract'}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-emerald-400 ${
              activeTab === 'extract'
                ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <FolderArchive className="w-3.5 h-3.5" />
            <span>Manifesto de Partições</span>
          </button>
        )}

        <button
          onClick={() => setActiveTab('writer')}
          role="tab"
          aria-selected={activeTab === 'writer'}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-purple-400 ${
            activeTab === 'writer'
              ? 'bg-purple-500/10 text-purple-400 border border-purple-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <HardDrive className="w-3.5 h-3.5" />
          <span>Gravação Física & mkfs</span>
        </button>
      </div>

      {/* Tab 1: Overview */}
      {activeTab === 'overview' && (
        <HddPartitionTable hddReport={hddReport} />
      )}

      {/* Tab 2: MBR vs Mirror LBA 57344 */}
      {activeTab === 'mbr' && (
        <HddMbrComparison hddReport={hddReport} />
      )}

      {/* Tab 3: MEIHDFS 256 Extent Records */}
      {activeTab === 'meihdfs' && (
        <HddMeihdfsTable
          filteredRecords={filteredRecords}
          searchQuery={searchQuery}
          tagFilter={tagFilter}
          onSearchChange={setSearchQuery}
          onTagFilterChange={setTagFilter}
        />
      )}

      {/* Tab 4: Journal WAL */}
      {activeTab === 'journal' && meihdfsReport && (
        <HddJournalTable meihdfsReport={meihdfsReport} />
      )}

      {/* Tab 5: Chunk Density Map (100 MB) */}
      {activeTab === 'density' && (
        <HddDensityMap hddReport={hddReport} />
      )}

      {/* Tab 6: Partition Extract Manifest */}
      {activeTab === 'extract' && manifest && (
        <HddExtractManifestCard
          manifest={manifest}
          copiedSha={copiedSha}
          onCopySha={copyToClipboard}
        />
      )}

      {/* Tab 7: Physical Drive Writer & mkfs.meihdfs */}
      {activeTab === 'writer' && (
        <HddWriterCard />
      )}
    </div>
  );
};
