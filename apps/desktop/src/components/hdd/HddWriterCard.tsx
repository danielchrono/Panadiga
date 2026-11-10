import React, { useState, useEffect } from 'react';
import { HardDrive, AlertTriangle, ShieldCheck, Flame, RefreshCw, CheckCircle2 } from 'lucide-react';
import { StatusBadge } from '@/components/StatusBadge';
import { PhysicalDriveInfo, DriveWriteReport } from '@/models';
import { invokeCommand } from '@/services/ipc';
import { useToast } from '@/context/ToastContext';

export const HddWriterCard: React.FC = () => {
  const { showToast } = useToast();
  const [drives, setDrives] = useState<PhysicalDriveInfo[]>([]);
  const [selectedDrive, setSelectedDrive] = useState<PhysicalDriveInfo | null>(null);
  const [loadingDrives, setLoadingDrives] = useState<boolean>(false);
  const [confirmInput, setConfirmInput] = useState<string>('');
  const [executing, setExecuting] = useState<boolean>(false);
  const [writeReport, setWriteReport] = useState<DriveWriteReport | null>(null);

  const fetchDrives = async () => {
    setLoadingDrives(true);
    try {
      const list = await invokeCommand<PhysicalDriveInfo[]>('list_physical_drives');
      setDrives(list);
      const safe = list.find(d => d.safe_to_write);
      if (safe) setSelectedDrive(safe);
    } catch (err) {
      showToast({
        type: 'error',
        title: 'Erro ao Listar Discos Físicos',
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setLoadingDrives(false);
    }
  };

  useEffect(() => {
    fetchDrives();
  }, []);

  const getExpectedToken = (devicePath: string) => {
    return `CONFIRM-${devicePath.replace(/^\\\\.\\/, '')}`;
  };

  const handleProceduralFormat = async () => {
    if (!selectedDrive) return;
    const expected = getExpectedToken(selectedDrive.device_path);
    if (confirmInput.trim() !== expected) {
      showToast({
        type: 'error',
        title: 'Token de Confirmação Inválido',
        message: `Digite exatamente "${expected}" para desbloquear a gravação física.`,
      });
      return;
    }

    setExecuting(true);
    try {
      const sectors = selectedDrive.size_bytes > 0
        ? Math.floor(selectedDrive.size_bytes / 512)
        : 200_000;
      const res = await invokeCommand<DriveWriteReport>('format_hdd_meihdfs', {
        devicePath: selectedDrive.device_path,
        totalSectors: sectors,
        confirmationToken: confirmInput.trim(),
      });
      setWriteReport(res);
      showToast({
        type: 'success',
        title: 'Formatação Concluída',
        message: res.message,
      });
    } catch (err) {
      showToast({
        type: 'error',
        title: 'Falha na Formatação MEIHDFS',
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setExecuting(false);
    }
  };

  const handleWriteRawImage = async () => {
    if (!selectedDrive) return;
    const expected = getExpectedToken(selectedDrive.device_path);
    if (confirmInput.trim() !== expected) {
      showToast({
        type: 'error',
        title: 'Token de Confirmação Inválido',
        message: `Digite exatamente "${expected}" para desbloquear a gravação da imagem.`,
      });
      return;
    }

    setExecuting(true);
    try {
      const res = await invokeCommand<DriveWriteReport>('write_hdd_image', {
        devicePath: selectedDrive.device_path,
        confirmationToken: confirmInput.trim(),
      });
      setWriteReport(res);
      showToast({
        type: 'success',
        title: 'Gravação Concluída',
        message: res.message,
      });
    } catch (err) {
      showToast({
        type: 'error',
        title: 'Falha na Gravação da Imagem',
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setExecuting(false);
    }
  };

  const handleRestoreMirrors = async () => {
    if (!selectedDrive) return;
    const expected = getExpectedToken(selectedDrive.device_path);
    if (confirmInput.trim() !== expected) {
      showToast({
        type: 'error',
        title: 'Token de Confirmação Inválido',
        message: `Digite exatamente "${expected}" para restaurar via espelho LBA 57.344.`,
      });
      return;
    }

    setExecuting(true);
    try {
      const res = await invokeCommand<DriveWriteReport>('restore_hdd_mirrors', {
        devicePath: selectedDrive.device_path,
        confirmationToken: confirmInput.trim(),
      });
      setWriteReport(res);
      showToast({
        type: 'success',
        title: 'Restauração Concluída',
        message: res.message,
      });
    } catch (err) {
      showToast({
        type: 'error',
        title: 'Falha na Restauração dos Espelhos',
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setExecuting(false);
    }
  };

  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-6 shadow-xl space-y-6 font-mono">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-slate-800 pb-4">
        <div>
          <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
            <HardDrive className="w-5 h-5 text-cyan-400" />
            <span>Gravador Físico de Disco & MEIHDFS Formatter (mkfs.meihdfs)</span>
          </h3>
          <p className="text-xs text-slate-400 mt-1">
            Gravação física direta em unidades PATA/IDE com trava de segurança de SO host e token anti-acidente.
          </p>
        </div>
        <button
          onClick={fetchDrives}
          disabled={loadingDrives}
          className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-750 text-slate-300 text-xs font-semibold flex items-center gap-1.5 transition-colors"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loadingDrives ? 'animate-spin' : ''}`} />
          <span>Atualizar Discos</span>
        </button>
      </div>

      {/* Safety Invariant Alert */}
      <div className="p-4 rounded-xl bg-amber-950/30 border border-amber-800/60 text-xs space-y-2">
        <div className="flex items-center gap-2 text-amber-400 font-bold">
          <AlertTriangle className="w-4 h-4 shrink-0" />
          <span>INVARIANTE DE SEGURANÇA: BLOQUEIO INCONDICIONAL DO DISCO DE BOOT</span>
        </div>
        <p className="text-slate-300 leading-relaxed text-[11px]">
          O disco do sistema operacional host (ex: <code className="text-rose-400">\\.\PhysicalDrive0</code> no Windows ou <code className="text-rose-400">/dev/sda</code> no Linux) possui bloqueio incondicional em hardware e kernel.
          Qualquer tentativa de escrita no disco de boot é rejeitada antes de abrir o descritor de dispositivo.
        </p>
      </div>

      {/* Drive Selection Grid */}
      <div className="space-y-3">
        <label className="text-xs text-slate-400 font-semibold uppercase">
          Selecione a Unidade Física de Destino:
        </label>
        {drives.length === 0 ? (
          <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-500 text-center">
            Nenhuma unidade de armazenamento física adicional detectada.
          </div>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {drives.map(drive => {
              const isSelected = selectedDrive?.device_path === drive.device_path;
              return (
                <div
                  key={drive.device_path}
                  onClick={() => drive.safe_to_write && setSelectedDrive(drive)}
                  className={`p-4 rounded-xl border transition-all ${
                    drive.is_system_drive
                      ? 'bg-rose-950/20 border-rose-900/60 opacity-60 cursor-not-allowed'
                      : isSelected
                        ? 'bg-cyan-500/10 border-cyan-500/50 shadow-lg cursor-pointer'
                        : 'bg-slate-950 border-slate-800 hover:border-slate-700 cursor-pointer'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-bold text-xs text-slate-200">
                      {drive.display_name}
                    </span>
                    {drive.is_system_drive ? (
                      <StatusBadge variant="error" label="BLOQUEADO (SO)" />
                    ) : (
                      <StatusBadge variant="success" label="SEGURO PARA GRAVAÇÃO" />
                    )}
                  </div>
                  <span className="text-[11px] font-mono text-cyan-400 block">
                    {drive.device_path}
                  </span>
                  {drive.lockout_reason && (
                    <span className="text-[10px] text-rose-400 block mt-1">
                      {drive.lockout_reason}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Confirmation Token & Execution Actions */}
      {selectedDrive && selectedDrive.safe_to_write && (
        <div className="p-5 rounded-xl bg-slate-950 border border-slate-800 space-y-4">
          <div className="flex items-center gap-2 text-cyan-300 font-semibold text-xs">
            <ShieldCheck className="w-4 h-4 text-cyan-400" />
            <span>Confirmação Deliberada para Gravação em {selectedDrive.device_path}</span>
          </div>

          <div className="space-y-1.5">
            <label className="text-[11px] text-slate-400">
              Digite o token de segurança <code className="text-amber-400 font-bold">{getExpectedToken(selectedDrive.device_path)}</code> para liberar a operação:
            </label>
            <input
              type="text"
              value={confirmInput}
              onChange={e => setConfirmInput(e.target.value)}
              placeholder={getExpectedToken(selectedDrive.device_path)}
              className="w-full px-3 py-2 rounded-xl bg-slate-900 border border-slate-700 text-xs text-amber-300 focus:outline-none focus:border-amber-500 font-mono"
            />
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-2">
            <button
              onClick={handleProceduralFormat}
              disabled={executing || confirmInput.trim() !== getExpectedToken(selectedDrive.device_path)}
              className="py-2.5 px-3 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-slate-950 font-bold text-xs flex items-center justify-center gap-1.5 transition-all disabled:opacity-40"
            >
              <HardDrive className="w-4 h-4" />
              <span>Formatar mkfs.meihdfs</span>
            </button>

            <button
              onClick={handleWriteRawImage}
              disabled={executing || confirmInput.trim() !== getExpectedToken(selectedDrive.device_path)}
              className="py-2.5 px-3 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold text-xs flex items-center justify-center gap-1.5 transition-all disabled:opacity-40"
            >
              <Flame className="w-4 h-4" />
              <span>Gravar Imagem Raw (100MB)</span>
            </button>

            <button
              onClick={handleRestoreMirrors}
              disabled={executing || confirmInput.trim() !== getExpectedToken(selectedDrive.device_path)}
              className="py-2.5 px-3 rounded-xl bg-purple-500 hover:bg-purple-400 text-slate-950 font-bold text-xs flex items-center justify-center gap-1.5 transition-all disabled:opacity-40"
            >
              <ShieldCheck className="w-4 h-4" />
              <span>Restaurar Espelho LBA 57k</span>
            </button>
          </div>
        </div>
      )}

      {/* Execution Report Notification */}
      {writeReport && (
        <div className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/40 text-xs space-y-1">
          <div className="flex items-center gap-2 text-emerald-300 font-bold">
            <CheckCircle2 className="w-4 h-4" />
            <span>{writeReport.operation} Realizada com Sucesso!</span>
          </div>
          <p className="text-slate-300 text-[11px]">
            {writeReport.message} ({writeReport.bytes_written.toLocaleString()} bytes / {writeReport.sectors_written.toLocaleString()} setores gravados em {writeReport.device_path}).
          </p>
        </div>
      )}
    </div>
  );
};
