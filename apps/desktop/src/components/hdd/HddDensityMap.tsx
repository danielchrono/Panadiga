import React from 'react';
import { BarChart3 } from 'lucide-react';
import { HddAnalysisReport } from '@/models';

export interface HddDensityMapProps {
  hddReport: HddAnalysisReport;
}

export const HddDensityMap: React.FC<HddDensityMapProps> = ({ hddReport }) => {
  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <BarChart3 className="w-4 h-4 text-cyan-400" />
            <span>Mapa de Densidade de Dados Não-Zero (1 Bloco = 1 Megabyte)</span>
          </h3>
          <p className="text-xs text-slate-400 mt-0.5">
            Identificação visual imediata das áreas de firmware, partições ativas e regiões vazias.
          </p>
        </div>
      </div>

      <div
        className="grid grid-cols-10 sm:grid-cols-20 gap-1.5 p-3 rounded-xl bg-slate-950 border border-slate-800"
        role="region"
        aria-label="Histograma visual de densidade dos 100 MB do disco rígido"
      >
        {hddReport.density_map.length === 0 ? (
          <div className="col-span-full py-8 text-center text-xs text-slate-500 font-mono">
            Nenhum dado de densidade disponível
          </div>
        ) : (
          hddReport.density_map.map(chunk => {
            let bg = 'bg-slate-900 text-slate-600';
            if (chunk.percentage > 90) bg = 'bg-cyan-500 text-slate-950 font-bold';
            else if (chunk.percentage > 50) bg = 'bg-cyan-700 text-slate-100';
            else if (chunk.percentage > 0) bg = 'bg-cyan-950 text-cyan-400 border border-cyan-800';

            return (
              <div
                key={chunk.mb_index}
                title={`MB ${chunk.mb_index} (0x${chunk.offset.toString(16).toUpperCase()}): ${chunk.percentage}% preenchido`}
                className={`h-9 rounded flex flex-col items-center justify-center text-[9px] cursor-pointer hover:ring-2 hover:ring-cyan-300 transition-all ${bg}`}
              >
                <span>{chunk.mb_index}M</span>
                <span className="text-[8px] opacity-80">{chunk.percentage > 0 ? `${chunk.percentage}%` : '0%'}</span>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
