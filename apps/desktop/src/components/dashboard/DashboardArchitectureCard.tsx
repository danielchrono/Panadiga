import React from 'react';
import { Cpu } from 'lucide-react';
import { SystemOverview } from '@/models';

interface DashboardArchitectureCardProps {
  overview: SystemOverview;
}

export const DashboardArchitectureCard: React.FC<DashboardArchitectureCardProps> = ({ overview }) => {
  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl font-mono">
      <h3 className="text-xs font-mono font-semibold uppercase tracking-wider text-slate-400 mb-4 flex items-center gap-2">
        <Cpu className="w-4 h-4 text-cyan-400" />
        <span>Arquitetura do Panasonic DMR-EH55</span>
      </h3>

      <div className="space-y-3 text-xs font-mono">
        <div className="flex items-center justify-between py-1.5 border-b border-slate-800/60">
          <span className="text-slate-500">SoC Digital</span>
          <span className="text-slate-200 font-semibold">{overview.soc_processor}</span>
        </div>
        <div className="flex items-center justify-between py-1.5 border-b border-slate-800/60">
          <span className="text-slate-500">Core Principal</span>
          <span className="text-slate-200">{overview.core_architecture}</span>
        </div>
        <div className="flex items-center justify-between py-1.5 border-b border-slate-800/60">
          <span className="text-slate-500">Flash ROM</span>
          <span className="text-slate-200">{overview.flash_rom}</span>
        </div>
        <div className="flex items-center justify-between py-1.5 border-b border-slate-800/60">
          <span className="text-slate-500">Disco Rígido</span>
          <span className="text-slate-200">{overview.hdd_specs}</span>
        </div>
        <div className="flex items-center justify-between py-1.5 border-b border-slate-800/60">
          <span className="text-slate-500">Drive Óptico</span>
          <span className="text-slate-200">{overview.optical_drive}</span>
        </div>
        <div className="flex items-center justify-between py-1.5 border-b border-slate-800/60">
          <span className="text-slate-500">Porta OCD</span>
          <span className="text-cyan-400 font-semibold">{overview.debug_port}</span>
        </div>
        <div className="flex items-center justify-between py-1.5">
          <span className="text-slate-500">Firmware</span>
          <span className="text-emerald-400 font-bold">{overview.firmware_version}</span>
        </div>
      </div>
    </div>
  );
};
