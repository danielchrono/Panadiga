import React from 'react';
import { ColdBreakinMode } from '@/models';
import { StatusBadge } from '@/components/StatusBadge';
import { Sliders } from 'lucide-react';

export interface ColdBreakinSelectorProps {
  currentMode: ColdBreakinMode;
  onSelectMode: (mode: ColdBreakinMode) => void;
}

const BREAKIN_MODES: Array<{
  mode: ColdBreakinMode;
  title: string;
  description: string;
  timing: string;
  isRecommended?: boolean;
}> = [
  {
    mode: 0,
    title: 'Modo 0: EXTRG High (Pre-Only)',
    description: 'EXTRG Idle LOW → Active HIGH na liberação de reset. 32 clocks durante reset + 200µs settle.',
    timing: 'Pre-Reset: 32 Clks | Post-Reset: 200µs delay',
  },
  {
    mode: 1,
    title: 'Modo 1: EXTRG Low (Pre-Only)',
    description: 'EXTRG Idle HIGH → Active LOW (gatilho invertido). 32 clocks durante reset.',
    timing: 'Pre-Reset: 32 Clks | Post-Reset: 200µs delay',
  },
  {
    mode: 2,
    title: 'Modo 2: EXTRG High (Pre + Post Burst)',
    description: 'Padrão recomendado para AM34/UniPhier. 32 clocks no reset + burst imediato de 32 clocks pós-reset.',
    timing: 'Pre: 32 Clks | Post: 32 Clks imediatos',
    isRecommended: true,
  },
  {
    mode: 3,
    title: 'Modo 3: EXTRG Low (Pre + Post Burst)',
    description: 'Gatilho invertido com burst duplo pré e pós-reset.',
    timing: 'Pre: 32 Clks | Post: 32 Clks imediatos',
  },
];

/**
 * Presentation Component: Selects Cold Break-In timing profile for intercepting CPU boot vector.
 */
export const ColdBreakinSelector: React.FC<ColdBreakinSelectorProps> = ({
  currentMode,
  onSelectMode,
}) => {
  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
          <Sliders className="w-4 h-4 text-cyan-400" />
          <span>Seletor de Modo de Cold Break-In</span>
        </h3>
        <p className="text-[11px] text-slate-500 font-sans mt-0.5">
          Garante o travamento da CPU AM34 antes da primeira instrução do bootloader.
        </p>
      </div>

      <div
        className="space-y-2.5"
        role="radiogroup"
        aria-label="Perfil de Timing do Cold Break-In"
      >
        {BREAKIN_MODES.length === 0 ? (
          <div className="text-slate-500 text-xs p-3 font-mono">Nenhum modo configurado</div>
        ) : (
          BREAKIN_MODES.map(m => {
          const isSelected = currentMode === m.mode;
          return (
            <div
              key={m.mode}
              role="radio"
              aria-checked={isSelected}
              tabIndex={0}
              onClick={() => onSelectMode(m.mode)}
              onKeyDown={(e) => {
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
              <div className="flex items-center justify-between mb-1">
                <span className={`text-xs font-bold ${isSelected ? 'text-cyan-400' : 'text-slate-200'}`}>
                  {m.title}
                </span>
                {m.isRecommended && <StatusBadge variant="success" label="RECOMENDADO" />}
              </div>
              <p className="text-[11px] text-slate-400 font-sans leading-relaxed">
                {m.description}
              </p>
              <span className="text-[10px] text-slate-500 block mt-2 font-mono">
                {m.timing}
              </span>
            </div>
          );
        })
        )}
      </div>
    </div>
  );
};
