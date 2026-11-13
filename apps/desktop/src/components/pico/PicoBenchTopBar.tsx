import React, { useState } from 'react';
import type {
  DigaModelProfile,
  PicoDeviceStatus,
  PicoTransportMode,
  PicoUf2Variant,
} from '@/models';
import { isTargetPowered } from '@/utils/formatters';
import { StatusBadge } from '@/components/StatusBadge';
import { Alert, AlertDescription, AlertTitle, Button } from '@/components/ui';
import {
  Radio,
  RefreshCw,
  Sparkles,
  Bluetooth,
  DownloadCloud,
  ShieldCheck,
  AlertTriangle,
  Usb,
} from 'lucide-react';
import { invokeCommand } from '@/services/ipc';
import { useToast } from '@/context/ToastContext';

export interface PicoBenchTopBarProps {
  deviceStatus: PicoDeviceStatus | null;
  availablePorts: string[];
  selectedPort: string;
  transportMode?: PicoTransportMode;
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
  onQuickPincheck?: () => void;
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

const SELECT_CLASS =
  'w-full px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-200 focus:outline-none focus:border-cyan-500 font-mono truncate';

export const PicoBenchTopBar: React.FC<PicoBenchTopBarProps> = ({
  deviceStatus,
  availablePorts,
  selectedPort,
  transportMode = 'usb',
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
  onQuickPincheck,
}) => {
  const { showToast } = useToast();
  const [isFlashing, setIsFlashing] = useState<boolean>(false);
  const [uf2Variant, setUf2Variant] = useState<PicoUf2Variant>('unified');

  const isBle = transportMode === 'ble' || selectedPort.startsWith('BLE_');
  const isOtg = transportMode === 'usb-otg' || selectedPort.startsWith('ANDROID-USB-OTG');
  const isPicoPort = /VID:2E8A|RP2040|RP2350|Pico|PANADIGA|P7501/i.test(selectedPort ?? '');
  const powered = isTargetPowered(deviceStatus?.vdd_volts);

  const handleBootselReset = async () => {
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
        message: `${msg}. Aguarde 2 segundos para o drive RPI-RP2 / RP2350 montar e clique em "Flash Pico (BOOTSEL)".`,
      });
    } catch (err) {
      showToast({
        type: 'error',
        title: 'Falha no Reset BOOTSEL',
        message: err instanceof Error ? err.message : String(err),
      });
    }
  };

  const handleFlashFirmware = async () => {
    setIsFlashing(true);
    try {
      let drives = await invokeCommand<string[]>('pico_detect_bootsel_drives').catch(() => []);
      if ((!drives || drives.length === 0) && selectedPort && isPicoPort) {
        showToast({
          type: 'info',
          title: 'Reiniciando Pico em BOOTSEL',
          message: 'Enviando pulso 1200-baud para reiniciar a placa em modo de gravação...',
        });
        await invokeCommand('pico_trigger_bootsel_reboot', {
          portName: selectedPort,
          port_name: selectedPort,
        }).catch(() => {});
        await new Promise(r => setTimeout(r, 1600));
        drives = await invokeCommand<string[]>('pico_detect_bootsel_drives').catch(() => []);
      }

      if (!drives || drives.length === 0) {
        showToast({
          type: 'warning',
          title: 'Nenhum Drive BOOTSEL Detectado',
          message:
            'Nenhum drive RPI-RP2 / RP2350 encontrado. Clique em "BOOTSEL Reset" ou segure BOOTSEL ao conectar.',
        });
        return;
      }

      const report = await invokeCommand<FlashReport>('pico_flash_firmware', {
        targetFamily: uf2Variant,
        target_family: uf2Variant,
      });
      const sizeStr = report.uf2_size_bytes ? report.uf2_size_bytes.toLocaleString() : 'UF2';
      showToast({
        type: 'success',
        title: 'Firmware Pico Gravado',
        message: `${report.message} (${report.board_family || uf2Variant}, ${sizeStr} bytes). Reconectando USB...`,
      });

      let detectedPort = '';
      for (let i = 0; i < 12; i++) {
        await new Promise(r => setTimeout(r, 500));
        const ports = await invokeCommand<string[]>('list_pico_serial_ports').catch(() => []);
        const found = ports?.find(p => /VID:2E8A|RP2040|RP2350|Pico|PANADIGA|P7501/i.test(p));
        if (found) {
          detectedPort = found;
          break;
        }
      }

      onRefreshPorts?.();
      if (detectedPort) {
        onSelectPort(detectedPort);
        showToast({
          type: 'success',
          title: 'Pico Detectado',
          message: `Reconectando automaticamente em ${detectedPort}...`,
        });
        setTimeout(() => onToggleConnection(), 400);
      }
    } catch (err) {
      showToast({
        type: 'error',
        title: 'Falha ao Gravar Firmware',
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setIsFlashing(false);
    }
  };

  return (
    <div className="p-4 sm:p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl space-y-4 font-mono">
      <div className="flex flex-col xl:flex-row items-start xl:items-center justify-between gap-3 border-b border-slate-800 pb-4">
        <div className="flex items-center gap-3.5 min-w-0">
          <div
            className={`p-3 rounded-xl border shrink-0 ${
              isBle
                ? 'bg-blue-500/10 text-blue-400 border-blue-500/30'
                : isOtg
                  ? 'bg-cyan-500/10 text-cyan-400 border-cyan-500/30'
                  : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'
            }`}
          >
            {isBle ? (
              <Bluetooth className="w-6 h-6" />
            ) : isOtg ? (
              <Usb className="w-6 h-6" />
            ) : (
              <Radio className="w-6 h-6" />
            )}
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <h2 className="text-sm sm:text-base font-bold text-slate-100">
                Raspberry Pi Pico (RP2040 / RP2350) — P7501 UART (115200 8N1)
              </h2>
              {deviceStatus?.connected ? (
                isBle ? (
                  <StatusBadge variant="info" label="BLE CONECTADO" />
                ) : isOtg ? (
                  <StatusBadge variant="info" label="USB-OTG CONECTADO" />
                ) : (
                  <StatusBadge variant="success" label="CONECTADO" />
                )
              ) : (
                <StatusBadge variant="error" label="DESCONECTADO" />
              )}
              {deviceStatus?.connected && onRefreshStatus && (
                <Button
                  type="button"
                  variant="outline"
                  size="icon"
                  onClick={onRefreshStatus}
                  title="Atualizar telemetria e verificar status do dispositivo"
                  className="h-7 w-7"
                >
                  <RefreshCw className="w-3 h-3" />
                </Button>
              )}
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Interface Primária P7501 UART 115200 8N1 &amp; Controle Discreto SW_UART / XRESET / NSW5V
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 shrink-0 flex-wrap">
          <select
            aria-label="Variante UF2"
            value={uf2Variant}
            onChange={e => setUf2Variant(e.target.value as PicoUf2Variant)}
            disabled={isFlashing || isOtg || isBle}
            className="h-8 px-2.5 rounded-lg bg-slate-950 border border-slate-800 text-[11px] text-purple-300 font-mono focus:outline-none focus:border-purple-500"
          >
            <option value="unified">UF2 Unificado (RP2040+RP2350)</option>
            <option value="rp2040">UF2 RP2040 (Pico 1)</option>
            <option value="rp2350">UF2 RP2350 (Pico 2)</option>
          </select>

          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleFlashFirmware}
            disabled={isFlashing || isOtg}
            title="Grava o firmware P7501 Async UART (RP2040 / RP2350) no drive BOOTSEL (RPI-RP2 / RP2350)"
            className="bg-purple-500/10 hover:bg-purple-500/20 text-purple-300 border-purple-500/30 font-bold gap-1.5"
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
          </Button>

          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleBootselReset}
            disabled={isBle || isOtg || !isPicoPort}
            title="Reinicia o Pico em modo BOOTSEL via 1200-baud touch / comando CDC"
            className="bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-300 border-indigo-500/30 font-bold gap-1.5"
          >
            <RefreshCw className="w-3.5 h-3.5" />
            <span>BOOTSEL Reset</span>
          </Button>
        </div>
      </div>

      {deviceStatus?.connected && !powered && (
        <Alert variant="warning" className="items-center justify-between flex-wrap">
          <div className="flex items-start gap-2.5">
            <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
            <div>
              <AlertTitle>
                Sonda Pico Operacional, mas Trilho Alvo em {deviceStatus.vdd_volts.toFixed(2)}V (&lt; 2.50V)
              </AlertTitle>
              <AlertDescription>
                Verifique se o Panasonic DMR-EH55 está energizado na bancada e se os pinos P7501 (Pino 4 GND, Pino 1 SW_UART / ADC0) estão conectados corretamente.
              </AlertDescription>
            </div>
          </div>
          {onQuickPincheck && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={onQuickPincheck}
              className="border-amber-500/40 bg-amber-500/15 text-amber-200 hover:bg-amber-500/25 shrink-0"
            >
              Executar PINCHECK
            </Button>
          )}
        </Alert>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3 items-end">
        <div className="flex flex-col gap-1">
          <label className="text-[10px] text-slate-500 uppercase font-semibold">
            Modelo Panasonic DIGA
          </label>
          <select
            aria-label="Selecionar modelo Panasonic DIGA"
            value={selectedModel}
            onChange={e => onSelectModel?.(e.target.value)}
            className={`${SELECT_CLASS} text-cyan-300`}
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

        <div className="flex flex-col gap-1">
          <div className="flex items-center justify-between">
            <label className="text-[10px] text-slate-500 uppercase font-semibold">
              Porta Serial / Interface
            </label>
            {onRefreshPorts && (
              <button
                type="button"
                onClick={onRefreshPorts}
                title="Escanear portas COM USB / OTG"
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
            className={SELECT_CLASS}
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

        <div className="flex flex-col gap-1 justify-end">
          <span className="text-[10px] text-transparent select-none">Conexão</span>
          <Button
            type="button"
            variant={deviceStatus?.connected ? 'secondary' : 'default'}
            onClick={onToggleConnection}
            className={`w-full font-bold text-xs gap-1.5 ${
              !deviceStatus?.connected && isBle ? 'bg-blue-600 hover:bg-blue-500 text-white' : ''
            }`}
          >
            <Radio className="w-3.5 h-3.5" />
            <span>
              {deviceStatus?.connected
                ? 'Desconectar'
                : isBle
                  ? 'Conectar BLE'
                  : isOtg
                    ? 'Conectar USB-OTG'
                    : 'Conectar Serial'}
            </span>
          </Button>
        </div>

        <div className="flex flex-col gap-1 justify-end">
          <span className="text-[10px] text-cyan-400 font-semibold uppercase">1º Contato</span>
          <Button
            type="button"
            variant="outline"
            onClick={onFirstContactBackup}
            disabled={isBackingUp || !deviceStatus?.connected}
            title="Backup não-destrutivo de NVRAM, SRAM e IPL com verificação de integridade"
            className="w-full bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-300 border-cyan-500/30 font-bold text-xs gap-1.5"
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
          </Button>
        </div>

        <div className="flex flex-col gap-1 justify-end">
          <span className="text-[10px] text-emerald-400 font-semibold uppercase">Higienização</span>
          <Button
            type="button"
            variant="emerald"
            onClick={onInjectAndErase}
            disabled={isErasing || !deviceStatus?.connected}
            className="w-full font-bold text-xs gap-1.5"
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
          </Button>
        </div>
      </div>
    </div>
  );
};
