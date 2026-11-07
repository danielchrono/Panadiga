import React from 'react';
import { useHexInspectorViewModel } from '@/viewmodels/useHexInspectorViewModel';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { useTranslation } from '@/i18n/useTranslation';
import {
  Binary,
  Bookmark,
  ChevronLeft,
  ChevronRight,
  Search,
  ArrowRight,
  Compass,
} from 'lucide-react';

export const HexInspectorView: React.FC = () => {
  const { t } = useTranslation();
  const {
    currentOffset,
    currentLba,
    hexRows,
    bookmarks,
    selectedBookmark,
    jumpInput,
    setJumpInput,
    loading,
    feedbackMessage,
    selectBookmark,
    jumpToInputOffset,
    nextPage,
    prevPage,
  } = useHexInspectorViewModel();

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Top Header */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
        <div className="flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30">
            <Binary className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-base font-bold text-slate-100">
                Inspetor Hexadecimal de Firmware & LBA Forensics
              </h2>
              <StatusBadge variant="info" label="16 BYTES/LINHA" />
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Endereçamento dual (Offset Físico Hexadecimal & Setor LBA de 512 Bytes) com Decodificação ASCII
            </p>
          </div>
        </div>

        {/* Jump & Navigation Controls */}
        <div className="flex items-center gap-2">
          <div className="relative">
            <input
              type="text"
              placeholder="0x00000000..."
              value={jumpInput}
              onChange={e => setJumpInput(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && jumpToInputOffset()}
              aria-label="Offset físico em hexadecimal ou número de setor LBA"
              className="px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-xs text-cyan-400 focus:outline-none focus:border-cyan-500 w-36 font-bold focus:ring-1 focus:ring-cyan-500"
            />
          </div>
          <button
            onClick={jumpToInputOffset}
            className="px-3 py-2 rounded-xl bg-slate-800 hover:bg-slate-750 text-slate-200 text-xs font-semibold border border-slate-700 transition-colors focus:outline-none focus:ring-2 focus:ring-slate-400"
          >
            {t('hex.jumpButton', 'Ir para')}
          </button>
        </div>
      </div>

      {feedbackMessage && (
        <div className="p-3 rounded-xl bg-emerald-950/80 border border-emerald-500/40 text-emerald-300 text-xs font-mono flex items-center justify-between animate-in fade-in duration-200">
          <span>{feedbackMessage}</span>
        </div>
      )}

      {/* Telemetry Metrics */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Offset Físico Atual"
          value={`0x${currentOffset.toString(16).padStart(8, '0').toUpperCase()}`}
          subtitle={`${currentOffset.toLocaleString()} bytes a partir do início`}
          variant="cyan"
        />
        <MetricCard
          title="Setor LBA ATA/IDE"
          value={`LBA ${currentLba.toLocaleString()}`}
          subtitle="Unidade de 512 bytes de bloco"
          variant="emerald"
        />
        <MetricCard
          title="Marcador Selecionado"
          value={selectedBookmark ? selectedBookmark.category : 'LIVRE'}
          subtitle={selectedBookmark ? selectedBookmark.label : 'Endereçamento manual'}
          variant="amber"
        />
        <MetricCard
          title="Janela de Exibição"
          value={`${hexRows.length * 16}`}
          unit="bytes"
          subtitle={`${hexRows.length} linhas de 16 bytes`}
          variant="slate"
        />
      </div>

      {/* Main Hex Workspace: Left Bookmarks / Right Hex Table */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Bookmarks List */}
        <div className="lg:col-span-4 space-y-3">
          <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-3">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
              <Compass className="w-4 h-4 text-cyan-400" />
              <span>Marcadores Arquiteturais Críticos</span>
            </h3>

            <div className="space-y-2">
              {bookmarks.length === 0 ? (
                <div className="p-6 text-center text-xs text-slate-500 font-mono">
                  Nenhum marcador registrado
                </div>
              ) : (
                bookmarks.map(bm => {
                  const isSelected = selectedBookmark?.id === bm.id;
                  return (
                    <div
                      key={bm.id}
                      onClick={() => selectBookmark(bm)}
                      className={`p-3 rounded-xl border cursor-pointer transition-all ${
                        isSelected
                          ? 'bg-cyan-500/10 border-cyan-500/50 shadow-md shadow-cyan-950/20'
                          : 'bg-slate-950 border-slate-800 hover:border-slate-700'
                      }`}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className={`text-xs font-bold ${isSelected ? 'text-cyan-400' : 'text-slate-200'}`}>
                          {bm.label}
                        </span>
                        <span className="text-[10px] text-slate-500 font-mono">
                          {bm.offset_hex}
                        </span>
                      </div>
                      <p className="text-[11px] text-slate-400 font-sans">
                        {bm.description}
                      </p>
                    </div>
                  );
                })
              )}
            </div>
          </div>
        </div>

        {/* Right: The High-Precision Hex Dump Grid */}
        <div className="lg:col-span-8 space-y-3">
          <div className="rounded-xl bg-bench-900 border border-slate-800 shadow-xl overflow-hidden flex flex-col">
            {/* Hex Header Bar */}
            <div className="px-4 py-2.5 bg-bench-950 border-b border-slate-800 flex items-center justify-between text-xs text-slate-400">
              <div className="flex items-center gap-6">
                <span className="w-24 font-bold text-slate-500">OFFSET</span>
                <span className="w-52 font-bold tracking-wider text-slate-400">
                  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
                </span>
                <span className="font-bold text-slate-500">ASCII DUMP</span>
              </div>

              {/* Pagination controls */}
              <div className="flex items-center gap-1">
                <button
                  onClick={prevPage}
                  className="p-1 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-slate-200 transition-colors focus:outline-none focus:ring-2 focus:ring-slate-400"
                  title="Página Anterior (384 bytes)"
                  aria-label="Página Anterior (384 bytes)"
                >
                  <ChevronLeft className="w-4 h-4" />
                </button>
                <button
                  onClick={nextPage}
                  className="p-1 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-slate-200 transition-colors focus:outline-none focus:ring-2 focus:ring-slate-400"
                  title="Próxima Página (384 bytes)"
                  aria-label="Próxima Página (384 bytes)"
                >
                  <ChevronRight className="w-4 h-4" />
                </button>
              </div>
            </div>

            {/* Rows */}
            <div className="p-4 bg-slate-950 overflow-x-auto space-y-1 text-xs select-text">
              {loading ? (
                <div className="py-12 text-center text-slate-500">
                  Carregando bytes na memória...
                </div>
              ) : (
                hexRows.map(row => (
                  <div
                    key={row.offset}
                    className="flex items-center gap-6 hover:bg-slate-900/60 px-2 py-0.5 rounded transition-colors group"
                  >
                    {/* Offset */}
                    <span className="w-24 text-cyan-400 font-bold shrink-0">
                      {row.offset_hex}
                    </span>

                    {/* 16 Hex bytes split 8 / 8 */}
                    <div className="w-52 flex items-center gap-3 shrink-0">
                      <span className="text-slate-200 tracking-wider">
                        {row.hex_part1}
                      </span>
                      <span className="text-slate-300 tracking-wider">
                        {row.hex_part2}
                      </span>
                    </div>

                    {/* ASCII */}
                    <span className="text-emerald-400 tracking-wider shrink-0 font-bold border-l border-slate-800 pl-4 group-hover:text-emerald-300">
                      |{row.ascii}|
                    </span>
                  </div>
                ))
              )}
            </div>

            {/* Hex Footer */}
            <div className="px-4 py-2 bg-bench-950 border-t border-slate-800 flex items-center justify-between text-[11px] text-slate-500">
              <span>Modo Zero-Copy Mmap • Alinhamento ATA/IDE 512B</span>
              <div className="flex items-center gap-3">
                <button
                  onClick={prevPage}
                  className="text-slate-400 hover:text-slate-200 underline"
                >
                  Pág Anterior (-384B)
                </button>
                <span>•</span>
                <button
                  onClick={nextPage}
                  className="text-slate-400 hover:text-slate-200 underline"
                >
                  Próxima Pág (+384B)
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
