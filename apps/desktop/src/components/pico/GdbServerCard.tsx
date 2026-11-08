import React from 'react';
import { GdbBridgeStatus, Am34RegisterState } from '@/models';
import { Am34RegisterGrid } from './Am34RegisterGrid';
import { GdbServerControls } from './GdbServerControls';
import { GdbExternalToolLaunchers, ExternalToolName } from './GdbExternalToolLaunchers';
import { GdbTelemetryInspector } from './GdbTelemetryInspector';

export interface GdbServerCardProps {
  gdbStatus: GdbBridgeStatus | null;
  registers: Am34RegisterState | null;
  isStarting: boolean;
  onStart: (listenAddr?: string) => Promise<void>;
  onStop: () => Promise<void>;
  onLaunchTool?: (tool: ExternalToolName) => Promise<void>;
}

/**
 * Clean Orchestrator Component: Manages GDB Remote Serial Protocol (RSP) subcomponents
 * strictly adhering to SRP (Controls, External Launchers, Register File, and Telemetry).
 */
export const GdbServerCard: React.FC<GdbServerCardProps> = ({
  gdbStatus,
  registers,
  isStarting,
  onStart,
  onStop,
  onLaunchTool,
}) => {
  return (
    <div className="rounded-2xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
      {/* 1. GDB Server Connection Controls & Status */}
      <GdbServerControls
        gdbStatus={gdbStatus}
        isStarting={isStarting}
        onStart={onStart}
        onStop={onStop}
      />

      {/* 2. Rapid Engineering Tool Launchers */}
      <GdbExternalToolLaunchers onLaunchTool={onLaunchTool} />

      {/* 3. AM34 Register Live Grid */}
      <Am34RegisterGrid registers={registers} />

      {/* 4. GDB Introspective Telemetry Stream Inspector */}
      <GdbTelemetryInspector isRunning={!!gdbStatus?.running} />
    </div>
  );
};
