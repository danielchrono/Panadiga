import React from 'react';
import { ViewKey } from '@/components/Sidebar';
import { HardDrive, Cpu, Radio, ArrowRight } from 'lucide-react';

interface DashboardInstrumentsProps {
  onNavigate: (view: ViewKey) => void;
}

export const DashboardInstruments: React.FC<DashboardInstrumentsProps> = ({ onNavigate }) => {
  return (
    <div>
      <h3 className="text-xs font-mono font-semibold uppercase tracking-wider text-slate-400 mb-3 flex items-center gap-2">
        <span>Instrumentos de Bancada & Recuperação</span>
      </h3>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Card 1: NVRAM */}
        <div
          role="button"
          tabIndex={0}
          aria-label="Abrir Cirurgia de NVRAM U99"
          onClick={() => onNavigate('nvram')}
          onKeyDown={e => (e.key === 'Enter' || e.key === ' ') && onNavigate('nvram')}
          className="group cursor-pointer p-4 rounded-xl bg-bench-900 border border-slate-800 hover:border-amber-500/50 hover:bg-bench-850 transition-all duration-150 shadow-lg flex flex-col justify-between focus:outline-none focus:ring-1 focus:ring-amber-500/50"
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
          role="button"
          tabIndex={0}
          aria-label="Abrir Forensics do Disco Rígido"
          onClick={() => onNavigate('hdd')}
          onKeyDown={e => (e.key === 'Enter' || e.key === ' ') && onNavigate('hdd')}
          className="group cursor-pointer p-4 rounded-xl bg-bench-900 border border-slate-800 hover:border-cyan-500/50 hover:bg-bench-850 transition-all duration-150 shadow-lg flex flex-col justify-between focus:outline-none focus:ring-1 focus:ring-cyan-500/50"
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
          role="button"
          tabIndex={0}
          aria-label="Abrir Bancada PicoBench P7501 UART"
          onClick={() => onNavigate('picobench')}
          onKeyDown={e => (e.key === 'Enter' || e.key === ' ') && onNavigate('picobench')}
          className="group cursor-pointer p-4 rounded-xl bg-bench-900 border border-slate-800 hover:border-emerald-500/50 hover:bg-bench-850 transition-all duration-150 shadow-lg flex flex-col justify-between focus:outline-none focus:ring-1 focus:ring-emerald-500/50"
        >
          <div>
            <div className="flex items-center justify-between mb-2">
              <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 group-hover:scale-110 transition-transform">
                <Radio className="w-5 h-5" />
              </div>
              <ArrowRight className="w-4 h-4 text-slate-600 group-hover:text-emerald-400 group-hover:translate-x-0.5 transition-all" />
            </div>
            <h4 className="text-xs font-bold font-mono text-slate-200 group-hover:text-emerald-300">
              Bancada PicoBench (P7501 UART / OCD)
            </h4>
            <p className="text-[11px] text-slate-400 mt-1 leading-relaxed">
              Captura UART P7501 (115200 8N1), matriz elétrica de 6 pinos, monitoramento ADC0 SW_UART e modo legado Cold Break-In / RAM Loader.
            </p>
          </div>
          <div className="mt-4 pt-3 border-t border-slate-800 flex items-center justify-between text-[11px] font-mono text-emerald-400">
            <span>P7501 Async UART</span>
            <span className="text-slate-500">Boot &amp; Recovery</span>
          </div>
        </div>
      </div>
    </div>
  );
};
