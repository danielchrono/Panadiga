import React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge } from '@/components/ui';
import { StatusBadge } from '@/components/StatusBadge';
import { useTranslation } from '@/i18n/useTranslation';
import type { BridgeStreamLine } from '@/viewmodels/useSc0Bridge';
import type { Sc0BridgeInputMode, Sc0BridgeStatus } from '@/models';
import { Activity, Disc, Play, Radio, Send, Square, Terminal, Trash2, Zap } from 'lucide-react';

export interface Sc0BridgePanelProps {
  readonly status: Sc0BridgeStatus;
  readonly logs: readonly BridgeStreamLine[];
  readonly baudRate: string;
  readonly onChangeBaudRate: (baud: string) => void;
  readonly bootMode: string;
  readonly onChangeBootMode: (mode: string) => void;
  readonly inputMode: Sc0BridgeInputMode;
  readonly onChangeInputMode: (mode: Sc0BridgeInputMode) => void;
  readonly txInput: string;
  readonly onChangeTxInput: (value: string) => void;
  readonly isBusy: boolean;
  readonly error: string | null;
  readonly onStart: () => void;
  readonly onStop: () => void;
  readonly onSubmitTx: () => void;
  readonly onClearLogs: () => void;
  readonly onTriggerWaittx: (spec: string) => void;
  readonly onSendRspPreset: (
    mode: Sc0BridgeInputMode,
    bridgePayload: string,
    panarspArg: string
  ) => void;
  readonly onSendDriveCommand: (cmd: string) => void;
}

const RSP_QUICK_PROBES: ReadonlyArray<{
  readonly label: string;
  readonly mode: Sc0BridgeInputMode;
  readonly bridgePayload: string;
  readonly panarspArg: string;
}> = [
  { label: '0x15 NAK', mode: 'hex', bridgePayload: '15', panarspArg: '4000 JIG:350 NAK' },
  { label: 'Magic 0E0402', mode: 'hex', bridgePayload: '0E0402', panarspArg: '4000 JIG:350 MAGIC NONAK' },
  { label: '$g#67 Regs', mode: 'rsp', bridgePayload: 'g', panarspArg: '6000 JIG:350 G' },
  { label: '$Z0#8A Exit', mode: 'rsp', bridgePayload: 'Z0', panarspArg: '4000 Z0' },
  { label: '$Z1#8B', mode: 'rsp', bridgePayload: 'Z1', panarspArg: '4000 Z1' },
];

const DRIVE_QUICK_COMMANDS: ReadonlyArray<string> = [
  'PASS',
  'START',
  'LDD',
  'DSCREG',
  'FEP',
  'PON',
  'STOP',
];

