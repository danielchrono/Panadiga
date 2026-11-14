import React, { useState } from 'react';
import { AlertCircle, CheckCircle2, FileArchive, RefreshCw } from 'lucide-react';
import { ExtrheadUnpackReport } from '@/models';
import { UnpackExtrheadParams } from '@/viewmodels/useFrmPackager';
import { StatusBadge } from '@/components/StatusBadge';
import { Button } from '@/components/ui';
import { useTranslation } from '@/i18n/useTranslation';

interface ExtrheadUnpackCardProps {
  unpackLoading: boolean;
  unpackError: string | null;
  unpackResult: ExtrheadUnpackReport | null;
  onUnpack: (params: UnpackExtrheadParams) => Promise<ExtrheadUnpackReport | null>;
}

export const ExtrheadUnpackCard: React.FC<ExtrheadUnpackCardProps> = ({
  unpackLoading,
  unpackError,
  unpackResult,
  onUnpack,
}) => {
  const { t } = useTranslation();
  const [inputPath, setInputPath] = useState('firmware_and_images/DIGADIAG.FRM');
  const [mode, setMode] = useState<'auto' | 'main' | 'drv' | 'plain'>('auto');
  const [outPath, setOutPath] = useState('');

  const submitDisabled = unpackLoading || !inputPath.trim();

  const handleUnpack = async (e: React.FormEvent) => {
    e.preventDefault();
    if (submitDisabled) return;
    await onUnpack({
      inputPath: inputPath.trim(),
      mode,
      outPath: outPath.trim() || undefined,
    });
  };

  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <FileArchive className="w-4 h-4 text-cyan-400" />
            <span>{t('crypto.packager.unpackTitle')}</span>
          </h3>
          <p className="text-xs text-slate-400 mt-0.5">{t('crypto.packager.unpackDesc')}</p>
        </div>
        <StatusBadge variant="neutral" label={mode.toUpperCase()} />
      </div>

      <form onSubmit={handleUnpack} className="grid grid-cols-1 md:grid-cols-12 gap-3 text-xs items-end">
        <label className="md:col-span-5 space-y-1 block">
          <span className="text-slate-400 block">Arquivo EXTRHEAD / Módulo</span>
          <input
            type="text"
            value={inputPath}
            onChange={e => setInputPath(e.target.value)}
            placeholder="firmware_and_images/out/MAIN3060NP.bin"
            className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200"
          />
        </label>

        <label className="md:col-span-2 space-y-1 block">
          <span className="text-slate-400 block">Perfil Feistel</span>
          <select
            value={mode}
            onChange={e => setMode(e.target.value as 'auto' | 'main' | 'drv' | 'plain')}
            className="w-full bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-2 text-slate-200"
          >
            <option value="auto">auto</option>
            <option value="main">main (0x19000)</option>
            <option value="drv">drv (0x2800)</option>
            <option value="plain">plain</option>
          </select>
        </label>

        <label className="md:col-span-3 space-y-1 block">
          <span className="text-slate-400 block">Saída descomprimida (opcional)</span>
          <input
            type="text"
            value={outPath}
            onChange={e => setOutPath(e.target.value)}
            placeholder="firmware_and_images/out/unpacked.bin"
            className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200"
          />
        </label>

        <div className="md:col-span-2">
          <Button
            type="submit"
            size="sm"
            variant="secondary"
            disabled={submitDisabled}
            className="w-full gap-1.5 font-bold"
          >
            {unpackLoading && <RefreshCw className="w-3.5 h-3.5 animate-spin" />}
            <span>{t('crypto.packager.unpackSubmit')}</span>
          </Button>
        </div>
      </form>

      {unpackError && (
        <div className="p-3 rounded-lg bg-rose-500/10 border border-rose-500/30 text-xs text-rose-300 flex items-start gap-2">
          <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
          <span>{unpackError}</span>
        </div>
      )}

      {!unpackLoading && !unpackError && !unpackResult && (
        <p className="text-[11px] text-slate-500 bg-slate-950 p-3 rounded-lg border border-slate-800">
          {t('crypto.packager.unpackIdle')}
        </p>
      )}

      {unpackResult && (
        <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-2 text-xs">
          <div className="flex items-center justify-between flex-wrap gap-2">
            <span className="font-bold text-slate-100 flex items-center gap-1.5">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              <span>
                Tag `{unpackResult.header.tag}` • Build `{unpackResult.header.build_name}` •{' '}
                {unpackResult.decompressed_bytes.toLocaleString()} bytes
              </span>
            </span>
            <div className="flex items-center gap-2">
              <StatusBadge
                variant={unpackResult.checksum_valid ? 'success' : 'error'}
                label={
                  unpackResult.checksum_valid
                    ? `Adler32 OK (${unpackResult.calculated_checksum_hex})`
                    : `Mismatch (${unpackResult.calculated_checksum_hex} != ${unpackResult.header_checksum_hex})`
                }
              />
              <StatusBadge
                variant={unpackResult.footer_present ? 'info' : 'warning'}
                label={unpackResult.footer_present ? 'EXTRFOOT OK' : 'NO EXTRFOOT'}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-[11px] text-slate-400 pt-1">
            <div>dst_addr: 0x{unpackResult.header.dst_addr.toString(16).toUpperCase()}</div>
            <div>dst_len: {unpackResult.header.dst_len.toLocaleString()} B</div>
            <div>src_len: {unpackResult.header.src_len.toLocaleString()} B</div>
            <div>step: {unpackResult.header.adler_step} B</div>
          </div>

          {unpackResult.written_output_path && (
            <p className="text-[11px] text-cyan-300">
              Gravado em: {unpackResult.written_output_path}
            </p>
          )}
        </div>
      )}
    </div>
  );
};
