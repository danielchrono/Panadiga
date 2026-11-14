import React from 'react';
import { StatusBadge } from './StatusBadge';
import { Button } from '@/components/ui';
import { isTargetPowered } from '@/utils/formatters';
import { Cpu, Activity, RefreshCw, Menu } from 'lucide-react';

interface HeaderProps {
  currentViewTitle: string;
  onRefresh?: () => void;
  onOpenMobileNav?: () => void;
  vddVolts?: number;
  hardwareConnected?: boolean;
}

export const Header: React.FC<HeaderProps> = ({
  currentViewTitle,
  onRefresh,
  onOpenMobileNav,
  vddVolts = 0.0,
  hardwareConnected = false,
}) => {
  const powered = isTargetPowered(vddVolts);

  return (
    <header className="min-h-16 px-4 sm:px-6 py-2 bg-bench-900/90 backdrop-blur border-b border-slate-800 flex items-center justify-between gap-2 select-none z-10 shrink-0">
      {/* Left: Mobile Drawer Trigger + View breadcrumbs */}
      <div className="flex items-center gap-2.5 min-w-0">
        {onOpenMobileNav && (
          <Button
            type="button"
            variant="outline"
            size="icon"
            onClick={onOpenMobileNav}
            className="lg:hidden h-9 w-9 shrink-0 bg-slate-900 border-slate-700"
            aria-label="Abrir navegação principal"
          >
            <Menu className="w-4 h-4 text-cyan-400" />
          </Button>
        )}

        <div className="flex items-center gap-2 min-w-0">
          <div className="hidden sm:flex w-8 h-8 rounded-lg bg-cyan-500/10 border border-cyan-500/30 items-center justify-center text-cyan-400 shrink-0">
            <Cpu className="w-4 h-4" />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2 min-w-0">
              <span className="text-xs font-mono font-medium text-slate-400 uppercase tracking-wider shrink-0">
                PANADIGA // DMR-EH55
              </span>
              <span className="text-slate-600 hidden sm:inline">/</span>
              <span className="text-xs sm:text-sm font-semibold text-slate-100 font-mono truncate">
                {currentViewTitle}
              </span>
            </div>
            <p className="hidden md:block text-[11px] text-slate-500 font-mono truncate">
              Matsushita UniPhier AM34 • Spansion S29JL064H • MEIHDFS 100MB
            </p>
          </div>
        </div>
      </div>

      {/* Right: Hardware Telemetry pill */}
      <div className="flex items-center gap-2 sm:gap-3 shrink-0">
        <div className="hidden md:flex items-center gap-2 px-3 py-1.5 rounded-lg bg-bench-950 border border-slate-800 font-mono text-xs">
          <Activity className="w-3.5 h-3.5 text-cyan-400 animate-pulse" />
          <span className="text-slate-400">P7501 UART:</span>
          {hardwareConnected ? (
            powered ? (
              <span className="text-emerald-400 font-semibold flex items-center gap-1">
                ONLINE <span className="text-[10px] text-slate-500">({vddVolts.toFixed(2)}V)</span>
              </span>
            ) : (
              <span className="text-amber-400 font-semibold flex items-center gap-1">
                PICO CONECTADO{' '}
                <span className="text-[10px] text-slate-500">
                  ({vddVolts > 0.2 ? `${vddVolts.toFixed(2)}V` : 'Alvo 0V - Desligado'})
                </span>
              </span>
            )
          ) : (
            <span className="text-rose-400 font-semibold">OFFLINE</span>
          )}
        </div>

        <StatusBadge
          variant="warning"
          label="U99 RECOVERY SUITE"
          className="hidden xl:inline-flex"
        />

        {onRefresh && (
          <Button
            type="button"
            variant="outline"
            size="icon"
            onClick={onRefresh}
            className="h-8 w-8 bg-slate-800/80 border-slate-700"
            title="Atualizar dados de telemetria"
            aria-label="Atualizar dados de telemetria"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </Button>
        )}
      </div>
    </header>
  );
};