export const Sc0BridgePanel: React.FC<Sc0BridgePanelProps> = ({
  status,
  logs,
  baudRate,
  onChangeBaudRate,
  bootMode,
  onChangeBootMode,
  inputMode,
  onChangeInputMode,
  txInput,
  onChangeTxInput,
  isBusy,
  error,
  onStart,
  onStop,
  onSubmitTx,
  onClearLogs,
  onTriggerWaittx,
  onSendRspPreset,
  onSendDriveCommand,
}) => {
  const { t } = useTranslation();

  return (
    <Card className="border-slate-800 bg-bench-900/90 shadow-xl">
      <CardHeader className="pb-3 border-b border-slate-800/80 flex flex-col lg:flex-row lg:items-center lg:justify-between gap-3">
        <div className="space-y-1">
          <div className="flex items-center gap-2 flex-wrap">
            <Radio className="w-4 h-4 text-cyan-400" />
            <CardTitle className="text-sm font-bold text-slate-100">
              {t('pico.bridge.title')}
            </CardTitle>
            <StatusBadge
              variant={status.running ? 'success' : 'neutral'}
              label={status.running ? t('pico.bridge.active') : t('pico.bridge.standby')}
            />
            <Badge variant="outline" className="text-[10px]">
              {t('pico.bridge.bytesRx', { count: status.bytes_rx })} |{' '}
              {t('pico.bridge.bytesTx', { count: status.bytes_tx })}
            </Badge>
          </div>
          <p className="text-xs text-slate-400 font-sans">{t('pico.bridge.subtitle')}</p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <label className="flex items-center gap-1.5 text-[11px] text-slate-400">
            <span>{t('pico.bridge.baudLabel')}</span>
            <select
              aria-label={t('pico.bridge.baudLabel')}
              value={baudRate}
              disabled={status.running || isBusy}
              onChange={e => onChangeBaudRate(e.target.value)}
              className="rounded-lg bg-slate-950 border border-slate-700 px-2 py-1 text-xs text-slate-100"
            >
              <option value="115200">115200</option>
              <option value="57600">57600</option>
              <option value="38400">38400</option>
              <option value="9600">9600</option>
            </select>
          </label>

          <label className="flex items-center gap-1.5 text-[11px] text-slate-400">
            <span>{t('pico.bridge.bootLabel')}</span>
            <select
              aria-label={t('pico.bridge.bootLabel')}
              value={bootMode}
              disabled={status.running || isBusy}
              onChange={e => onChangeBootMode(e.target.value)}
              className="rounded-lg bg-slate-950 border border-slate-700 px-2 py-1 text-xs text-slate-100"
            >
              <option value="JIG:350">JIG:350</option>
              <option value="NORMAL">NORMAL</option>
              <option value="NOPULL">NOPULL</option>
              <option value="PASSIVE">{t('pico.bridge.passiveOption')}</option>
            </select>
          </label>

          {status.running ? (
            <Button
              type="button"
              variant="destructive"
              size="sm"
              disabled={isBusy}
              onClick={onStop}
              className="gap-1.5 text-xs font-bold"
            >
              <Square className="w-3.5 h-3.5" />
              <span>{t('pico.bridge.stop')}</span>
            </Button>
          ) : (
            <Button
              type="button"
              variant="default"
              size="sm"
              disabled={isBusy}
              onClick={onStart}
              className="gap-1.5 text-xs font-bold"
            >
              <Play className="w-3.5 h-3.5" />
              <span>{t('pico.bridge.start')}</span>
            </Button>
          )}
        </div>
      </CardHeader>

      <CardContent className="pt-4 space-y-4">
        {error && (
          <div
            role="alert"
            className="rounded-lg border border-rose-500/40 bg-rose-500/10 px-3 py-2 text-xs text-rose-300"
          >
            {error}
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
          <div className="rounded-xl border border-slate-800 bg-slate-950/70 p-3 space-y-2">
            <div className="flex items-center justify-between gap-2">
              <span className="text-[11px] font-bold text-cyan-300 flex items-center gap-1.5">
                <Activity className="w-3.5 h-3.5" />
                <span>{t('pico.bridge.rspSection')}</span>
              </span>
              <div className="flex gap-1.5">
                <Button
                  type="button"
                  size="sm"
                  variant="secondary"
                  disabled={status.running || isBusy}
                  onClick={() => onTriggerWaittx('8000 JIG:350 "wait 1" 0D 4000')}
                  className="text-[10px] h-7 px-2 gap-1"
                >
                  <Zap className="w-3 h-3" />
                  <span>{t('pico.bridge.waittxWait1')}</span>
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="secondary"
                  disabled={status.running || isBusy}
                  onClick={() => onTriggerWaittx('20000 JIG:350 "Hit Any Key" 1B0D 6000')}
                  className="text-[10px] h-7 px-2 gap-1"
                >
                  <Zap className="w-3 h-3" />
                  <span>{t('pico.bridge.waittxHitKey')}</span>
                </Button>
              </div>
            </div>

            <div className="flex flex-wrap gap-1.5">
              {RSP_QUICK_PROBES.map(probe => (
                <Button
                  key={probe.label}
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={isBusy}
                  onClick={() => onSendRspPreset(probe.mode, probe.bridgePayload, probe.panarspArg)}
                  className="text-[11px] h-7 px-2.5 font-mono"
                >
                  {probe.label}
                </Button>
              ))}
            </div>
          </div>

          <div className="rounded-xl border border-slate-800 bg-slate-950/70 p-3 space-y-2">
            <span className="text-[11px] font-bold text-amber-300 flex items-center gap-1.5">
              <Disc className="w-3.5 h-3.5" />
              <span>{t('pico.bridge.driveSection')}</span>
            </span>
            <div className="flex flex-wrap gap-1.5">
              {DRIVE_QUICK_COMMANDS.map(cmd => (
                <Button
                  key={cmd}
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={isBusy}
                  onClick={() => onSendDriveCommand(cmd)}
                  className="text-[11px] h-7 px-2.5 font-mono"
                >
                  {cmd}
                </Button>
              ))}
            </div>
          </div>
        </div>

        <div className="rounded-xl border border-slate-800 bg-slate-950 p-3 space-y-2">
          <div className="flex items-center justify-between text-[11px] text-slate-400">
            <span className="flex items-center gap-1.5">
              <Terminal className="w-3.5 h-3.5 text-cyan-400" />
              <span>{status.last_event}</span>
            </span>
            <button
              type="button"
              onClick={onClearLogs}
              className="flex items-center gap-1 text-[11px] text-slate-400 hover:text-slate-200 px-2 py-0.5 rounded hover:bg-slate-800"
            >
              <Trash2 className="w-3 h-3" />
              <span>{t('pico.bridge.clearStream')}</span>
            </button>
          </div>

          <div
            role="log"
            aria-label={t('pico.bridge.title')}
            className="h-36 overflow-y-auto rounded-lg bg-slate-900/60 border border-slate-800/80 p-2 space-y-1 font-mono text-[11px]"
          >
            {logs.length === 0 ? (
              <div className="h-full flex items-center justify-center text-slate-500 text-center px-4">
                {t('pico.bridge.emptyStream')}
              </div>
            ) : (
              logs.map(entry => (
                <div key={entry.id} className="flex items-start gap-2 leading-snug">
                  <span className="text-slate-500 shrink-0">{entry.timestamp}</span>
                  <span
                    className={
                      entry.stream === 'tx'
                        ? 'text-amber-300 shrink-0 font-bold'
                        : entry.stream === 'sys'
                        ? 'text-emerald-400 shrink-0 font-bold'
                        : 'text-cyan-400 shrink-0 font-bold'
                    }
                  >
                    [{entry.stream.toUpperCase()}]
                  </span>
                  <span className="text-slate-200 break-all">{entry.text}</span>
                  {entry.hex && <span className="text-slate-500 shrink-0">[{entry.hex}]</span>}
                </div>
              ))
            )}
          </div>

          <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 pt-1">
            <select
              aria-label={t('pico.bridge.inputModeLabel')}
              value={inputMode}
              onChange={e => onChangeInputMode(e.target.value as Sc0BridgeInputMode)}
              className="rounded-lg bg-slate-900 border border-slate-700 px-2.5 py-1.5 text-xs text-slate-100"
            >
              <option value="line">{t('pico.bridge.modeLine')}</option>
              <option value="hex">{t('pico.bridge.modeHex')}</option>
              <option value="rsp">{t('pico.bridge.modeRsp')}</option>
              <option value="text">{t('pico.bridge.modeText')}</option>
            </select>

            <input
              type="text"
              aria-label={t('pico.bridge.inputPlaceholder')}
              placeholder={t('pico.bridge.inputPlaceholder')}
              value={txInput}
              disabled={!status.running || isBusy}
              onChange={e => onChangeTxInput(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && onSubmitTx()}
              className="flex-1 rounded-lg bg-slate-900 border border-slate-700 px-3 py-1.5 text-xs text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-cyan-500"
            />

            <Button
              type="button"
              size="sm"
              disabled={!status.running || isBusy || !txInput.trim()}
              onClick={onSubmitTx}
              className="gap-1.5 text-xs font-bold"
            >
              <Send className="w-3.5 h-3.5" />
              <span>{t('pico.bridge.send')}</span>
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
};
