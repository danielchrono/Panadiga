import React, { useEffect, useRef } from 'react';
import { SerialLogEntry } from '@/models';
import { Terminal, Trash2, ArrowDown } from 'lucide-react';

interface TerminalViewProps {
  logs: SerialLogEntry[];
  onClear: () => void;
  title?: string;
  autoScroll?: boolean;
  containerRef?: React.RefObject<HTMLDivElement>;
}

export const TerminalView: React.FC<TerminalViewProps> = ({
  logs,
  onClear,
  title = 'DWire32A Bus Monitor / OCD CDC Console',
  autoScroll = true,
  containerRef,
}) => {
  const localRef = useRef<HTMLDivElement>(null);
  const scrollRef = containerRef || localRef;

  useEffect(() => {
    if (autoScroll && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  const levelColor = (level: SerialLogEntry['level']) => {
    switch (level) {
      case 'success':
        return 'text-emerald-400';
      case 'warn':
        return 'text-amber-400';
      case 'error':
        return 'text-rose-400';
      case 'rx':
        return 'text-cyan-400';
      case 'tx':
        return 'text-sky-300';
      default:
        return 'text-slate-300';
    }
  };

  const sourceBadge = (src: SerialLogEntry['source']) => {
    switch (src) {
      case 'PICO':
        return 'bg-cyan-950 text-cyan-400 border-cyan-800';
      case 'TARGET':
        return 'bg-purple-950 text-purple-300 border-purple-800';
      default:
        return 'bg-slate-800 text-slate-300 border-slate-700';
    }
  };

  return (
    <div className="flex flex-col h-full rounded-xl bg-slate-950/90 border border-slate-800 shadow-xl overflow-hidden font-mono">
      {/* Terminal Title Bar */}
      <div className="flex items-center justify-between px-3 py-2 bg-bench-900 border-b border-slate-800">
        <div className="flex items-center gap-2 text-xs font-semibold text-slate-300">
          <Terminal className="w-4 h-4 text-cyan-400" />
          <span>{title}</span>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-400">
            {logs.length} eventos
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={onClear}
            className="flex items-center gap-1 text-[11px] px-2 py-1 rounded hover:bg-slate-800 text-slate-400 hover:text-slate-200 transition-colors focus:outline-none focus:ring-2 focus:ring-slate-400"
            title="Limpar buffer"
            aria-label="Limpar buffer de eventos do terminal"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>Limpar</span>
          </button>
        </div>
      </div>

      {/* Terminal Log Stream */}
      <div
        ref={scrollRef}
        role="log"
        aria-live="polite"
        tabIndex={0}
        aria-label="Console de telemetria serial DWire32A"
        className="flex-1 p-3 overflow-y-auto space-y-1 text-xs select-text scrollbar-thin scrollbar-thumb-slate-800 focus:outline-none focus:ring-1 focus:ring-slate-700"
      >
        {logs.length === 0 ? (
          <div className="flex items-center justify-center h-32 text-slate-600 text-xs italic">
            Nenhum evento registrado no canal serial. Aguardando atividade de barramento...
          </div>
        ) : (
          logs.map(log => (
            <div key={log.id} className="flex items-start gap-2 leading-relaxed hover:bg-slate-900/50 px-1 py-0.5 rounded">
              <span className="text-[11px] text-slate-500 shrink-0">{log.timestamp}</span>
              <span className={`text-[10px] px-1 rounded border shrink-0 ${sourceBadge(log.source)}`}>
                {log.source}
              </span>
              <span className={`flex-1 break-all ${levelColor(log.level)}`}>
                {log.message}
              </span>
              {log.hexData && (
                <span className="text-[11px] font-mono text-slate-400 bg-slate-900 px-1.5 py-0.5 rounded border border-slate-800 shrink-0">
                  {log.hexData}
                </span>
              )}
            </div>
          ))
        )}
      </div>

      {/* Terminal Footer Indicator */}
      <div className="px-3 py-1 bg-bench-950 border-t border-slate-900 flex items-center justify-between text-[10px] text-slate-500">
        <span>Baud: 115200 8N1 | PIO0 Half-Duplex | Open-Drain Reset</span>
        <span className="flex items-center gap-1 text-emerald-400">
          <ArrowDown className="w-3 h-3 animate-bounce" /> Auto-Scroll Ativo
        </span>
      </div>
    </div>
  );
};
