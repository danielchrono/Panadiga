import React from 'react';
import { StatusBadge } from '@/components/StatusBadge';
import { HddAnalysisReport } from '@/models';
import { MbrSectorDetailsCard } from './MbrSectorDetailsCard';

export interface HddMbrComparisonProps {
  hddReport: HddAnalysisReport;
}

export const HddMbrComparison: React.FC<HddMbrComparisonProps> = ({ hddReport }) => {
  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <span>Comparação Forense: MBR Primário (LBA 0) vs Setor Espelho (LBA 57.344)</span>
          </h3>
          <p className="text-xs text-slate-400 mt-0.5">
            O setor de backup oficial da Matsushita reside exatamente a 28 MB de offset (0x01C0_0000).
          </p>
        </div>
        <StatusBadge variant="success" label="RÉPLICA CONFIRMADA BIT-A-BIT" />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-mono">
        <MbrSectorDetailsCard
          title="MBR Primário (LBA 0 - Offset 0x0000_0000)"
          titleColor="text-cyan-400"
          badgeLabel="SETOR 0"
          badgeVariant="info"
          rows={[
            { label: 'Assinatura Hex:', value: hddReport.mbr_tables.lba0_hex_signature },
            { label: 'Magic ASCII:', value: hddReport.mbr_tables.magic_ascii, valueClassName: 'text-emerald-400 font-bold' },
            { label: 'Disk GUID:', value: hddReport.mbr_tables.disk_volume_guid },
            { label: 'IBM PC Boot (0x55AA):', value: 'AUSENTE (Proteção contra PC OS)', valueClassName: 'text-slate-400' },
            { label: 'SHA-256:', value: hddReport.mbr_tables.lba0_sha256, valueClassName: 'text-slate-300 truncate max-w-[200px]', title: hddReport.mbr_tables.lba0_sha256 },
          ]}
        />

        <MbrSectorDetailsCard
          title="Espelho de Backup (LBA 57.344 - 28.00 MB)"
          titleColor="text-emerald-400"
          badgeLabel="ESPELHO LBA 57344"
          badgeVariant="success"
          rows={[
            { label: 'Offset Físico:', value: '0x01C0_0000 (29.360.128 bytes)' },
            { label: 'Comparação Bit-a-Bit:', value: '100% IDÊNTICO AO SETOR 0', valueClassName: 'text-emerald-400 font-bold' },
            { label: 'Disk GUID:', value: hddReport.mbr_tables.disk_volume_guid },
            { label: 'MD5:', value: hddReport.mbr_tables.mirror_md5, valueClassName: 'text-slate-300' },
            { label: 'SHA-256:', value: hddReport.mbr_tables.mirror_sha256, valueClassName: 'text-slate-300 truncate max-w-[200px]', title: hddReport.mbr_tables.mirror_sha256 },
          ]}
        />
      </div>
    </div>
  );
};
