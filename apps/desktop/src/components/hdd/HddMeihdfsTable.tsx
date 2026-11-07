import React from 'react';
import { Table, Search } from 'lucide-react';
import { MeihdfsAllocationRecord } from '@/models';

export interface HddMeihdfsTableProps {
  filteredRecords: MeihdfsAllocationRecord[];
  searchQuery: string;
  tagFilter: string;
  onSearchChange: (val: string) => void;
  onTagFilterChange: (val: string) => void;
}

export const HddMeihdfsTable: React.FC<HddMeihdfsTableProps> = ({
  filteredRecords,
  searchQuery,
  tagFilter,
  onSearchChange,
  onTagFilterChange,
}) => {
  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <Table className="w-4 h-4 text-cyan-400" />
            <span>256 Registros de Alocação MEIHDFS (Decodificados com XOR 0xFF)</span>
          </h3>
          <p className="text-xs text-slate-400 mt-0.5">
            Offset Físico: 0x0000_3000 (LBA 24) • Comprimento de 32 bytes por nó de alocação
          </p>
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto">
          <div className="relative flex-1 sm:w-64">
            <Search className="w-3.5 h-3.5 absolute left-2.5 top-2.5 text-slate-500" />
            <input
              type="text"
              placeholder="Filtrar por tag ou hex..."
              value={searchQuery}
              onChange={e => onSearchChange(e.target.value)}
              aria-label="Filtrar registros MEIHDFS por tag ou hex"
              className="pl-8 pr-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-xs text-slate-200 focus:outline-none focus:border-cyan-500 w-full"
            />
          </div>

          <select
            value={tagFilter}
            onChange={e => onTagFilterChange(e.target.value)}
            aria-label="Filtrar por tipo de tag MEIHDFS"
            className="px-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-xs text-slate-300 focus:outline-none focus:border-cyan-500"
          >
            <option value="all">Todas as Tags</option>
            <option value="Extent">Extent Record (0x20)</option>
            <option value="Directory">Directory Descriptor (0x30)</option>
            <option value="Root">Root Volume Anchor (0x01)</option>
            <option value="Journal">WAL Journal Pointer (0x10)</option>
          </select>
        </div>
      </div>

      {/* Table */}
      <div className="overflow-x-auto max-h-[500px] scrollbar-thin scrollbar-thumb-slate-800">
        <table className="w-full text-left text-xs font-mono" aria-label="Tabela de Alocações MEIHDFS">
          <thead className="sticky top-0 bg-bench-950 border-b border-slate-800 text-slate-400 uppercase text-[10px]">
            <tr>
              <th scope="col" className="py-2 px-3">Idx</th>
              <th scope="col" className="py-2 px-3">Tag Decodificada</th>
              <th scope="col" className="py-2 px-3">Hex Bruto (0xFF Invertido)</th>
              <th scope="col" className="py-2 px-3">Parâmetros (Setores / LBA)</th>
              <th scope="col" className="py-2 px-3">Descrição Forense</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800/60">
            {filteredRecords.length === 0 ? (
              <tr>
                <td colSpan={5} className="py-8 text-center text-xs text-slate-500 font-mono">
                  Nenhum registro MEIHDFS encontrado para os filtros selecionados
                </td>
              </tr>
            ) : (
              filteredRecords.map(rec => (
                <tr key={rec.index} className="hover:bg-slate-800/40 transition-colors">
                  <td className="py-2 px-3 font-bold text-slate-400">
                    #{rec.index}
                  </td>
                  <td className="py-2 px-3">
                    <span className="font-bold text-cyan-400">{rec.tag_name}</span>
                    <span className="text-[10px] text-slate-500 block">
                      Raw: 0x{rec.raw_tag.toString(16).toUpperCase()} → Dec: 0x{rec.decoded_tag.toString(16).toUpperCase()}
                    </span>
                  </td>
                  <td className="py-2 px-3 text-slate-300 text-[11px]">
                    <span className="text-slate-500 block line-through">{rec.raw_hex}</span>
                    <span className="text-emerald-400">{rec.inverted_hex}</span>
                  </td>
                  <td className="py-2 px-3 text-slate-300">
                    {rec.sector_count ? (
                      <div>
                        <span>{rec.sector_count} setores</span>
                        <span className="text-slate-500 block text-[10px]">LBA {rec.lba_offset}</span>
                      </div>
                    ) : (
                      <span className="text-slate-600">-</span>
                    )}
                  </td>
                  <td className="py-2 px-3 text-slate-400 text-[11px] max-w-xs truncate">
                    {rec.description}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};
