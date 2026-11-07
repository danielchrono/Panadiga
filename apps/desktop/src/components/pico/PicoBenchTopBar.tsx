import React from 'react';
import { PicoDeviceStatus } from '@/models';
import { StatusBadge } from '@/components/StatusBadge';
import { Radio, RefreshCw, Sparkles } from 'lucide-react';

export interface PicoBenchTopBarProps {
  deviceStatus: PicoDeviceStatus | null;
  availablePorts: string[];
  selectedPort: string;
  isErasing: boolean;
  onSelectPort: (port: string) => void;
  onToggleConnection: () => void;
  onInjectAndErase: () => void;
}

/**
 * Presentation Component: Top control bar for Pico hardware connection and surgical in-circuit clean.
 */
export const PicoBenchTopBar: React.FC<PicoBenchTopBarProps> = ({
  deviceStatus,
  availablePorts,
  selectedPort,
  isErasing,
  onSelectPort,
  onToggleConnection,
  onInjectAndErase,
}) => {
  return (
    <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
      <div className="flex items-center gap-3.5">
        <div className="p-3 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
          <Radio className="w-6 h-6" />
        </div>
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-base font-bold text-slate-100">
              Raspberry Pi Pico (RP2040 / RP2350) DWire32A Bus Master
            </h2>
            {deviceStatus?.connected ? (
              <StatusBadge variant="success" label="CONECTADO" />
            ) : (
              <StatusBadge variant="error" label="DESCONECTADO" />
            )}
          </div>
          <p className="text-xs text-slate-400 mt-0.5">
            Matsushita AM34 / UniPhier Single-Wire OCD Host Controller via PIO0 State Machine
          </p>
        </div>
      </div>

      {/* Port Picker & In-Circuit Action */}
      <div className="flex flex-wrap items-center gap-3 shrink-0">
        <select
          value={selectedPort}
          onChange={e => onSelectPort(e.target.value)}
          className="px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-200 focus:outline-none focus:border-cyan-500 font-mono"
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

        <button
          onClick={onToggleConnection}
          className={`px-4 py-2 rounded-xl font-bold text-xs transition-colors ${
            deviceStatus?.connected
              ? 'bg-slate-800 hover:bg-slate-750 text-slate-300 border border-slate-700'
              : 'bg-cyan-500 hover:bg-cyan-400 text-slate-950 shadow-lg shadow-cyan-950/40'
          }`}
        >
          {deviceStatus?.connected ? 'Desconectar' : 'Conectar Serial'}
        </button>

        <button
          onClick={onInjectAndErase}
          disabled={isErasing || !deviceStatus?.connected}
          className="px-5 py-2 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-slate-950 font-bold text-xs flex items-center gap-2 shadow-xl shadow-emerald-950/50 transition-all disabled:opacity-50"
        >
          {isErasing ? (
            <>
              <RefreshCw className="w-4 h-4 animate-spin" />
              <span>Injetando no AM34...</span>
            </>
          ) : (
            <>
              <Sparkles className="w-4 h-4" />
              <span>Injetar RAM Loader & Limpar NVRAM</span>
            </>
          )}
        </button>
      </div>
    </div>
  );
};
