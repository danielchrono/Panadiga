import React from 'react';
import { Zap } from 'lucide-react';

/**
 * Pure Functional Presentation Component: Displays physical wiring guide between Pico and CN55001.
 */
export const DwirePinoutCard: React.FC = () => {
  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-3">
      <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
        <Zap className="w-4 h-4 text-amber-400" />
        <span>Conexões Físicas (Pico &rarr; CN55001)</span>
      </h3>
      <div className="space-y-1.5 text-[11px] font-mono">
        <div className="flex justify-between py-1 border-b border-slate-800">
          <span className="text-slate-400">Pino 1 (VDD 3.3V):</span>
          <span className="text-slate-200">GP26 (ADC0 / Pin 31) • Monitor de Tensão</span>
        </div>
        <div className="flex justify-between py-1 border-b border-slate-800">
          <span className="text-slate-400">Pino 2 (EXTRG):</span>
          <span className="text-slate-200">GP2 (Pin 4) • External Break Trigger</span>
        </div>
        <div className="flex justify-between py-1 border-b border-slate-800">
          <span className="text-slate-400">Pino 3 (SCLK):</span>
          <span className="text-slate-200">GP3 (Pin 5) • Synchronous Clock (1 MHz)</span>
        </div>
        <div className="flex justify-between py-1 border-b border-slate-800">
          <span className="text-slate-400">Pino 4 (SDATA):</span>
          <span className="text-slate-200">GP4 (Pin 6) • Half-Duplex Bidirecional</span>
        </div>
        <div className="flex justify-between py-1 border-b border-slate-800">
          <span className="text-slate-400">Pino 5 (nRESET):</span>
          <span className="text-slate-200">GP5 (Pin 7) • Open-Drain Target Reset</span>
        </div>
        <div className="flex justify-between py-1">
          <span className="text-slate-400">Pino 6 (GND):</span>
          <span className="text-slate-200">GND (Pin 38) • Terra de Referência Comum</span>
        </div>
      </div>
    </div>
  );
};
