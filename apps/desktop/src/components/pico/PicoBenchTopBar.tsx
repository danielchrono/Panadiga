import React, { useState } from 'react';
import { PicoDeviceStatus, DigaModelProfile } from '@/models';
import { StatusBadge } from '@/components/StatusBadge';
import { Radio, RefreshCw, Sparkles, Bluetooth, Cpu, DownloadCloud, CheckCircle2, ShieldCheck } from 'lucide-react';
import { invokeCommand } from '@/services/ipc';
import { useToast } from '@/context/ToastContext';

export interface PicoBenchTopBarProps {
  deviceStatus: PicoDeviceStatus | null;
  availablePorts: string[];
  selectedPort: string;
  modelCatalog?: DigaModelProfile[];
  selectedModel?: string;
  isErasing: boolean;
  isBackingUp?: boolean;
  onSelectPort: (port: string) => void;
  onSelectModel?: (modelId: string) => void;
  onToggleConnection: () => void;
  onInjectAndErase: () => void;
  onFirstContactBackup?: () => void;
  onRefreshPorts?: () => void;
  onRefreshStatus?: () => void;
}

interface FlashReport {
  success: boolean;
  target_drive: string;
  board_family: string;
  uf2_size_bytes: number;
  blocks_written: number;
  sha256_hash: string;
  message: string;
}

/**
 * Presentation Component: Top control bar for Pico hardware connection,
 * embedded UF2 firmware flashing, and surgical in-circuit clean.
 * Structured with 2-tier CSS Grid for zero bleeding and consistent alignment.
 */
