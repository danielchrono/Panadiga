import React from 'react';
import { Layers } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { FlashSectorInfo } from '@/models';
import { cn } from '@/lib/utils';

interface NvramSectorMapProps {
  sectorMap: FlashSectorInfo[];
  selectedSector: FlashSectorInfo | null;
  filterType: string;
  cleanSuccess: boolean;
  onSelectSector: (sector: FlashSectorInfo) => void;
  onSelectFilter: (filter: string) => void;
}

export const NvramSectorMap: React.FC<NvramSectorMapProps> = ({
  sectorMap,
  selectedSector,
  filterType,
  cleanSuccess,
  onSelectSector,
  onSelectFilter,
}) => (
  <div className="rounded-2xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4 font-mono">
    {/* Map Header & Filter Tabs */}
    <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 border-b border-slate-800 pb-3">
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
          <Layers className="w-4 h-4 text-cyan-400" />
          <span>Topologia Física dos 71 Setores (Spansion S29JL064H)</span>
        </h3>
        <p className="text-[11px] text-slate-500 font-sans mt-0.5">
          Clique em qualquer setor para inspecionar offset, tamanho e status de integridade.
        </p>
      </div>

      <div className="flex items-center gap-1.5 p-1 rounded-xl bg-slate-950 border border-slate-800 text-xs">
        <button
          onClick={() => onSelectFilter('all')}
          className={cn(
            'px-2.5 py-1 rounded-lg transition-colors',
            filterType === 'all' ? 'bg-cyan-500/20 text-cyan-300 font-bold' : 'text-slate-400 hover:text-slate-200'
          )}
        >
          Todos (71)
        </button>
        <button
          onClick={() => onSelectFilter('nvram')}
          className={cn(
            'px-2.5 py-1 rounded-lg transition-colors',
            filterType === 'nvram' ? 'bg-amber-500/20 text-amber-300 font-bold' : 'text-slate-400 hover:text-slate-200'
          )}
        >
          NVRAM (SA134..SA141)
        </button>
        <button
          onClick={() => onSelectFilter('bootloader')}
          className={cn(
            'px-2.5 py-1 rounded-lg transition-colors',
            filterType === 'bootloader' ? 'bg-cyan-500/20 text-cyan-300 font-bold' : 'text-slate-400 hover:text-slate-200'
          )}
        >
          Bootloader (SA0..SA15)
        </button>
      </div>
    </div>

    {/* High-Contrast Interactive Grid */}
    <div className="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
      <div className="flex items-center justify-between text-[11px] text-slate-500 mb-2 px-1">
        <span>0x0000_0000 (SA0 Bootloader)</span>
        <span className="text-amber-400 font-bold">0x007F_0000 (SA134..SA141 NVRAM)</span>
        <span>0x0080_0000 (Fim da Flash)</span>
      </div>

      <div className="grid grid-cols-12 sm:grid-cols-24 md:grid-cols-36 lg:grid-cols-71 gap-1">
        {sectorMap.length === 0 ? (
          <div className="col-span-full py-8 text-center text-xs text-slate-500 font-mono">
            Nenhum setor da Flash NOR mapeado
          </div>
        ) : (
          sectorMap.map(sector => {
            const isSelected = selectedSector?.name === sector.name;
            let bg = 'bg-slate-800 hover:bg-slate-700 text-slate-300';

            if (sector.is_corrupted_area) {
              bg = cleanSuccess
                ? 'bg-emerald-500 hover:bg-emerald-400 text-slate-950 font-bold'
                : 'bg-rose-500 hover:bg-rose-400 text-slate-950 font-bold animate-pulse';
            } else if (sector.sector_type === 'bootloader') {
              bg = 'bg-cyan-900/60 hover:bg-cyan-800/80 text-cyan-200';
            }

            return (
              <button
                key={sector.name}
                onClick={() => onSelectSector(sector)}
                title={`${sector.name} (0x${sector.start_offset.toString(16).toUpperCase()}) - ${sector.size_bytes / 1024} KB`}
                className={cn(
                  'h-7 rounded text-[9px] font-bold flex items-center justify-center transition-all',
                  bg,
                  isSelected && 'ring-2 ring-cyan-400 scale-110 z-10'
                )}
              >
                {sector.index}
              </button>
            );
          })
        )}
      </div>

      {/* Grid Legend */}
      <div className="flex flex-wrap items-center gap-4 mt-3 pt-3 border-t border-slate-800 text-[11px] text-slate-400">
        <div className="flex items-center gap-1.5">
          <span className="w-3 h-3 rounded bg-cyan-900/60 border border-cyan-700/50" />
          <span>Bootloader Imutável (SA0..SA15)</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="w-3 h-3 rounded bg-slate-800" />
          <span>OS Firmware Normal (SA16..SA133 - 64 KB)</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className={cn('w-3 h-3 rounded', cleanSuccess ? 'bg-emerald-500' : 'bg-rose-500 animate-pulse')} />
          <span>
            {cleanSuccess
              ? 'Top Bank Limpo / Virgem (SA134..SA141)'
              : 'Área Corrompida U99 (SA134..SA141 - 8 KB cada)'}
          </span>
        </div>
      </div>
    </div>

    {/* Selected Sector Inspector Panel */}
    {selectedSector && (
      <div className="p-4 rounded-xl bg-bench-950 border border-slate-800 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <span className="text-sm font-bold text-slate-100">
              Setor Selecionado: {selectedSector.name}
            </span>
            <StatusBadge
              variant={selectedSector.is_corrupted_area ? (cleanSuccess ? 'success' : 'error') : 'info'}
              label={selectedSector.is_corrupted_area ? (cleanSuccess ? 'LIMPO 0xFF' : 'CORRUPÇÃO U99') : 'INTACTO'}
            />
          </div>
          <p className="text-xs text-slate-400 font-sans">{selectedSector.notes}</p>
        </div>

        <div className="flex flex-wrap items-center gap-4 text-xs font-mono">
          <div className="text-slate-400">
            <span className="text-slate-500 block text-[10px]">OFFSET FÍSICO</span>
            <span>0x{selectedSector.start_offset.toString(16).padStart(8, '0').toUpperCase()}</span>
          </div>
          <div className="text-slate-400">
            <span className="text-slate-500 block text-[10px]">TAMANHO</span>
            <span>{selectedSector.size_bytes / 1024} KB</span>
          </div>
          <div className="text-slate-400">
            <span className="text-slate-500 block text-[10px]">BYTES ATIVOS</span>
            <span className={selectedSector.is_corrupted_area && !cleanSuccess ? 'text-rose-400 font-bold' : 'text-slate-200'}>
              {selectedSector.active_bytes ?? 0} B
            </span>
          </div>
        </div>
      </div>
    )}
  </div>
);
