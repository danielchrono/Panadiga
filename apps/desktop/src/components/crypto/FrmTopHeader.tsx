import React, { useState, useEffect } from 'react';
import { FolderOpen, Lock, RefreshCw } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { Button } from '@/components/ui';
import { useTranslation } from '@/i18n/useTranslation';

export interface FrmTopHeaderProps {
  frmFile?: string;
  containers?: string[];
  selectedPath?: string;
  onSelectPath?: (path: string) => void;
  onRefresh?: () => void;
  loading?: boolean;
}

const PRESET_CONTAINERS = [
  'firmware_and_images/PANA_DVD.FRM',
  'firmware_and_images/PANA__SD.FRM',
  'firmware_and_images/DIGADIAG.FRM',
];

export const FrmTopHeader: React.FC<FrmTopHeaderProps> = ({
  frmFile,
  containers = [],
  selectedPath = 'firmware_and_images/PANA_DVD.FRM',
  onSelectPath,
  onRefresh,
  loading = false,
}) => {
  const { t } = useTranslation();
  const [draftPath, setDraftPath] = useState(selectedPath);

  useEffect(() => {
    setDraftPath(selectedPath);
  }, [selectedPath]);

  const options = Array.from(new Set([...containers, ...PRESET_CONTAINERS]));

  const handleLoad = (e: React.FormEvent) => {
    e.preventDefault();
    if (draftPath.trim() && onSelectPath) {
      onSelectPath(draftPath.trim());
    }
  };

  return (
    <div className="flex flex-col gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl font-mono">
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
        <div className="flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/30">
            <Lock className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2 flex-wrap">
              <h2 className="text-base font-bold text-slate-100">
                {t('crypto.title')}
              </h2>
              <StatusBadge variant="info" label={t('crypto.badge')} />
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              {t('crypto.subtitle')}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          <span className="text-xs text-slate-400">
            {t('crypto.signature')}{' '}
            <span className="text-purple-400 font-bold">50 41 4E 41 5F 46 52 4D</span>
          </span>
        </div>
      </div>

      {onSelectPath && (
        <form
          onSubmit={handleLoad}
          className="flex flex-col lg:flex-row items-stretch lg:items-center gap-2 pt-3 border-t border-slate-800/80 text-xs"
        >
          <div className="flex items-center gap-2 flex-wrap">
            <FolderOpen className="w-3.5 h-3.5 text-purple-400 shrink-0" />
            <select
              aria-label="Selecionar contêiner .FRM"
              value={options.includes(selectedPath) ? selectedPath : ''}
              onChange={e => {
                if (e.target.value) {
                  setDraftPath(e.target.value);
                  onSelectPath(e.target.value);
                }
              }}
              className="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-xs text-slate-200 focus:outline-none focus:ring-1 focus:ring-purple-500"
            >
              {options.map(path => (
                <option key={path} value={path}>
                  {path}
                </option>
              ))}
            </select>
          </div>

          <input
            type="text"
            value={draftPath}
            onChange={e => setDraftPath(e.target.value)}
            placeholder={frmFile || 'firmware_and_images/PANA_DVD.FRM'}
            className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-purple-500"
          />

          <div className="flex items-center gap-2">
            <Button
              type="submit"
              size="sm"
              variant="secondary"
              disabled={loading || !draftPath.trim()}
              className="text-xs font-semibold gap-1.5"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
              <span>Analisar .FRM</span>
            </Button>
            {onRefresh && (
              <Button
                type="button"
                size="sm"
                variant="outline"
                disabled={loading}
                onClick={onRefresh}
                className="text-xs"
              >
                Atualizar
              </Button>
            )}
          </div>
        </form>
      )}
    </div>
  );
};
