import React from 'react';
import { Layers } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { HddAnalysisReport } from '@/models';

export interface HddPartitionTableProps {
  hddReport: HddAnalysisReport;
}

export const HddPartitionTable: React.FC<HddPartitionTableProps> = ({ hddReport }) => {
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {hddReport.partitions.length === 0 ? (
          <div className="col-span-full p-8 text-center text-xs text-slate-500 font-mono rounded-xl bg-bench-900 border border-slate-800">
            Nenhuma partição MEIHDFS identificada no dump de disco rígido
          </div>
        ) : (
          hddReport.partitions.map(part => (
            <div
              key={part.partition_id}
              className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl flex flex-col justify-between"
            >
              <div>
                <div className="flex items-center justify-between mb-3">
                  <span className="text-xs font-bold text-cyan-400">
                    PARTIÇÃO #{part.partition_id}
                  </span>
                  <StatusBadge variant="success" label="MEIHDFS V1.0" />
                </div>
                <h3 className="text-sm font-bold text-slate-100">{part.name}</h3>
                <p className="text-xs text-slate-400 mt-1">
                  Magic: {part.signature_hex} • Início: LBA {part.lba_start} ({part.offset_hex})
                </p>

                <div className="mt-4 p-3 rounded-lg bg-slate-950 border border-slate-800/80 space-y-2 text-xs">
                  <div className="flex justify-between text-slate-400">
                    <span>Tamanho do Payload:</span>
                    <span className="text-slate-200 font-bold">{part.exact_payload_size.toLocaleString()} bytes</span>
                  </div>
                  <div className="flex justify-between text-slate-400">
                    <span>Alinhamento em Setores:</span>
                    <span className="text-slate-200">{part.sectors_count} setores ({part.sector_aligned_size.toLocaleString()} B)</span>
                  </div>
                  <div className="flex justify-between text-slate-400">
                    <span>Entropia de Shannon:</span>
                    <span className="text-amber-400 font-bold">{part.entropy.toFixed(4)} bits/byte</span>
                  </div>
                  <div className="flex justify-between text-slate-400">
                    <span>SHA-256:</span>
                    <span className="text-emerald-400 truncate max-w-xs" title={part.exact_sha256}>
                      {part.exact_sha256}
                    </span>
                  </div>
                </div>
              </div>

              <div className="mt-4 pt-3 border-t border-slate-800 text-[11px] text-slate-500 truncate">
                Header: {part.header_hex_64}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Allocation & Journal Summary */}
      <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-3 flex items-center gap-2">
          <Layers className="w-4 h-4 text-cyan-400" />
          <span>Resumo Arquitetural do MEIHDFS</span>
        </h3>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs font-mono">
          <div className="p-3 rounded-lg bg-slate-950 border border-slate-800">
            <span className="text-slate-500 text-[10px] block">TABELA DE DESCRITORES</span>
            <span className="text-slate-200 font-bold">LBA 24 a 31 (0x3000)</span>
            <span className="text-slate-400 text-[11px] block mt-1">4.096 bytes (256 registros)</span>
          </div>
          <div className="p-3 rounded-lg bg-slate-950 border border-slate-800">
            <span className="text-slate-500 text-[10px] block">CATÁLOGO DO JOURNAL</span>
            <span className="text-slate-200 font-bold">8.00 MB (LBA 16.384)</span>
            <span className="text-slate-400 text-[11px] block mt-1">10 blocos WAL de 128 KB</span>
          </div>
          <div className="p-3 rounded-lg bg-slate-950 border border-slate-800">
            <span className="text-slate-500 text-[10px] block">MARCADOR DE TRANSIÇÃO</span>
            <span className="text-slate-200 font-bold">24.00 MB (LBA 49.152)</span>
            <span className="text-slate-400 text-[11px] block mt-1">Boundary de integridade</span>
          </div>
        </div>
      </div>
    </div>
  );
};
