import React from 'react';
import { useDashboardViewModel } from '@/viewmodels/useDashboardViewModel';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { ViewKey } from '@/components/Sidebar';
import {
  HardDrive,
  Cpu,
  Radio,
  Lock,
  BookOpen,
  Binary,
  CheckCircle2,
  AlertTriangle,
  FolderCheck,
  ArrowRight,
  ShieldAlert,
} from 'lucide-react';

interface DashboardViewProps {
  onNavigate: (view: ViewKey) => void;
  onQuickClean?: () => void;
}

export const DashboardView: React.FC<DashboardViewProps> = ({ onNavigate }) => {
  const { overview, loading } = useDashboardViewModel();

  if (loading || !overview) {
    return (
      <div className="flex items-center justify-center h-full text-slate-500 font-mono text-xs">
        Carregando telemetria e integridade da bancada DMR-EH55...
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12">
      {/* Alert Banner: Root cause analysis of U99 */}
      <div className="rounded-2xl bg-gradient-to-r from-amber-500/10 via-amber-500/5 to-transparent border border-amber-500/30 p-5 shadow-xl relative overflow-hidden">
        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div className="flex items-start gap-3.5">
            <div className="p-2.5 rounded-xl bg-amber-500/20 text-amber-400 border border-amber-500/30 shrink-0">
              <ShieldAlert className="w-6 h-6" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-sm font-bold font-mono text-amber-300">
                  DIAGNÓSTICO CRÍTICO: {overview.error_code}
                </h2>
                <StatusBadge variant="warning" label="AUTO-CHECK TIMEOUT" />
              </div>
              <p className="text-xs text-slate-300 mt-1 max-w-3xl leading-relaxed">
                {overview.error_description}
              </p>
            </div>
          </div>

          <button
            onClick={() => onNavigate('nvram')}
            className="px-4 py-2 rounded-xl bg-amber-500 hover:bg-amber-400 text-slate-950 font-mono font-bold text-xs flex items-center gap-2 shadow-lg shadow-amber-950/40 transition-all shrink-0"
          >
            <span>Executar Limpeza Cirúrgica</span>
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Top Telemetry Metric Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Spansion NOR Flash"
          value="8.00"
          unit="MB"
          subtitle="Top-Boot (SA134..SA141 EEMUL)"
          variant="amber"
          icon={<Cpu className="w-4 h-4" />}
          badge={<StatusBadge variant="warning" label="SA134 CORROMPIDO" />}
        />
        <MetricCard
          title="Head Dump do HD"
          value="100.0"
          unit="MB"
          subtitle="MEIHDFS 204.800 setores LBA"
          variant="cyan"
          icon={<HardDrive className="w-4 h-4" />}
          badge={<StatusBadge variant="success" label="MBR SH VERIFICADO" />}
        />
        <MetricCard
          title="Entropia PANA_DVD.FRM"
          value="7.99958"
          unit="bits/byte"
          subtitle="Cifra ECB Feistel 64-bits KPA"
          variant="slate"
          icon={<Lock className="w-4 h-4" />}
          badge={<StatusBadge variant="info" label="3 CPUs TOC" />}
        />
        <MetricCard
          title="Interface DWire32A"
          value="3.28"
          unit="V (ADC0)"
          subtitle="RP2040 PIO0 • 1.0 MHz OCD Bus"
          variant="emerald"
          icon={<Radio className="w-4 h-4" />}
          badge={<StatusBadge variant="success" label="VDD OK" />}
        />
      </div>

      {/* Quick Launch Instruments */}
      <div>
        <h3 className="text-xs font-mono font-semibold uppercase tracking-wider text-slate-400 mb-3 flex items-center gap-2">
          <span>Instrumentos de Bancada & Recuperação</span>
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Card 1: NVRAM */}
          <div
            onClick={() => onNavigate('nvram')}
            className="group cursor-pointer p-4 rounded-xl bg-bench-900 border border-slate-800 hover:border-amber-500/50 hover:bg-bench-850 transition-all duration-150 shadow-lg flex flex-col justify-between"
          >
            <div>
              <div className="flex items-center justify-between mb-2">
                <div className="p-2 rounded-lg bg-amber-500/10 text-amber-400 border border-amber-500/20 group-hover:scale-110 transition-transform">
                  <Cpu className="w-5 h-5" />
                </div>
                <ArrowRight className="w-4 h-4 text-slate-600 group-hover:text-amber-400 group-hover:translate-x-0.5 transition-all" />
              </div>
              <h4 className="text-xs font-bold font-mono text-slate-200 group-hover:text-amber-300">
                Cirurgia de NVRAM (U99)
              </h4>
              <p className="text-[11px] text-slate-400 mt-1 leading-relaxed">
                Auditoria de microsetores SA0..SA141, limpeza cirúrgica dos 64KB de EEPROM emulada (0xFF) mantendo o bootloader imutável.
              </p>
            </div>
            <div className="mt-4 pt-3 border-t border-slate-800 flex items-center justify-between text-[11px] font-mono text-amber-400">
              <span>8 Microsetores de 8KB</span>
              <span className="text-slate-500">1-Clique Fix</span>
            </div>
          </div>

          {/* Card 2: HDD Forensics */}
          <div
            onClick={() => onNavigate('hdd')}
            className="group cursor-pointer p-4 rounded-xl bg-bench-900 border border-slate-800 hover:border-cyan-500/50 hover:bg-bench-850 transition-all duration-150 shadow-lg flex flex-col justify-between"
          >
            <div>
              <div className="flex items-center justify-between mb-2">
                <div className="p-2 rounded-lg bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 group-hover:scale-110 transition-transform">
                  <HardDrive className="w-5 h-5" />
                </div>
                <ArrowRight className="w-4 h-4 text-slate-600 group-hover:text-cyan-400 group-hover:translate-x-0.5 transition-all" />
              </div>
              <h4 className="text-xs font-bold font-mono text-slate-200 group-hover:text-cyan-300">
                Forensics do Disco Rígido (100MB)
              </h4>
              <p className="text-[11px] text-slate-400 mt-1 leading-relaxed">
                Validação MBR "SH" vs Espelho LBA 57.344, decodificação MEIHDFS (XOR 0xFF), histórico do Journal WAL (10 blocos) e extração com manifesto.
              </p>
            </div>
            <div className="mt-4 pt-3 border-t border-slate-800 flex items-center justify-between text-[11px] font-mono text-cyan-400">
              <span>256 Registros MEIHDFS</span>
              <span className="text-slate-500">SHA256SUMS</span>
            </div>
          </div>

          {/* Card 3: PicoBench */}
          <div
            onClick={() => onNavigate('picobench')}
            className="group cursor-pointer p-4 rounded-xl bg-bench-900 border border-slate-800 hover:border-emerald-500/50 hover:bg-bench-850 transition-all duration-150 shadow-lg flex flex-col justify-between"
          >
            <div>
              <div className="flex items-center justify-between mb-2">
                <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 group-hover:scale-110 transition-transform">
                  <Radio className="w-5 h-5" />
                </div>
                <ArrowRight className="w-4 h-4 text-slate-600 group-hover:text-emerald-400 group-hover:translate-x-0.5 transition-all" />
              </div>
              <h4 className="text-xs font-bold font-mono text-slate-200 group-hover:text-emerald-300">
                Bancada PicoBench DWire32A
              </h4>
              <p className="text-[11px] text-slate-400 mt-1 leading-relaxed">
                Controle do Raspberry Pi Pico via USB CDC, monitoramento ADC0 VDD, Cold Break-In Modo 0..3 e injeção do RAM Loader in-circuit.
              </p>
            </div>
            <div className="mt-4 pt-3 border-t border-slate-800 flex items-center justify-between text-[11px] font-mono text-emerald-400">
              <span>AM34 OCD Bus Master</span>
              <span className="text-slate-500">In-Circuit Erase</span>
            </div>
          </div>
        </div>
      </div>

      {/* Binary Status & Hardware Table */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Hardware Architecture Spec */}
        <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl">
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

        {/* Local Binaries Status Table */}
        <div className="lg:col-span-2 rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-xs font-mono font-semibold uppercase tracking-wider text-slate-400 flex items-center gap-2">
              <FolderCheck className="w-4 h-4 text-emerald-400" />
              <span>Status dos Arquivos de Firmware e Bancada</span>
            </h3>
            <span className="text-[11px] font-mono text-slate-500">
              {overview.binaries.filter(b => b.exists).length} de {overview.binaries.length} ativos
            </span>
          </div>

          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs font-mono">
              <thead>
                <tr className="border-b border-slate-800 text-slate-500 uppercase text-[10px]">
                  <th className="py-2 px-3">Binário / Imagem</th>
                  <th className="py-2 px-3">Tamanho</th>
                  <th className="py-2 px-3">Função no Sistema</th>
                  <th className="py-2 px-3 text-right">Integridade</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60">
                {overview.binaries.map(bin => (
                  <tr key={bin.id} className="hover:bg-slate-800/40 transition-colors">
                    <td className="py-2.5 px-3">
                      <div className="font-semibold text-slate-200 flex items-center gap-2">
                        {bin.exists ? (
                          <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                        ) : (
                          <AlertTriangle className="w-3.5 h-3.5 text-amber-400 shrink-0" />
                        )}
                        <span className="truncate max-w-[220px]" title={bin.name}>
                          {bin.name}
                        </span>
                      </div>
                      <span className="text-[10px] text-slate-500 truncate block">
                        {bin.path}
                      </span>
                    </td>
                    <td className="py-2.5 px-3 text-slate-300">
                      {bin.formatted_size ?? '-'}
                    </td>
                    <td className="py-2.5 px-3 text-slate-400 max-w-[240px] truncate text-[11px]">
                      {bin.expected_role}
                    </td>
                    <td className="py-2.5 px-3 text-right">
                      {bin.status === 'ready' && (
                        <StatusBadge variant="success" label="PRONTO" />
                      )}
                      {bin.status === 'warning' && (
                        <StatusBadge variant="warning" label="U99 DUMP" />
                      )}
                      {bin.status === 'missing' && (
                        <StatusBadge variant="error" label="AUSENTE" />
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
};
