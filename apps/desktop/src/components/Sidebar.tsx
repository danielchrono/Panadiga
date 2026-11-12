import React from 'react';
import { cn } from '@/lib/utils';
import { useTranslation } from '@/i18n/useTranslation';
import {
  LayoutDashboard,
  Cpu,
  HardDrive,
  Radio,
  Lock,
  BookOpen,
  Binary,
  Activity,
  Layers,
} from 'lucide-react';

export type ViewKey =
  | 'dashboard'
  | 'nvram'
  | 'hdd'
  | 'picobench'
  | 'frmcrypto'
  | 'manuals'
  | 'hexinspector';

interface SidebarProps {
  currentView: ViewKey;
  onSelectView: (view: ViewKey) => void;
  vddVolts?: number;
  nvramCleaned?: boolean;
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentView,
  onSelectView,
  vddVolts = 0.0,
  nvramCleaned = false,
}) => {
  const navItems: Array<{
    key: ViewKey;
    label: string;
    sublabel: string;
    icon: React.ReactNode;
    badge?: string;
    badgeColor?: string;
  }> = [
    {
      key: 'dashboard',
      label: 'Visão Geral',
      sublabel: 'Status do Gravador & Binários',
      icon: <LayoutDashboard className="w-4 h-4" />,
    },
    {
      key: 'nvram',
      label: 'Flash & NVRAM',
      sublabel: 'Spansion SA134..SA141 U99',
      icon: <Cpu className="w-4 h-4" />,
      badge: nvramCleaned ? 'LIMPO' : 'U99',
      badgeColor: nvramCleaned ? 'bg-emerald-500/20 text-emerald-400' : 'bg-rose-500/20 text-rose-400',
    },
    {
      key: 'hdd',
      label: 'Armazenamento HD',
      sublabel: 'MEIHDFS 100MB & Partições',
      icon: <HardDrive className="w-4 h-4" />,
      badge: '100 MB',
      badgeColor: 'bg-cyan-500/20 text-cyan-400',
    },
    {
      key: 'picobench',
      label: 'Bancada PicoBench',
      sublabel: 'RP2040 DWire32A & RAM Loader',
      icon: <Radio className="w-4 h-4" />,
      badge: `${vddVolts.toFixed(1)}V`,
      badgeColor: 'bg-emerald-500/20 text-emerald-400',
    },
    {
      key: 'frmcrypto',
      label: 'Criptoanálise FRM',
      sublabel: 'Dissecador PANA_DVD & ECB',
      icon: <Lock className="w-4 h-4" />,
      badge: '7.999b',
      badgeColor: 'bg-purple-500/20 text-purple-400',
    },
    {
      key: 'manuals',
      label: 'Manuais & Esquemas',
      sublabel: '15 PDFs & Indexação Rápida',
      icon: <BookOpen className="w-4 h-4" />,
      badge: '15',
      badgeColor: 'bg-slate-700 text-slate-300',
    },
    {
      key: 'hexinspector',
      label: 'Inspetor Hexadecimal',
      sublabel: 'Navegação LBA & Offset ASCII',
      icon: <Binary className="w-4 h-4" />,
    },
  ];

  return (
    <aside className="w-64 bg-bench-950 border-r border-slate-800 flex flex-col justify-between shrink-0 select-none">
      {/* Top Logo / System Title */}
      <div>
        <div className="p-4 border-b border-slate-800/80 bg-bench-900/40">
          <div className="flex items-center gap-2.5">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center shadow-lg shadow-cyan-950 text-slate-950 font-black">
              <Layers className="w-5 h-5 text-white" />
            </div>
            <div>
              <h1 className="text-sm font-bold tracking-tight text-slate-100 font-mono">
                DIGA SUITE
              </h1>
              <span className="text-[10px] text-cyan-400 font-mono tracking-wider">
                ENGINEERING V1.0
              </span>
            </div>
          </div>
        </div>

        {/* Navigation Items */}
        <nav className="p-3 space-y-1" aria-label="Navegação Principal do Sistema">
          {navItems.length === 0 ? (
            <div className="p-4 text-center text-xs text-slate-500 font-mono">
              Nenhum módulo de bancada disponível
            </div>
          ) : (
            navItems.map(item => {
              const isActive = currentView === item.key;
              return (
                <button
                  key={item.key}
                  data-testid={`nav-${item.key}`}
                  onClick={() => onSelectView(item.key)}
                  aria-current={isActive ? 'page' : undefined}
                  className={cn(
                    'w-full flex items-center justify-between px-3 py-2.5 rounded-xl text-left transition-all duration-150 focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400',
                    isActive
                      ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 shadow-md shadow-cyan-950/20'
                      : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900 border border-transparent'
                  )}
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <div
                      className={cn(
                        isActive ? 'text-cyan-400' : 'text-slate-500'
                      )}
                    >
                      {item.icon}
                    </div>
                    <div className="truncate">
                      <div className="text-xs font-semibold leading-tight font-mono">
                        {item.label}
                      </div>
                      <div className="text-[10px] text-slate-500 truncate leading-tight font-sans">
                        {item.sublabel}
                      </div>
                    </div>
                  </div>

                  {item.badge && (
                    <span
                      className={`ml-2 text-[10px] font-mono px-1.5 py-0.5 rounded font-medium ${item.badgeColor}`}
                    >
                      {item.badge}
                    </span>
                  )}
                </button>
              );
            })
          )}
        </nav>
      </div>

      {/* Footer Bench Telemetry */}
      <div className="p-3 border-t border-slate-800/80 bg-bench-900/50">
        <div className="p-2.5 rounded-xl bg-slate-950 border border-slate-800/80 space-y-1.5 font-mono text-[11px]">
          <div className="flex items-center justify-between text-slate-400">
            <span className="flex items-center gap-1.5">
              <Activity className="w-3.5 h-3.5 text-cyan-400" />
              <span>Target Rail</span>
            </span>
            <span className="text-slate-200 font-bold">
              {vddVolts > 0.2 ? `${vddVolts.toFixed(2)} V` : '0.00 V (Sem Alimentação)'}
            </span>
          </div>

          <div className="flex items-center justify-between text-slate-400">
            <span>AM34 Core</span>
            <span
              className={
                vddVolts >= 2.8
                  ? 'text-emerald-400 font-semibold'
                  : 'text-amber-400 font-semibold'
              }
            >
              {vddVolts >= 2.8 ? 'HALTED (OCD)' : 'STANDBY (0V)'}
            </span>
          </div>

          <div className="flex items-center justify-between text-slate-400">
            <span>Spansion NOR</span>
            <span className="text-slate-200">8 MB (TSOP48)</span>
          </div>
        </div>
      </div>
    </aside>
  );
};
