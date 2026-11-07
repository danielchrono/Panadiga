import React from 'react';
import { Zap, AlertTriangle } from 'lucide-react';

/**
 * Pure Functional Presentation Component: Displays physical wiring guide between Pico and CN55001 / P7501.
 *
 * CRITICAL BENCH SAFETY INVARIANT:
 * - Panasonic Official Service Manual (P7501): GND is Pin 4.
 * - Physical PCB orientation (counting from 10-way flat FP50003 to 8-way flat FP50004): GND is Pin 3.
 * - The canonical reference is the official manual (Pin 4).
 */
export const DwirePinoutCard: React.FC = () => {
  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
      <div className="flex items-center justify-between border-b border-slate-800 pb-3">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
          <Zap className="w-4 h-4 text-amber-400" />
          <span>Conexões Físicas (Pico → CN55001)</span>
        </h3>
        <span className="text-[10px] font-mono text-emerald-400 bg-emerald-950/60 border border-emerald-800/80 px-2 py-0.5 rounded">
          P7501 [B6B-ZR-3R4]
        </span>
      </div>

      {/* Critical Safety Alert Box */}
      <div className="p-3 bg-amber-950/30 border border-amber-800/60 rounded-lg text-xs space-y-1.5">
        <div className="flex items-center gap-1.5 text-amber-400 font-bold text-[11px]">
          <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0" />
          <span>DISCLAIMER CRÍTICO: IDENTIFICAÇÃO DO TERRA (GND)</span>
        </div>
        <p className="text-slate-300 text-[11px] leading-relaxed">
          • <strong>Manual Oficial (P7501):</strong> O GND é estritamente o <strong>PINO 4</strong>.<br />
          • <strong>Contagem Física (10V → 8V):</strong> O GND corresponde ao <strong>PINO 3</strong> (0.0Ω com chassi).<br />
          • <span className="text-amber-300 font-medium">O correto do projeto é a referência do manual (Pino 4).</span> Valide sempre o bipe de 0.0Ω no multímetro antes de energizar!
        </p>
      </div>

      {/* Dual Pinout Table */}
      <div className="space-y-1.5 text-[11px] font-mono">
        <div className="flex justify-between py-1 border-b border-slate-800/60">
          <span className="text-rose-400">Pino 1 (Manual) / Pino 6 (Físico) [VDD 3.3V]:</span>
          <span className="text-slate-200">GP26 (ADC0 / Pin 31) • Monitor 3.3V</span>
        </div>
        <div className="flex justify-between py-1 border-b border-slate-800/60">
          <span className="text-purple-400">Pino 2 (Manual) / Pino 5 (Físico) [nRESET]:</span>
          <span className="text-slate-200">GP5 (Pin 7) • Open-Drain (Nunca Push-Pull)</span>
        </div>
        <div className="flex justify-between py-1 border-b border-slate-800/60">
          <span className="text-sky-400">Pino 3 (Manual) / Pino 4 (Físico) [SDATA]:</span>
          <span className="text-slate-200">GP4 (Pin 6) • Half-Duplex (100Ω + Pull-up 4.7k)</span>
        </div>
        <div className="flex justify-between py-1 border-b border-emerald-800/60 bg-emerald-950/20 px-1 rounded">
          <span className="text-emerald-400 font-bold">Pino 4 (Manual) / Pino 3 (Físico) [GND]:</span>
          <span className="text-emerald-300 font-bold">GND (Pin 38) • Terra 0.0Ω Chassi</span>
        </div>
        <div className="flex justify-between py-1 border-b border-slate-800/60">
          <span className="text-sky-400">Pino 5 (Manual) / Pino 2 (Físico) [SCLK]:</span>
          <span className="text-slate-200">GP3 (Pin 5) • Clock Master 1.0 MHz</span>
        </div>
        <div className="flex justify-between py-1">
          <span className="text-sky-400">Pino 6 (Manual) / Pino 1 (Físico) [EXTRG]:</span>
          <span className="text-slate-200">GP2 (Pin 4) • External Break Trigger</span>
        </div>
      </div>
    </div>
  );
};
