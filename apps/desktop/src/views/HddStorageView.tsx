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
    imagePath,
    error,
    extractPartitions,
    refresh,
  } = useHddStorageViewModel();

  const [copiedSha, setCopiedSha] = useState(false);
  const [inputPath, setInputPath] = useState(imagePath || '');
  const [showPathInput, setShowPathInput] = useState(false);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedSha(true);
    setTimeout(() => setCopiedSha(false), 2000);
  };

  const handleOpenPath = () => {
    if (inputPath.trim()) {
      refresh(inputPath.trim());
      setShowPathInput(false);
    }
  };

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Top Header & Fast Partition Extractor */}
      <HddTopHeader
        report={hddReport}
        extracting={extracting}
        onExtractPartitions={extractPartitions}
        onSelectImage={() => setShowPathInput(prev => !prev)}
      />

      {/* Path Input Dropdown when user clicks Abrir Imagem */}
      {showPathInput && (
        <div className="p-4 rounded-xl bg-slate-950 border border-cyan-500/40 shadow-xl space-y-2">
          <label className="text-[11px] text-cyan-300 font-bold uppercase">
            Caminho do Arquivo de Imagem (.bin / .raw / .img)
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              placeholder="Ex: firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"
              value={inputPath}
              onChange={e => setInputPath(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleOpenPath()}
              className="flex-1 px-3 py-2 rounded-lg bg-bench-900 border border-slate-700 text-xs text-slate-100 focus:outline-none focus:border-cyan-400"
            />
            <button
              onClick={handleOpenPath}
              className="px-4 py-2 rounded-lg bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold text-xs transition-colors"
            >
              Carregar
            </button>
          </div>
        </div>
      )}

      {/* Metrics Row */}
      <HddMetricsGrid report={hddReport} />

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

      {/* Empty State / Image Selector when no report is loaded */}
      {!hddReport && activeTab !== 'writer' && (
        <div className="p-8 rounded-2xl bg-bench-900 border border-slate-800 text-center space-y-4 max-w-2xl mx-auto shadow-xl">
          <div className="w-12 h-12 rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 flex items-center justify-center mx-auto">
            <HardDrive className="w-6 h-6" />
          </div>
          <div>
            <h3 className="text-sm font-bold text-slate-100">
              {error ? 'Imagem de Disco Padrão Não Encontrada' : 'Nenhuma Imagem MEIHDFS Carregada'}
            </h3>
            <p className="text-xs text-slate-400 mt-1 max-w-md mx-auto">
              {error || 'Informe o caminho de um dump físico (.bin, .raw, .img) ou alterne para a aba Gravação Física para gerenciar discos conectados via SATA/USB.'}
            </p>
          </div>

          <div className="flex items-center gap-2 max-w-md mx-auto pt-2">
            <input
              type="text"
              placeholder="Ex: C:\dumps\panasonic_hdd.bin"
              value={inputPath}
              onChange={e => setInputPath(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleOpenPath()}
              className="flex-1 px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-200 focus:outline-none focus:border-cyan-500 font-mono"
            />
            <button
              onClick={handleOpenPath}
              disabled={loading || !inputPath.trim()}
              className="px-4 py-2 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold text-xs transition-colors disabled:opacity-50"
            >
              {loading ? 'Carregando...' : 'Analisar'}
            </button>
          </div>

          <div className="pt-4 border-t border-slate-800 flex items-center justify-center gap-4 text-xs">
            <button
              onClick={() => setActiveTab('writer')}
              className="px-4 py-2 rounded-xl bg-purple-500/10 hover:bg-purple-500/20 text-purple-300 border border-purple-500/30 font-bold flex items-center gap-2 transition-colors"
            >
              <HardDrive className="w-4 h-4" />
              <span>Ver Discos Físicos Conectados (Win32/SATA)</span>
            </button>
          </div>
        </div>
      )}

      {/* Tab 1: Overview */}
      {activeTab === 'overview' && hddReport && (
        <HddPartitionTable hddReport={hddReport} />
      )}

      {/* Tab 2: MBR vs Mirror LBA 57344 */}
      {activeTab === 'mbr' && hddReport && (
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
      {activeTab === 'density' && hddReport && (
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
