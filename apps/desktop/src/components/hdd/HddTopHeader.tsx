import React from 'react';
import { HardDrive, Download } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { useTranslation } from '@/i18n/useTranslation';

export interface HddTopHeaderProps {
  extracting: boolean;
  onExtractPartitions: () => void;
}

export const HddTopHeader: React.FC<HddTopHeaderProps> = ({
  extracting,
  onExtractPartitions,
}) => {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
      <div className="flex items-center gap-3.5">
        <div className="p-3 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30">
          <HardDrive className="w-6 h-6" />
        </div>
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-base font-bold text-slate-100">
              {t('hdd.title', 'Hitachi PATA 100 MB Forensic Image (MEIHDFS)')}
            </h2>
            <StatusBadge variant="success" label={t('hdd.badgeOk', 'MBR & ESPELHO 100% OK')} />
          </div>
          <p className="text-xs text-slate-400 mt-0.5">
            {t('hdd.subtitle', 'panasonic DMR EH-55 Firmware hdd_100.bin • 204.800 setores LBA de 512 bytes')}
          </p>
        </div>
      </div>

      <div className="flex items-center gap-3 shrink-0">
        <button
          onClick={onExtractPartitions}
          disabled={extracting}
          className="px-5 py-2.5 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold text-xs flex items-center gap-2 shadow-xl shadow-cyan-950/50 transition-all focus:outline-none focus:ring-2 focus:ring-cyan-300 disabled:opacity-50"
        >
          {extracting ? (
            <span>{t('hdd.extracting', 'Extraindo Partições...')}</span>
          ) : (
            <>
              <Download className="w-4 h-4" />
              <span>{t('hdd.extractButton', 'Extrair Partições (SHA256SUMS)')}</span>
            </>
          )}
        </button>
      </div>
    </div>
  );
};
