import React, { useState } from 'react';
import { GdbPacketTelemetryEvent } from '@/models';
import { Activity, ChevronDown, ChevronUp } from 'lucide-react';

export interface GdbTelemetryInspectorProps {
  isRunning?: boolean;
  telemetry?: GdbPacketTelemetryEvent[];
}

export const GdbTelemetryInspector: React.FC<GdbTelemetryInspectorProps> = ({
  telemetry = [],
}) => {
  const [showTelemetry, setShowTelemetry] = useState<boolean>(false);

  return (
    <div className="border-t border-slate-800 pt-3">
      <button
        onClick={() => setShowTelemetry(!showTelemetry)}
        className="flex items-center justify-between w-full text-xs font-semibold text-slate-300 hover:text-cyan-400 py-1 transition-colors"
      >
        <div className="flex items-center gap-2">
          <Activity className="w-4 h-4 text-cyan-400" />
          <span>Telemetria RSP Introspectiva ({telemetry.length} pacotes no buffer circular)</span>
        </div>
        {showTelemetry ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
      </button>

      {showTelemetry && (
        <div className="mt-3 p-3 rounded-xl bg-slate-950 border border-slate-800 max-h-48 overflow-y-auto font-mono text-[11px] space-y-1.5">
          {telemetry.length === 0 ? (
            <p className="text-slate-500 italic">Nenhum pacote RSP registrado no momento.</p>
          ) : (
            telemetry
              .slice(-12)
              .reverse()
              .map((pkt, idx) => (
                <div
                  key={`${pkt.timestamp_ms}-${idx}`}
                  className="flex items-center justify-between gap-2 p-1.5 rounded bg-slate-900 border border-slate-800/60"
                >
                  <div className="flex items-center gap-2 overflow-hidden">
                    <span
                      className={`px-1.5 py-0.5 rounded text-[10px] font-bold ${
                        pkt.direction === 'INBOUND'
                          ? 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
                          : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                      }`}
                    >
                      {pkt.direction === 'INBOUND' ? 'HOST->PICO' : 'PICO->HOST'}
                    </span>
                    <span className="text-slate-400 font-bold">[{pkt.command_type}]</span>
                    <span className="text-slate-300 truncate max-w-md">{pkt.raw_packet}</span>
                  </div>
                  <div className="shrink-0">
                    {pkt.checksum_valid ? (
                      <span className="text-emerald-400 text-[10px] font-bold">✓ CSUM OK</span>
                    ) : (
                      <span className="text-rose-400 text-[10px] font-bold">✗ CSUM ERR</span>
                    )}
                  </div>
                </div>
              ))
          )}
        </div>
      )}
    </div>
  );
};
