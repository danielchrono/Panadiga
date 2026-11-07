import React from 'react';
import { Am34RegisterState } from '@/models';
import { formatHex32 } from '@/utils/formatters';
import { Cpu } from 'lucide-react';

export interface Am34RegisterGridProps {
  registers: Am34RegisterState | null;
}

/**
 * Pure Functional Presentation Component: Displays AM34 / MN103S CPU register bank in real-time.
 */
export const Am34RegisterGrid: React.FC<Am34RegisterGridProps> = ({ registers }) => {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between text-xs text-slate-400">
        <span className="font-semibold text-slate-300 flex items-center gap-1.5">
          <Cpu className="w-3.5 h-3.5 text-cyan-400" />
          <span>Banco de Registradores Matsushita AM34 (Em Tempo Real)</span>
        </span>
        <span className="text-[11px] text-slate-500 font-mono">
          {registers?.is_halted ? '● CPU Interceptada (OCD HALT)' : '○ CPU em Execução'}
        </span>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-12 gap-2 text-center text-xs">
        <div className="p-2 rounded-lg bg-slate-950 border border-cyan-500/30 col-span-2 shadow-sm">
          <span className="text-[10px] text-cyan-400 block font-bold">PC (Program Counter)</span>
          <span className="font-mono text-cyan-200 text-xs">
            {formatHex32(registers?.pc, '0x80000000')}
          </span>
        </div>
        <div className="p-2 rounded-lg bg-slate-950 border border-purple-500/30 col-span-2 shadow-sm">
          <span className="text-[10px] text-purple-400 block font-bold">SP (Stack Pointer)</span>
          <span className="font-mono text-purple-200 text-xs">
            {formatHex32(registers?.sp, '0x0003FFF0')}
          </span>
        </div>
        <div className="p-2 rounded-lg bg-slate-950 border border-slate-800 col-span-2">
          <span className="text-[10px] text-slate-400 block font-bold">PSW (Status Word)</span>
          <span className="font-mono text-slate-300 text-xs">
            {formatHex32(registers?.psw, '0x00000100')}
          </span>
        </div>
        <div className="p-2 rounded-lg bg-slate-950 border border-slate-800 col-span-2">
          <span className="text-[10px] text-slate-400 block font-bold">MDR (Mult/Div)</span>
          <span className="font-mono text-slate-300 text-xs">
            {formatHex32(registers?.mdr, '0x00000000')}
          </span>
        </div>
        <div className="p-2 rounded-lg bg-slate-950 border border-slate-800 col-span-2">
          <span className="text-[10px] text-slate-400 block font-bold">D0 (Data 0)</span>
          <span className="font-mono text-slate-300 text-xs">
            {formatHex32(registers?.d?.[0], '0x00000000')}
          </span>
        </div>
        <div className="p-2 rounded-lg bg-slate-950 border border-slate-800 col-span-2">
          <span className="text-[10px] text-slate-400 block font-bold">A0 (Addr 0)</span>
          <span className="font-mono text-slate-300 text-xs">
            {formatHex32(registers?.a?.[0], '0x00000000')}
          </span>
        </div>
      </div>
    </div>
  );
};
