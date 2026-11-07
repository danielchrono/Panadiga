import React from 'react';
import { BookOpen } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { MeihdfsAllocationReport } from '@/models';

export interface HddJournalTableProps {
  meihdfsReport: MeihdfsAllocationReport;
}

export const HddJournalTable: React.FC<HddJournalTableProps> = ({ meihdfsReport }) => {
  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <BookOpen className="w-4 h-4 text-cyan-400" />
            <span>Anel de Transações do Journal WAL (10 Blocos de 128 KB)</span>
          </h3>
          <p className="text-xs text-slate-400 mt-0.5">
            Faixa: 0x0082_0000 .. 0x0096_0000 (1.310.720 bytes) • Resiliência contra quedas de energia AC
          </p>
        </div>
        <StatusBadge variant="info" label="10 BLOCOS WAL" />
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3">
        {meihdfsReport.journal_blocks.length === 0 ? (
          <div className="col-span-full py-8 text-center text-xs text-slate-500 font-mono">
            Nenhum bloco de journal WAL identificado
          </div>
        ) : (
          meihdfsReport.journal_blocks.map(blk => (
            <div
              key={blk.block_index}
              className="p-3 rounded-xl bg-slate-950 border border-slate-800 space-y-2 text-xs font-mono"
            >
              <div className="flex items-center justify-between border-b border-slate-800/80 pb-1.5">
                <span className="font-bold text-cyan-400">Bloco #{blk.block_index}</span>
                <span className="text-[10px] text-slate-500">{blk.start_offset_hex}</span>
              </div>
              <div className="space-y-1 text-[11px] text-slate-400">
                <div className="flex justify-between">
                  <span>Seq Tx:</span>
                  <span className="text-slate-200">#{blk.transaction_seq}</span>
                </div>
                <div className="flex justify-between">
                  <span>Setores:</span>
                  <span className="text-slate-200">{blk.active_payload_sectors}</span>
                </div>
                <div className="flex justify-between">
                  <span>CRC:</span>
                  <span className="text-emerald-400">{blk.checksum_crc}</span>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
