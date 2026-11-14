import React from 'react';
import { Layers, Cpu, KeyRound } from 'lucide-react';
import { FrmTocDumpReport } from '@/models';
import { StatusBadge } from '@/components/StatusBadge';
import { useTranslation } from '@/i18n/useTranslation';

interface FrmTocTabProps {
  tocReport: FrmTocDumpReport;
}

export const FrmTocTab: React.FC<FrmTocTabProps> = ({ tocReport }) => {
  const { t } = useTranslation();
  const decryptedRows = tocReport.decrypted_toc ?? [];

  return (
    <div className="space-y-6 font-mono">
      {/* Decrypted Feistel 48B / Plaintext SD 64B Firm List Table */}
      <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <div>
            <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
              <KeyRound className="w-4 h-4 text-purple-400" />
              <span>{t('crypto.tocSection.decryptedTitle')}</span>
            </h3>
            <p className="text-xs text-slate-400 mt-0.5">{t('crypto.tocSection.desc')}</p>
          </div>
          <StatusBadge
            variant={tocReport.toc_invariants_valid ? 'success' : 'warning'}
            label={`Formato: ${tocReport.toc_format || 'unknown'}`}
          />
        </div>

        {decryptedRows.length === 0 ? (
          <div className="p-6 text-center text-xs text-slate-500 rounded-xl bg-slate-950 border border-slate-800">
            {t('crypto.tocSection.decryptedEmpty')}
          </div>
        ) : (
          <div className="overflow-x-auto rounded-xl border border-slate-800 bg-slate-950">
            <table className="w-full text-left text-xs">
              <thead className="bg-bench-950 text-[11px] uppercase text-slate-400 border-b border-slate-800">
                <tr>
                  <th className="py-2.5 px-3">#</th>
                  <th className="py-2.5 px-3">Slot</th>
                  <th className="py-2.5 px-3">Módulo</th>
                  <th className="py-2.5 px-3">HW / Modelo</th>
                  <th className="py-2.5 px-3">Offset</th>
                  <th className="py-2.5 px-3">Tamanho</th>
                  <th className="py-2.5 px-3">FVU CRC32</th>
                  <th className="py-2.5 px-3">list_sum</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/70">
                {decryptedRows.map(row => (
                  <tr key={`${row.index}-${row.module_name}`} className="hover:bg-slate-900/50">
                    <td className="py-2 px-3 text-slate-500">{row.index}</td>
                    <td className="py-2 px-3 text-slate-400">{row.entry_offset_hex}</td>
                    <td className="py-2 px-3">
                      <span className="font-bold text-cyan-300">{row.module_name}</span>
                      <span className="block text-[10px] text-slate-500">
                        ver {row.version_u16_hex} • flag 0x{row.target_flag.toString(16).padStart(2, '0')}
                      </span>
                    </td>
                    <td className="py-2 px-3 text-slate-300">
                      {row.hardware_family} / {row.model_group}
                    </td>
                    <td className="py-2 px-3 text-purple-300">
                      0x{row.module_offset.toString(16).toUpperCase().padStart(8, '0')}
                    </td>
                    <td className="py-2 px-3 text-slate-200">
                      {row.module_size.toLocaleString()} B
                    </td>
                    <td className="py-2 px-3 text-slate-400">
                      0x{row.fvu_checksum_u32.toString(16).toUpperCase().padStart(8, '0')}
                    </td>
                    <td className="py-2 px-3">
                      <StatusBadge
                        variant={row.list_sum_valid ? 'success' : 'error'}
                        label={`0x${row.list_sum_u16.toString(16).toUpperCase().padStart(4, '0')} ${row.list_sum_valid ? 'OK' : 'ERR'}`}
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <Layers className="w-4 h-4 text-cyan-400" />
            <span>{t('crypto.tocSection.title')}</span>
          </h3>
          <p className="text-xs text-slate-400 mt-0.5">
            {t('crypto.tocSection.desc')}
          </p>
        </div>

        {tocReport.toc_entries.length === 0 ? (
          <div className="p-8 text-center text-xs text-slate-500 font-mono rounded-xl bg-slate-950 border border-slate-800">
            {t('crypto.tocSection.empty')}
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {tocReport.toc_entries.map(entry => (
              <div
                key={entry.index}
                className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3 text-xs"
              >
                <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                  <span className="font-bold text-cyan-400">{entry.target_name}</span>
                  <span className="text-[10px] text-slate-500">{entry.offset_hex}</span>
                </div>

                <div className="space-y-1.5 text-slate-400">
                  <div className="flex justify-between">
                    <span>{t('crypto.tocSection.arch')}</span>
                    <span className="text-slate-200">{entry.target_architecture}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>{t('crypto.tocSection.familyId')}</span>
                    <span className="text-emerald-400">{entry.processor_family_id}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>{t('crypto.tocSection.descriptor')}</span>
                    <span className="text-slate-200">{entry.target_descriptor_id}</span>
                  </div>
                </div>

                <div className="pt-2 border-t border-slate-800">
                  <span className="text-[10px] text-slate-500 block mb-1">{t('crypto.tocSection.dump')}</span>
                  <p className="text-[10px] text-slate-400 break-all bg-bench-950 p-2 rounded border border-slate-900">
                    {entry.raw_hex}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Module Layout */}
      <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-3">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400 flex items-center gap-2">
          <Cpu className="w-4 h-4 text-emerald-400" />
          <span>{t('crypto.tocSection.modulesTitle')}</span>
        </h3>

        <div className="space-y-2.5">
          {tocReport.modules.length === 0 ? (
            <div className="p-6 text-center text-xs text-slate-500 font-mono rounded-xl bg-slate-950 border border-slate-800">
              {t('crypto.tocSection.modulesEmpty')}
            </div>
          ) : (
            tocReport.modules.map(mod => (
              <div
                key={mod.module_index}
                className="p-3.5 rounded-xl bg-slate-950 border border-slate-800 flex flex-col md:flex-row items-start md:items-center justify-between gap-3 text-xs"
              >
                <div>
                  <span className="font-bold text-slate-200">{mod.name}</span>
                  <span className="text-slate-500 text-[11px] block">{mod.target} • {mod.hdd_correlation}</span>
                </div>

                <div className="flex items-center gap-4 text-xs font-mono text-slate-400 shrink-0">
                  <span>{mod.start_offset_hex} .. {mod.end_offset_hex}</span>
                  <span className="text-cyan-400 font-bold">{mod.payload_size_bytes.toLocaleString()} bytes</span>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
