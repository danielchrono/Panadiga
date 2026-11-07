import React from 'react';
import { Wrench } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { NvramCleanReport, MicroSectorAudit } from '@/models';

interface NvramMicrosectorsTableProps {
  report: NvramCleanReport;
  cleanSuccess: boolean;
}

export const NvramMicrosectorsTable: React.FC<NvramMicrosectorsTableProps> = ({
  report,
  cleanSuccess,
}) => (
  <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl font-mono">
    <div className="flex items-center justify-between mb-4">
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
          <Wrench className="w-4 h-4 text-amber-400" />
          <span>Auditoria Detalhada dos 8 Microsetores (SA134 a SA141 - 8 KB cada)</span>
        </h3>
        <p className="text-[11px] text-slate-500 font-sans mt-0.5">
          Validação estrutural dos parâmetros de inicialização e checksums da EEPROM emulada.
        </p>
      </div>
      <span className="text-xs text-slate-400">65.536 bytes totais</span>
    </div>

    <div className="overflow-x-auto">
      <table className="w-full text-left text-xs font-mono">
        <thead>
          <tr className="border-b border-slate-800 text-slate-500 uppercase text-[10px]">
            <th className="py-2.5 px-3">Microsetor</th>
            <th className="py-2.5 px-3">Offset Físico</th>
            <th className="py-2.5 px-3">Dados Ativos (Não-0xFF)</th>
            <th className="py-2.5 px-3">Header (Primeiros 8 Bytes)</th>
            <th className="py-2.5 px-3 text-right">Diagnóstico</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-800/60">
          {report.microsectors.length === 0 ? (
            <tr>
              <td colSpan={5} className="py-8 text-center text-xs text-slate-500 font-mono">
                Nenhum microsetor identificado na imagem da NVRAM
              </td>
            </tr>
          ) : (
            report.microsectors.map((sec: MicroSectorAudit) => {
              const pct = ((sec.active_bytes / sec.total_bytes) * 100).toFixed(1);
              return (
                <tr key={sec.sector_name} className="hover:bg-slate-800/40 transition-colors">
                  <td className="py-2.5 px-3 font-bold text-slate-200">
                    {sec.sector_name}
                  </td>
                  <td className="py-2.5 px-3 text-cyan-400">
                    0x{sec.offset.toString(16).padStart(8, '0').toUpperCase()}
                  </td>
                  <td className="py-2.5 px-3">
                    <div className="flex items-center gap-2">
                      <span className={cleanSuccess ? 'text-slate-400' : 'text-amber-400 font-bold'}>
                        {sec.active_bytes} / {sec.total_bytes} ({pct}%)
                      </span>
                      <div className="w-16 h-1.5 rounded-full bg-slate-800 overflow-hidden">
                        <div
                          className={`h-full ${cleanSuccess ? 'bg-emerald-400' : 'bg-amber-400'}`}
                          style={{ width: `${pct}%` }}
                        />
                      </div>
                    </div>
                  </td>
                  <td className="py-2.5 px-3 text-slate-300 tracking-wider">
                    {sec.header_hex}
                  </td>
                  <td className="py-2.5 px-3 text-right">
                    {cleanSuccess ? (
                      <StatusBadge variant="success" label="VIRGEM 0xFF" />
                    ) : sec.sector_name === 'SA137' ? (
                      <StatusBadge variant="error" label="U99 REBOOT LOOP" />
                    ) : (
                      <StatusBadge variant="warning" label="EEMUL ATIVA" />
                    )}
                  </td>
                </tr>
              );
            })
          )}
        </tbody>
      </table>
    </div>
  </div>
);
