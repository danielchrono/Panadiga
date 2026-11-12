import React from 'react';
import { StatusBadge } from './StatusBadge';
import {
  Cpu,
  Activity,
  Zap,
  FlaskConical,
  RefreshCw,
} from 'lucide-react';

interface HeaderProps {
  currentViewTitle: string;
  demoMode: boolean;
  onToggleDemoMode: (val: boolean) => void;
  onRefresh?: () => void;
  vddVolts?: number;
  hardwareConnected?: boolean;
}

export const Header: React.FC<HeaderProps> = ({
  currentViewTitle,
  demoMode,
  onToggleDemoMode,
  onRefresh,
  vddVolts = 0.0,
  hardwareConnected = false,
}) => {
  return (
    <header className="h-16 px-6 bg-bench-900/90 backdrop-blur border-b border-slate-800 flex items-center justify-between select-none z-10 shrink-0">
      {/* Left: View breadcrumbs */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-lg bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center text-cyan-400">
            <Cpu className="w-4 h-4" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="text-xs font-mono font-medium text-slate-400 uppercase tracking-wider">
                PANADIGA // DMR-EH55
              </span>
              <span className="text-slate-600">/</span>
              <span className="text-sm font-semibold text-slate-100 font-mono">
                {currentViewTitle}
              </span>
            </div>
            <p className="text-[11px] text-slate-500 font-mono">
              Matsushita UniPhier AM34 • Spansion S29JL064H • MEIHDFS 100MB
            </p>
          </div>
        </div>
      </div>

      {/* Right: Telemetry pill & Demo Mode controls */}
      <div className="flex items-center gap-3">
        {/* Hardware Status Pill */}
        <div className="hidden md:flex items-center gap-2 px-3 py-1.5 rounded-lg bg-bench-950 border border-slate-800 font-mono text-xs">
          <Activity className="w-3.5 h-3.5 text-cyan-400 animate-pulse" />
          <span className="text-slate-400">DWire32A:</span>
          {hardwareConnected ? (
            vddVolts >= 2.8 ? (
              <span className="text-emerald-400 font-semibold flex items-center gap-1">
                ONLINE <span className="text-[10px] text-slate-500">({vddVolts.toFixed(2)}V)</span>
              </span>
            ) : (
              <span className="text-amber-400 font-semibold flex items-center gap-1">
                PICO CONECTADO{' '}
                <span className="text-[10px] text-slate-500">({vddVolts > 0.2 ? `${vddVolts.toFixed(2)}V` : 'Alvo 0V - Desligado'})</span>
              </span>
            )
          ) : (
            <span className="text-rose-400 font-semibold">OFFLINE</span>
          )}
        </div>

        {/* Self-check / U99 Status */}
        <StatusBadge
          variant="warning"
          label="U99 RECOVERY SUITE"
          className="hidden sm:inline-flex"
        />

        {/* Demo Mode Switch */}
        <button
          onClick={() => onToggleDemoMode(!demoMode)}
          role="switch"
          aria-checked={demoMode}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-medium border transition-all focus:outline-none focus:ring-2 focus:ring-amber-400 ${
            demoMode
              ? 'bg-amber-500/10 text-amber-300 border-amber-500/40 shadow-sm shadow-amber-950'
              : 'bg-slate-800 text-slate-300 border-slate-700 hover:bg-slate-750'
          }`}
          title="Alternar entre Mock IPC e hardware real"
          aria-label="Alternar modo demonstração e hardware real"
        >
          <FlaskConical className="w-3.5 h-3.5" />
          <span>{demoMode ? 'Modo Demonstração: ATIVO' : 'Modo Hardware: IPC'}</span>
        </button>

        {/* Refresh Button */}
        {onRefresh && (
          <button
            onClick={onRefresh}
            className="p-2 rounded-lg bg-slate-800/80 border border-slate-700 hover:bg-slate-750 text-slate-300 hover:text-slate-100 transition-colors focus:outline-none focus:ring-2 focus:ring-slate-400"
            title="Atualizar dados de telemetria"
            aria-label="Atualizar dados de telemetria"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </button>
        )}
      </div>
    </header>
  );
};
