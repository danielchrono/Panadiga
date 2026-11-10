import React from 'react';
import { usePicoBenchViewModel } from '@/viewmodels/usePicoBenchViewModel';
import { TerminalView } from '@/components/TerminalView';
import { StatusBadge } from '@/components/StatusBadge';
import {
  PicoBenchTopBar,
  PicoBenchMetrics,
  GdbServerCard,
  ColdBreakinSelector,
  DwirePinoutCard,
} from '@/components/pico';
import { useTranslation } from '@/i18n/useTranslation';
import { CheckCircle2, Send } from 'lucide-react';

/**
 * MVVM View: Panasonic DWire32A Hardware Bench Controller.
 * Cleanly composes presentation components and coordinates with usePicoBenchViewModel.
 * Equalized dual columns with CSS Grid items-stretch and matching heights.
 */
export const PicoBenchView: React.FC = () => {
  const { t } = useTranslation();
  const {
    deviceStatus,
    availablePorts,
    selectedPort,
    setSelectedPort,
    modelCatalog,
    selectedModel,
    selectTargetModel,
    logs,
    isErasing,
    eraseResult,
    commandInput,
    setCommandInput,
    gdbStatus,
    am34Registers,
    isGdbStarting,
    startGdbServer,
    stopGdbServer,
    launchExternalTool,
    changeBreakinMode,
    toggleConnection,
    injectRamLoaderAndErase,
    sendCommand,
    clearLogs,
  } = usePicoBenchViewModel();

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Erase Success Notification */}
      {eraseResult?.success && (
        <div className="rounded-xl bg-emerald-500/10 border border-emerald-500/40 p-4 flex items-center justify-between shadow-xl">
          <div className="flex items-center gap-3">
            <CheckCircle2 className="w-5 h-5 text-emerald-400 shrink-0" />
            <div>
              <h4 className="text-xs font-bold text-emerald-300">
                INJEÇÃO DE RAM LOADER & APAGAMENTO IN-CIRCUIT CONCLUÍDOS!
              </h4>
              <p className="text-[11px] text-slate-300 mt-0.5 font-sans">
                Executado em {eraseResult.elapsed_ms}ms via DWire32A OCD. NVRAM limpa para 0xFF (65.536 bytes). Alvo pronto para boot.
              </p>
            </div>
          </div>
          <StatusBadge variant="success" label="HARDWARE RESTAURADO" />
        </div>
      )}

      {/* Top Bar: Port Selector, Model Selector, Connect/Disconnect, Flash UF2 & In-Circuit Action */}
      <PicoBenchTopBar
        deviceStatus={deviceStatus}
        availablePorts={availablePorts}
        selectedPort={selectedPort}
        modelCatalog={modelCatalog}
        selectedModel={selectedModel}
        isErasing={isErasing}
        onSelectPort={setSelectedPort}
        onSelectModel={selectTargetModel}
        onToggleConnection={toggleConnection}
        onInjectAndErase={injectRamLoaderAndErase}
      />

      {/* Telemetry Instruments Grid */}
      <PicoBenchMetrics deviceStatus={deviceStatus} />

      {/* GDB Remote RSP Server & AM34 Live Register File */}
      <GdbServerCard
        gdbStatus={gdbStatus}
        registers={am34Registers}
        isStarting={isGdbStarting}
        onStart={startGdbServer}
        onStop={stopGdbServer}
        onLaunchTool={launchExternalTool}
      />

      {/* Main Grid: Timing Modes & Live Serial Terminal (Balanced Columns) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
        {/* Left Column: Timing Profile Selector & Pinout Guide */}
        <div className="lg:col-span-5 flex flex-col space-y-6 h-[740px]">
          <ColdBreakinSelector
            currentMode={deviceStatus?.breakin_mode ?? 2}
            onSelectMode={changeBreakinMode}
          />
          <div className="flex-1 min-h-0">
            <DwirePinoutCard />
          </div>
        </div>

        {/* Right Column: Live Terminal & CDC Command Bar */}
        <div className="lg:col-span-7 flex flex-col space-y-3 h-[740px]">
          <div className="flex-1 min-h-0 flex flex-col">
            <TerminalView logs={logs} onClear={clearLogs} />
          </div>

          {/* Interactive Command Input */}
          <div className="flex items-center gap-2 p-2 rounded-xl bg-bench-900 border border-slate-800 shrink-0">
            <span className="text-slate-500 text-xs font-mono pl-2">{'>'}</span>
            <input
              type="text"
              placeholder="Digite comando CDC (STATUS, RESET, ADC, GDB, PEEK, POKE)..."
              value={commandInput}
              onChange={e => setCommandInput(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && sendCommand()}
              className="flex-1 bg-transparent border-none text-xs text-slate-100 placeholder-slate-500 focus:outline-none font-mono"
            />
            <button
              onClick={sendCommand}
              className="px-3 py-1.5 rounded-lg bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold text-xs flex items-center gap-1.5 transition-colors"
            >
              <Send className="w-3.5 h-3.5" />
              <span>Enviar</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
