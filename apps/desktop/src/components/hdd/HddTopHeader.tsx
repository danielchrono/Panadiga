import React from 'react';
import { HardDrive, Download, FolderOpen } from 'lucide-react';
import { Button } from '@/components/ui';
import { StatusBadge } from '@/components/StatusBadge';
import { useTranslation } from '@/i18n/useTranslation';
import { HddAnalysisReport } from '@/models';

export interface HddTopHeaderProps {
  report?: HddAnalysisReport | null;
  extracting: boolean;
  onExtractPartitions: () => void;
  onSelectImage?: () => void;
}

export const HddTopHeader: React.FC<HddTopHeaderProps> = ({
  report,
  extracting,
  onExtractPartitions,
  onSelectImage,
}) => {
  const { t } = useTranslation();

  const fileName = report ? report.image_path.split(/[/\\]/).pop() || report.image_path : null;
  const isGb = (report?.file_size_mb ?? 0) >= 1024;
  const sizeFormatted = isGb
    ? `${((report?.file_size_mb ?? 0) / 1024).toFixed(2)} GB`
    : `${report?.file_size_mb.toFixed(2)} MB`;

  return (
    <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
      <div className="flex items-center gap-3.5">
        <div className="p-3 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30">
          <HardDrive className="w-6 h-6" />
        </div>
        <div>
          <div className="flex items-center gap-2 flex-wrap">
            <h2 className="text-base font-bold text-slate-100">
              {fileName ? fileName : t('hdd.noImageTitle', 'Nenhuma Imagem de Disco Carregada')}
            </h2>
            {report ? (
              report.mbr_tables.mirror_is_bit_identical ? (
                <StatusBadge variant="success" label={t('hdd.badgeOk', 'MBR & ESPELHO 100% OK')} />
              ) : (
                <StatusBadge variant="error" label="ESPELHO COM DIVERGÊNCIA" />
              )
            ) : (
              <StatusBadge variant="neutral" label="OFFLINE" />
            )}
          </div>
          <p className="text-xs text-slate-400 mt-0.5">
            {report
              ? `${sizeFormatted} • ${report.total_sectors.toLocaleString()} setores LBA • Magic: ${report.mbr_tables.magic_ascii || 'PAD_HDD'}`
              : t('hdd.noImageSub', 'Selecione um arquivo de imagem .bin / .raw ou acerte um disco físico na aba Gravação Direta')}
          </p>
        </div>
      </div>

      <div className="flex items-center gap-3 shrink-0 flex-wrap">
        {onSelectImage && (
          <Button
            type="button"
            variant="secondary"
            onClick={onSelectImage}
            className="font-bold text-xs gap-2"
          >
            <FolderOpen className="w-4 h-4 text-cyan-400" />
            <span>{t('hdd.openImage', 'Abrir Imagem')}</span>
          </Button>
        )}

        <Button
          onClick={onExtractPartitions}
          disabled={extracting || !report}
          className="px-5 bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold text-xs gap-2 shadow-xl shadow-cyan-950/50"
        >
          {extracting ? (
            <span>{t('hdd.extracting', 'Extraindo Partições...')}</span>
          ) : (
            <>
              <Download className="w-4 h-4" />
              <span>{t('hdd.extractButton', 'Extrair Partições (SHA256SUMS)')}</span>
            </>
          )}
        </Button>
      </div>
    </div>
  );
};
