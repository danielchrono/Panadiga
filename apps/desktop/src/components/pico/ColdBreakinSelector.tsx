import React from 'react';
import type { BreakInModeSpec, ColdBreakinMode } from '@/models';
import { DEFAULT_HARDWARE_SSOT_SPEC } from '@/constants';
import { Badge, Card } from '@/components/ui';
import { Sliders } from 'lucide-react';

export interface ColdBreakinSelectorProps {
  currentMode: ColdBreakinMode;
  onSelectMode: (mode: ColdBreakinMode) => void;
  breakinModes?: BreakInModeSpec[];
}

/**
 * Presentation Component: Selects Cold Break-In timing profile for intercepting CPU boot vector
 * driven directly from Rust SSOT (`ssot.breakin_modes`).
 */
export const ColdBreakinSelector: React.FC<ColdBreakinSelectorProps> = ({
  currentMode,
  onSelectMode,
  breakinModes = DEFAULT_HARDWARE_SSOT_SPEC.breakin_modes,
}) => {
  return (
    <Card className="p-5 shadow-xl space-y-4">
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
          <Sliders className="w-4 h-4 text-cyan-400" />
          <span>Seletor de Perfil Boot/Reset P7501</span>
        </h3>
        <p className="text-[11px] text-slate-500 font-sans mt-0.5">
          Controla SW_UART (GP26) e pulso Open-Drain XRESET (GP5) para captura UART / JIGBOOT.
        </p>
      </div>

      <div
        className="space-y-2.5"
        role="radiogroup"
        aria-label="Perfil de Boot e Reset P7501"
      >
        {breakinModes.length === 0 ? (
          <div className="text-slate-500 text-xs p-3 font-mono">Nenhum modo configurado</div>
        ) : (
          breakinModes.map(m => {
            const isSelected = currentMode === m.mode;
            return (
              <div
                key={m.mode}
                role="radio"
                aria-checked={isSelected}
                tabIndex={0}
                onClick={() => onSelectMode(m.mode)}
                onKeyDown={e => {
                  if (e.key === ' ' || e.key === 'Enter') {
                    e.preventDefault();
                    onSelectMode(m.mode);
                  }
                }}
                className={`p-3.5 rounded-xl border cursor-pointer transition-all focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-bench-900 ${
                  isSelected
                    ? 'bg-cyan-500/10 border-cyan-500/50 shadow-md shadow-cyan-950/20'
                    : 'bg-slate-950 border-slate-800 hover:border-slate-700'
                }`}
              >
                <div className="flex items-center justify-between mb-1 gap-2">
                  <span className={`text-xs font-bold ${isSelected ? 'text-cyan-400' : 'text-slate-200'}`}>
                    {m.label}
                  </span>
                  {m.mode === 2 && <Badge variant="success">{m.badge}</Badge>}
                </div>
                <p className="text-[11px] text-slate-400 font-sans leading-relaxed">
                  {m.description}
                </p>
                <span className="text-[10px] text-slate-500 block mt-2 font-mono">
                  {m.signal_diagram}
                </span>
              </div>
            );
          })
        )}
      </div>
    </Card>
  );
};
