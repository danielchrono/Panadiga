import React from 'react';
import { Zap, AlertTriangle } from 'lucide-react';
import { Badge, Card } from '@/components/ui';
import type { HardwareSsotSpec } from '@/models';

export interface P7501PinoutCardProps {
  ssot: HardwareSsotSpec;
}

const PIN_COLOR_MAP: Record<number, string> = {
  1: 'text-rose-400',
  2: 'text-purple-400',
  4: 'text-emerald-400 font-bold',
};

/**
 * Pure Functional Presentation Component: Displays physical wiring guide between Pico and DMR-EH55 P7501 (B6B-ZR-3R4)
 * driven directly from Rust SSOT (`ssot.pinout`).
 */
export const P7501PinoutCard: React.FC<P7501PinoutCardProps> = ({ ssot }) => {
  return (
    <Card className="p-4 shadow-xl space-y-3 flex flex-col justify-between overflow-hidden">
      <div>
        <div className="flex items-center justify-between border-b border-slate-800 pb-2.5 gap-2">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2 min-w-0">
            <Zap className="w-4 h-4 text-amber-400 shrink-0" />
            <span className="truncate">Conexões Físicas (Pico → P7501)</span>
          </h3>
          <Badge variant="success" className="shrink-0">
            P7501 [B6B-ZR-3R4]
          </Badge>
        </div>

        <div className="mt-2.5 p-2.5 bg-amber-950/30 border border-amber-800/60 rounded-lg text-xs space-y-1">
          <div className="flex items-center gap-1.5 text-amber-400 font-bold text-[11px]">
            <AlertTriangle className="w-3.5 h-3.5 text-amber-400 shrink-0" />
            <span>DISCLAIMER CRÍTICO: IDENTIFICAÇÃO DO TERRA (GND)</span>
          </div>
          <p className="text-slate-300 text-[11px] leading-relaxed font-sans">
            • <strong>Conector P7501 (UART 115200 8N1):</strong> O GND oficial é o <strong>PINO 4</strong>; mantenha <strong>GP3 / NSW5V (Pino 5)</strong> apenas como entrada Hi-Z (~3.29V em repouso; rail IC1506 ~5.15V — NUNCA dirigir push-pull).<br />
            • <strong>XRESET (Pino 2 / GP5):</strong> Operar estritamente em Open-Drain.<br />
            • <span className="text-amber-300 font-medium">Valide sempre o bipe de 0.0Ω no multímetro antes de energizar!</span>
          </p>
        </div>
      </div>

      <div className="space-y-1 text-[11px] font-mono pt-2 border-t border-slate-800/80">
        {ssot.pinout.length === 0 ? (
          <p className="text-slate-500 py-2">Nenhum pino P7501 mapeado no SSOT.</p>
        ) : (
          ssot.pinout.map(pin => {
            const isGnd = pin.manual_pin === 4;
            const labelClass = PIN_COLOR_MAP[pin.manual_pin] || 'text-sky-400';
            return (
              <div
                key={pin.manual_pin}
                className={`flex flex-col 2xl:flex-row 2xl:items-center justify-between py-1 gap-0.5 border-b last:border-b-0 ${
                  isGnd ? 'border-emerald-800/60 bg-emerald-950/20 px-1.5 rounded' : 'border-slate-800/60'
                }`}
              >
                <span className={`${labelClass} font-medium break-words`}>{pin.pin_label}</span>
                <span
                  className={`text-[10px] 2xl:text-[11px] 2xl:text-right shrink-0 ${
                    isGnd ? 'text-emerald-300 font-bold' : 'text-slate-200'
                  }`}
                >
                  {pin.pico_gpio} • {pin.signal_name}
                </span>
              </div>
            );
          })
        )}
      </div>
    </Card>
  );
};
