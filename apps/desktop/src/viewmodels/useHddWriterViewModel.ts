import { useCallback, useEffect, useState } from 'react';
import type { DriveWriteReport, PhysicalDriveInfo } from '@/models';
import { invokeCommand } from '@/services/ipc';
import { useToast } from '@/context/ToastContext';

type HddWriteCommand = 'format_hdd_meihdfs' | 'write_hdd_image' | 'restore_hdd_mirrors';

interface HddOperationDescriptor {
  readonly cmd: HddWriteCommand;
  readonly invalidTokenSuffix: string;
  readonly successTitle: string;
  readonly errorTitle: string;
  readonly buildArgs?: (drive: PhysicalDriveInfo) => Record<string, unknown>;
}

const HDD_OPERATION_DESCRIPTORS: Readonly<Record<HddWriteCommand, HddOperationDescriptor>> = Object.freeze({
  format_hdd_meihdfs: {
    cmd: 'format_hdd_meihdfs',
    invalidTokenSuffix: 'para desbloquear a gravação física.',
    successTitle: 'Formatação Concluída',
    errorTitle: 'Falha na Formatação MEIHDFS',
    buildArgs: drive => ({
      totalSectors: drive.size_bytes > 0 ? Math.floor(drive.size_bytes / 512) : 200_000,
    }),
  },
  write_hdd_image: {
    cmd: 'write_hdd_image',
    invalidTokenSuffix: 'para desbloquear a gravação da imagem.',
    successTitle: 'Gravação Concluída',
    errorTitle: 'Falha na Gravação da Imagem',
  },
  restore_hdd_mirrors: {
    cmd: 'restore_hdd_mirrors',
    invalidTokenSuffix: 'para restaurar via espelho LBA 57.344.',
    successTitle: 'Restauração Concluída',
    errorTitle: 'Falha na Restauração dos Espelhos',
  },
});

/**
 * Pure helper computing the required confirmation token for a physical disk target.
 */
export function getExpectedDriveConfirmToken(devicePath: string): string {
  return `CONFIRM-${devicePath.replace(/^\\\\.\\/, '')}`;
}

/**
 * Single Responsibility ViewModel: manages physical ATA/IDE disk enumeration and guarded
 * MEIHDFS write operations (format, raw image write, mirror recovery) via Tauri IPC.
 */
export function useHddWriterViewModel() {
  const { showToast } = useToast();
  const [drives, setDrives] = useState<PhysicalDriveInfo[]>([]);
  const [selectedDrive, setSelectedDrive] = useState<PhysicalDriveInfo | null>(null);
  const [loadingDrives, setLoadingDrives] = useState<boolean>(false);
  const [confirmInput, setConfirmInput] = useState<string>('');
  const [executing, setExecuting] = useState<boolean>(false);
  const [writeReport, setWriteReport] = useState<DriveWriteReport | null>(null);

  const fetchDrives = useCallback(async () => {
    setLoadingDrives(true);
    try {
      const list = await invokeCommand<PhysicalDriveInfo[]>('list_physical_drives');
      setDrives(list);
      const safe = list.find(d => d.safe_to_write);
      if (safe) {
        setSelectedDrive(safe);
      }
    } catch (err) {
      showToast({
        type: 'error',
        title: 'Erro ao Listar Discos Físicos',
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setLoadingDrives(false);
    }
  }, [showToast]);

  useEffect(() => {
    void fetchDrives();
  }, [fetchDrives]);

  const runConfirmedOperation = useCallback(
    async (command: HddWriteCommand) => {
      if (!selectedDrive) return;
      const descriptor = HDD_OPERATION_DESCRIPTORS[command];
      const expected = getExpectedDriveConfirmToken(selectedDrive.device_path);
      const provided = confirmInput.trim();

      if (provided !== expected) {
        showToast({
          type: 'error',
          title: 'Token de Confirmação Inválido',
          message: `Digite exatamente "${expected}" ${descriptor.invalidTokenSuffix}`,
        });
        return;
      }

      setExecuting(true);
      try {
        const res = await invokeCommand<DriveWriteReport>(descriptor.cmd, {
          devicePath: selectedDrive.device_path,
          confirmationToken: provided,
          ...(descriptor.buildArgs ? descriptor.buildArgs(selectedDrive) : {}),
        });
        setWriteReport(res);
        showToast({
          type: 'success',
          title: descriptor.successTitle,
          message: res.message,
        });
      } catch (err) {
        showToast({
          type: 'error',
          title: descriptor.errorTitle,
          message: err instanceof Error ? err.message : String(err),
        });
      } finally {
        setExecuting(false);
      }
    },
    [selectedDrive, confirmInput, showToast]
  );

  const handleProceduralFormat = useCallback(
    () => runConfirmedOperation('format_hdd_meihdfs'),
    [runConfirmedOperation]
  );

  const handleWriteRawImage = useCallback(
    () => runConfirmedOperation('write_hdd_image'),
    [runConfirmedOperation]
  );

  const handleRestoreMirrors = useCallback(
    () => runConfirmedOperation('restore_hdd_mirrors'),
    [runConfirmedOperation]
  );

  const expectedToken = selectedDrive ? getExpectedDriveConfirmToken(selectedDrive.device_path) : '';
  const isTokenValid = Boolean(
    selectedDrive && selectedDrive.safe_to_write && confirmInput.trim() === expectedToken
  );

  return {
    drives,
    selectedDrive,
    setSelectedDrive,
    loadingDrives,
    confirmInput,
    setConfirmInput,
    executing,
    writeReport,
    expectedToken,
    isTokenValid,
    fetchDrives,
    handleProceduralFormat,
    handleWriteRawImage,
    handleRestoreMirrors,
  };
}
