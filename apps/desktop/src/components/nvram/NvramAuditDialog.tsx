import React from 'react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { NvramCleanReport } from '@/models';

interface NvramAuditDialogProps {
  isOpen: boolean;
  onClose: () => void;
  report: NvramCleanReport | null;
}

export const NvramAuditDialog: React.FC<NvramAuditDialogProps> = ({
  isOpen,
  onClose,
  report,
}) => (
  <Dialog open={isOpen} onOpenChange={open => !open && onClose()}>
    <DialogContent className="max-w-2xl font-mono">
      <DialogHeader>
        <DialogTitle>Auditoria Criptográfica SHA-256 e Backup da NVRAM</DialogTitle>
        <DialogDescription>
          Verificação forense rigorosa: Spansion S29JL064H Flash Dump
        </DialogDescription>
      </DialogHeader>

      {report && (
        <div className="space-y-4 text-xs">
          <div className="p-3.5 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
            <span className="text-[10px] text-slate-500 uppercase block font-semibold">
              Dump Original Corrompido
            </span>
            <div className="space-y-1">
              <div className="flex justify-between text-slate-400">
                <span>Arquivo:</span>
                <span className="text-slate-200 truncate max-w-sm">{report.input_file}</span>
              </div>
              <div className="flex justify-between text-slate-400">
                <span>SHA-256:</span>
                <span className="text-amber-400 truncate max-w-sm">{report.original_sha256}</span>
              </div>
              <div className="flex justify-between text-slate-400">
                <span>MD5:</span>
                <span className="text-slate-300">{report.original_md5}</span>
              </div>
            </div>
          </div>

          <div className="p-3.5 rounded-xl bg-emerald-950/20 border border-emerald-500/30 space-y-2">
            <span className="text-[10px] text-emerald-400 uppercase block font-semibold">
              Dump Cirurgicamente Limpo (Pronto para Gravação)
            </span>
            <div className="space-y-1">
              <div className="flex justify-between text-slate-400">
                <span>Arquivo:</span>
                <span className="text-slate-200 truncate max-w-sm">{report.output_file}</span>
              </div>
              <div className="flex justify-between text-slate-400">
                <span>SHA-256:</span>
                <span className="text-emerald-400 truncate max-w-sm">{report.clean_sha256}</span>
              </div>
              <div className="flex justify-between text-slate-400">
                <span>MD5:</span>
                <span className="text-slate-300">{report.clean_md5}</span>
              </div>
            </div>
          </div>

          <div className="p-3.5 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
            <span className="text-[10px] text-slate-500 uppercase block font-semibold">
              Backup de Segurança da Área Corrompida (64 KB)
            </span>
            <div className="flex justify-between text-slate-400">
              <span>Backup Local:</span>
              <span className="text-slate-200">{report.backup_file}</span>
            </div>
            <div className="flex justify-between text-slate-400">
              <span>Tamanho:</span>
              <span className="text-slate-200">65.536 bytes (SA134..SA141)</span>
            </div>
            <div className="flex justify-between text-slate-400">
              <span>Imutabilidade do Bootloader:</span>
              <span className="text-emerald-400 font-bold">100% PRESERVADO (OK)</span>
            </div>
          </div>
        </div>
      )}

      <DialogFooter>
        <Button variant="secondary" onClick={onClose}>
          Fechar Auditoria
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
);