export const PicoBenchTopBar: React.FC<PicoBenchTopBarProps> = ({
  deviceStatus,
  availablePorts,
  selectedPort,
  modelCatalog = [],
  selectedModel = 'AUTO',
  isErasing,
  isBackingUp = false,
  onSelectPort,
  onSelectModel,
  onToggleConnection,
  onInjectAndErase,
  onFirstContactBackup,
  onRefreshPorts,
  onRefreshStatus,
}) => {
  const { showToast } = useToast();
  const [isFlashing, setIsFlashing] = useState<boolean>(false);
  const isBle = selectedPort.startsWith('BLE_');

  const handleFlashFirmware = async () => {
    setIsFlashing(true);
    try {
      const drives = await invokeCommand<string[]>('pico_detect_bootsel_drives').catch(() => []);
      if (!drives || drives.length === 0) {
        showToast({
          type: 'warning',
          title: 'Nenhum Drive BOOTSEL Detectado',
          message:
            'Nenhum drive RPI-RP2 encontrado. Clique em "BOOTSEL Reset" ou segure o botão BOOTSEL na placa ao conectar o cabo USB.',
        });
        return;
      }
      const report = await invokeCommand<FlashReport>('pico_flash_firmware', {});
      const sizeStr = report.uf2_size_bytes ? report.uf2_size_bytes.toLocaleString() : '116,736';
      showToast({
        type: 'success',
        title: 'Firmware Pico Gravado',
        message: `${report.message} (${report.board_family || 'RP2040'}, ${sizeStr} bytes gravados em ${report.target_drive || 'RPI-RP2'}).`,
      });
      onRefreshPorts?.();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast({
        type: 'error',
        title: 'Falha ao Gravar Firmware',
        message: msg,
      });
    } finally {
      setIsFlashing(false);
    }
  };

  return (
    <div className="p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl space-y-4 font-mono">
      {/* Tier 1: Device Title, Status Badges & Firmware Flasher Action */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-3 border-b border-slate-800 pb-4">
        <div className="flex items-center gap-3.5">
          <div
            className={`p-3 rounded-xl border ${
              isBle
                ? 'bg-blue-500/10 text-blue-400 border-blue-500/30'
                : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'
            }`}
          >
            {isBle ? <Bluetooth className="w-6 h-6" /> : <Radio className="w-6 h-6" />}
          </div>
          <div>
            <div className="flex items-center gap-2 flex-wrap">
              <h2 className="text-base font-bold text-slate-100">
                Raspberry Pi Pico (RP2040 / RP2350) DWire32A Bus Master
              </h2>
              {deviceStatus?.connected ? (
                deviceStatus.port.startsWith('BLE_') ? (
                  <StatusBadge variant="info" label="BLE CONECTADO" />
                ) : (
                  <StatusBadge variant="success" label="CONECTADO" />
                )
              ) : (
                <StatusBadge variant="error" label="DESCONECTADO" />
              )}
              {deviceStatus?.connected && onRefreshStatus && (
                <button
                  type="button"
                  onClick={onRefreshStatus}
                  title="Atualizar telemetria e verificar status do dispositivo"
                  className="p-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 transition-colors"
                >
                  <RefreshCw className="w-3 h-3" />
                </button>
              )}
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Matsushita AM34 / UniPhier Single-Wire OCD Host Controller via PIO0 State Machine
            </p>
          </div>
        </div>

        {/* Action: Flash Embedded UF2 Firmware directly */}
        <div className="flex items-center gap-2 shrink-0">
          <button
            onClick={handleFlashFirmware}
            disabled={isFlashing}
            title="Grava o firmware panasonic_dwire32a.uf2 embutido no drive BOOTSEL (RPI-RP2)"
            className="px-3.5 py-1.5 rounded-xl bg-purple-500/10 hover:bg-purple-500/20 text-purple-300 border border-purple-500/30 font-bold text-xs flex items-center gap-1.5 transition-colors disabled:opacity-50"
          >
            {isFlashing ? (
              <>
                <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                <span>Gravando UF2...</span>
              </>
            ) : (
              <>
                <DownloadCloud className="w-3.5 h-3.5" />
                <span>Flash Pico (BOOTSEL)</span>
              </>
            )}
          </button>
          {/* New Action: Reboot to BOOTSEL via 1200‑baud touch */}
          <button
            onClick={async () => {
              if (!selectedPort) {
                showToast({
                  type: 'warning',
                  title: 'Porta Não Selecionada',
                  message: 'Selecione a porta serial do Pico antes de solicitar o reset para BOOTSEL.',
                });
                return;
              }
              try {
                const msg = await invokeCommand<string>('pico_trigger_bootsel_reboot', {
                  portName: selectedPort,
                  port_name: selectedPort,
                });
                showToast({
                  type: 'success',
                  title: 'BOOTSEL Reset Iniciado',
                  message: `${msg}. Aguarde 2 segundos para o drive RPI-RP2 ser montado e clique em "Flash Pico".`,
                });
              } catch (err) {
                const msg = err instanceof Error ? err.message : String(err);
                showToast({
                  type: 'error',
                  title: 'Falha no Reset BOOTSEL',
                  message: msg,
                });
              }
            }}
            disabled={isBle || !/VID:2E8A|RP2040|RP2350|Pico|DWire32A/i.test(selectedPort ?? '')}
            title="Reinicia o Pico em modo BOOTSEL usando a sequência 1200‑baud (sem botão físico). Só habilitado para portas do Pico (VID:2E8A)."
            className="px-3.5 py-1.5 rounded-xl bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 font-bold text-xs flex items-center gap-1.5 transition-colors disabled:opacity-50"
          >
            <RefreshCw className="w-3.5 h-3.5" />
            <span>BOOTSEL Reset</span>
          </button>
        </div>
      </div>

      {/* Tier 2: 5-Column Controls Grid (Balanced Proportions, No Bleeding) */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3 items-end">
        {/* Col 1: Model Selector */}
        <div className="flex flex-col gap-1">
          <label className="text-[10px] text-slate-500 uppercase font-semibold">
            Modelo Panasonic DIGA
          </label>
          <select
            aria-label="Selecionar modelo Panasonic DIGA"
            value={selectedModel}
            onChange={e => onSelectModel?.(e.target.value)}
            className="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-xs text-cyan-300 focus:outline-none focus:border-cyan-500 font-mono truncate"
          >
            {modelCatalog.map(m => (
              <option key={m.model_id} value={m.model_id}>
                {m.model_id === 'AUTO'
                  ? 'AUTO (Detectar)'
                  : `${m.model_id} (${m.flash_size_mb}MB)`}
              </option>
            ))}
          </select>
        </div>

        {/* Col 2: Serial Port Selector */}
        <div className="flex flex-col gap-1">
          <div className="flex items-center justify-between">
            <label className="text-[10px] text-slate-500 uppercase font-semibold">
              Porta Serial / Interface
            </label>
            {onRefreshPorts && (
              <button
                type="button"
                onClick={onRefreshPorts}
                title="Escanear portas COM USB"
                className="text-[10px] text-cyan-400 hover:text-cyan-300 flex items-center gap-1 transition-colors focus:outline-none"
              >
                <RefreshCw className="w-2.5 h-2.5" />
                <span>Atualizar</span>
              </button>
            )}
          </div>
          <select
            aria-label="Selecionar porta serial do Pico"
            value={selectedPort}
            onChange={e => onSelectPort(e.target.value)}
            className="w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-200 focus:outline-none focus:border-cyan-500 font-mono truncate"
          >
            {availablePorts.length === 0 ? (
              <option value="">Nenhuma porta serial detectada</option>
            ) : (
              availablePorts.map(p => (
                <option key={p} value={p}>
                  {p}
                </option>
              ))
            )}
          </select>
        </div>

        {/* Col 3: Connect / Disconnect Action */}
        <div className="flex flex-col gap-1 justify-end">
          <span className="text-[10px] text-transparent select-none">Conexão</span>
          <button
            onClick={onToggleConnection}
            className={`w-full py-2 px-3 rounded-xl font-bold text-xs transition-colors flex items-center justify-center gap-1.5 ${
              deviceStatus?.connected
                ? 'bg-slate-800 hover:bg-slate-750 text-slate-300 border border-slate-700'
                : isBle
                  ? 'bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-950/40'
                  : 'bg-cyan-500 hover:bg-cyan-400 text-slate-950 shadow-lg shadow-cyan-950/40'
            }`}
          >
            <Radio className="w-3.5 h-3.5" />
            <span>
              {deviceStatus?.connected
                ? 'Desconectar'
                : isBle
                  ? 'Conectar BLE'
                  : 'Conectar Serial'}
            </span>
          </button>
        </div>

        {/* Col 4: First-Contact Read-Only Backup Action */}
        <div className="flex flex-col gap-1 justify-end">
          <span className="text-[10px] text-cyan-400 font-semibold uppercase">1º Contato</span>
          <button
            onClick={onFirstContactBackup}
            disabled={isBackingUp || !deviceStatus?.connected}
            title="Backup não-destrutivo de NVRAM, SRAM e IPL com verificação de integridade"
            className="w-full py-2 px-3 rounded-xl bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-300 border border-cyan-500/30 font-bold text-xs flex items-center justify-center gap-1.5 transition-all disabled:opacity-50"
          >
            {isBackingUp ? (
              <>
                <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                <span>Lendo NVRAM...</span>
              </>
            ) : (
              <>
                <ShieldCheck className="w-3.5 h-3.5" />
                <span>Backup 1º Contato</span>
              </>
            )}
          </button>
        </div>

        {/* Col 5: Inject RAM Loader & Erase Action */}
        <div className="flex flex-col gap-1 justify-end">
          <span className="text-[10px] text-emerald-400 font-semibold uppercase">Higienização</span>
          <button
            onClick={onInjectAndErase}
            disabled={isErasing || !deviceStatus?.connected}
            className="w-full py-2 px-3 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-slate-950 font-bold text-xs flex items-center justify-center gap-1.5 shadow-xl shadow-emerald-950/50 transition-all disabled:opacity-50"
          >
            {isErasing ? (
              <>
                <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                <span>Injetando...</span>
              </>
            ) : (
              <>
                <Sparkles className="w-3.5 h-3.5" />
                <span>Injetar & Limpar</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
