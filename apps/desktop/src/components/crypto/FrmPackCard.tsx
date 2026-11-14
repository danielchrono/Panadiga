import React, { useState } from 'react';
import { AlertCircle, CheckCircle2, PackagePlus, Plus, RefreshCw, Trash2 } from 'lucide-react';
import { FrmPackModuleInput, FrmPackReport, PackFrmRequest } from '@/models';
import { StatusBadge } from '@/components/StatusBadge';
import { Button } from '@/components/ui';
import { useTranslation } from '@/i18n/useTranslation';

interface FrmPackCardProps {
  packLoading: boolean;
  packError: string | null;
  packResult: FrmPackReport | null;
  onPack: (request: PackFrmRequest) => Promise<FrmPackReport | null>;
  onInspectOutput: (outPath: string) => void;
}

const DEFAULT_OUT_BY_MODE: Record<'sd' | 'dvd' | 'digadiag', string> = {
  sd: 'firmware_and_images/out/PANA__SD.FRM',
  dvd: 'firmware_and_images/out/PANA_DVD.FRM',
  digadiag: 'firmware_and_images/out/DIGADIAG.FRM',
};

export const FrmPackCard: React.FC<FrmPackCardProps> = ({
  packLoading,
  packError,
  packResult,
  onPack,
  onInspectOutput,
}) => {
  const { t } = useTranslation();
  const [mode, setMode] = useState<'sd' | 'dvd' | 'digadiag'>('sd');
  const [outPath, setOutPath] = useState(DEFAULT_OUT_BY_MODE.sd);
  const [modules, setModules] = useState<FrmPackModuleInput[]>([
    { name: 'DRV S211', path: '' },
    { name: 'PROG061 ', path: '' },
    { name: 'MAIN3060NP', path: '' },
  ]);
  const [hwFamily, setHwFamily] = useState('M6930');
  const [modelGroup, setModelGroup] = useState('NP-309');
  const [versionU16, setVersionU16] = useState('0x0001');

  const [diagInputPath, setDiagInputPath] = useState('');
  const [dstAddr, setDstAddr] = useState('0x50400000');
  const [tag, setTag] = useState('FVU ');
  const [buildName, setBuildName] = useState('DIAG01');

  const handleModeChange = (nextMode: 'sd' | 'dvd' | 'digadiag') => {
    setMode(nextMode);
    if (Object.values(DEFAULT_OUT_BY_MODE).includes(outPath)) {
      setOutPath(DEFAULT_OUT_BY_MODE[nextMode]);
    }
  };

  const validModules = modules.filter(m => m.name.trim() && m.path.trim());
  const isSubmitDisabled =
    packLoading ||
    !outPath.trim() ||
    (mode === 'digadiag' ? !diagInputPath.trim() : validModules.length === 0);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (isSubmitDisabled) return;

    await onPack(
      mode === 'digadiag'
        ? {
            mode,
            out_path: outPath.trim(),
            input_path: diagInputPath.trim(),
            dst_addr: dstAddr.trim() || '0x50400000',
            tag: tag || 'FVU ',
            build_name: buildName.trim() || 'DIAG01',
          }
        : {
            mode,
            out_path: outPath.trim(),
            modules: validModules,
            hw_family: hwFamily.trim() || 'M6930',
            model_group: modelGroup.trim() || 'NP-309',
            version_u16: versionU16.trim() || '0x0001',
          }
    );
  };

  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <PackagePlus className="w-4 h-4 text-emerald-400" />
            <span>{t('crypto.packager.packTitle')}</span>
          </h3>
          <p className="text-xs text-slate-400 mt-0.5">{t('crypto.packager.packDesc')}</p>
        </div>
        <StatusBadge variant="info" label={mode.toUpperCase()} />
      </div>

      <form onSubmit={handleSubmit} className="space-y-3 text-xs">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          <label className="space-y-1 block">
            <span className="text-slate-400 block">{t('crypto.packager.packMode')}</span>
            <select
              value={mode}
              onChange={e => handleModeChange(e.target.value as 'sd' | 'dvd' | 'digadiag')}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200"
            >
              <option value="sd">sd — PANA__SD.FRM (TOC SD 64B plaintext)</option>
              <option value="dvd">dvd — PANA_DVD.FRM (TOC Feistel 48B)</option>
              <option value="digadiag">digadiag — DIGADIAG.FRM (128 KiB FVU)</option>
            </select>
          </label>

          <label className="space-y-1 block">
            <span className="text-slate-400 block">{t('crypto.packager.outPath')}</span>
            <input
              type="text"
              value={outPath}
              onChange={e => setOutPath(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200"
            />
          </label>
        </div>

        {mode === 'digadiag' ? (
          <div className="grid grid-cols-1 md:grid-cols-4 gap-3 pt-1">
            <input
              type="text"
              placeholder="Binário entrada (ex: diag.bin)"
              value={diagInputPath}
              onChange={e => setDiagInputPath(e.target.value)}
              className="md:col-span-2 bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200"
            />
            <input
              type="text"
              placeholder="dst_addr (0x50400000)"
              value={dstAddr}
              onChange={e => setDstAddr(e.target.value)}
              className="bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200"
            />
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="Tag"
                value={tag}
                onChange={e => setTag(e.target.value)}
                className="w-20 bg-slate-950 border border-slate-800 rounded-lg px-2 py-2 text-slate-200"
              />
              <input
                type="text"
                placeholder="Build"
                value={buildName}
                onChange={e => setBuildName(e.target.value)}
                className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-2 py-2 text-slate-200"
              />
            </div>
          </div>
        ) : (
          <div className="space-y-2 pt-1">
            <div className="grid grid-cols-3 gap-2">
              <input
                type="text"
                placeholder="HW (M6930)"
                value={hwFamily}
                onChange={e => setHwFamily(e.target.value)}
                className="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-slate-200"
              />
              <input
                type="text"
                placeholder="Modelo (NP-309)"
                value={modelGroup}
                onChange={e => setModelGroup(e.target.value)}
                className="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-slate-200"
              />
              <input
                type="text"
                placeholder="Versão (0x0001)"
                value={versionU16}
                onChange={e => setVersionU16(e.target.value)}
                className="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-slate-200"
              />
            </div>

            {modules.map((mod, idx) => (
              <div key={idx} className="flex items-center gap-2">
                <input
                  type="text"
                  placeholder="Nome slot (ex: PROG061 )"
                  value={mod.name}
                  onChange={e =>
                    setModules(prev =>
                      prev.map((m, i) => (i === idx ? { ...m, name: e.target.value } : m))
                    )
                  }
                  className="w-40 bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-slate-200"
                />
                <input
                  type="text"
                  placeholder="Caminho payload (ex: firmware_and_images/out/prog061.bin)"
                  value={mod.path}
                  onChange={e =>
                    setModules(prev =>
                      prev.map((m, i) => (i === idx ? { ...m, path: e.target.value } : m))
                    )
                  }
                  className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-slate-200"
                />
                <button
                  type="button"
                  onClick={() => setModules(prev => prev.filter((_, i) => i !== idx))}
                  className="p-1.5 text-slate-500 hover:text-rose-400"
                  aria-label="Remover módulo"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            ))}

            <button
              type="button"
              onClick={() => setModules(prev => [...prev, { name: '', path: '' }])}
              className="inline-flex items-center gap-1 text-[11px] text-cyan-400 hover:text-cyan-300"
            >
              <Plus className="w-3 h-3" />
              <span>{t('crypto.packager.addModule')}</span>
            </button>
          </div>
        )}

        <div className="flex items-center gap-3 pt-1">
          <Button type="submit" size="sm" disabled={isSubmitDisabled} className="gap-1.5 font-bold">
            {packLoading && <RefreshCw className="w-3.5 h-3.5 animate-spin" />}
            <span>{t('crypto.packager.packSubmit')}</span>
          </Button>
          {packResult && (
            <Button
              type="button"
              size="sm"
              variant="outline"
              onClick={() => onInspectOutput(outPath)}
              className="text-xs"
            >
              {t('crypto.packager.inspectPacked')}
            </Button>
          )}
        </div>
      </form>

      {packError && (
        <div className="p-3 rounded-lg bg-rose-500/10 border border-rose-500/30 text-xs text-rose-300 flex items-start gap-2">
          <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
          <span>{packError}</span>
        </div>
      )}

      {!packLoading && !packError && !packResult && (
        <p className="text-[11px] text-slate-500 bg-slate-950 p-3 rounded-lg border border-slate-800">
          {t('crypto.packager.packIdle')}
        </p>
      )}

      {packResult && (
        <div className="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/30 space-y-2 text-xs">
          <div className="flex items-center justify-between">
            <span className="text-emerald-300 font-bold flex items-center gap-1.5">
              <CheckCircle2 className="w-4 h-4" />
              <span>{t('crypto.packager.packSuccess')}</span>
            </span>
            <span className="text-slate-300">
              {packResult.total_bytes.toLocaleString()} bytes • TOC {packResult.toc_region_size} B
            </span>
          </div>
          <div className="space-y-1 text-[11px] text-slate-300">
            {packResult.modules.map(m => (
              <div key={m.index} className="flex justify-between bg-slate-950/70 px-2.5 py-1 rounded">
                <span>
                  [{m.index}] {m.name} @ 0x{m.module_offset.toString(16).toUpperCase()}
                </span>
                <span className="text-cyan-300">
                  {m.module_size.toLocaleString()} B • FVU 0x{m.fvu_checksum_u32.toString(16).toUpperCase()} • sum 0x
                  {m.list_sum_u16.toString(16).toUpperCase().padStart(4, '0')}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
