import React from 'react';
import { usePicoBenchViewModel } from '@/viewmodels/usePicoBenchViewModel';
import { TerminalView } from '@/components/TerminalView';
import { StatusBadge } from '@/components/StatusBadge';
import { Button } from '@/components/ui';
import {
  PicoBenchTopBar,
  PicoBenchMetrics,
  GdbServerCard,
  ColdBreakinSelector,
  P7501PinoutCard,
  PicoDiagnosticsStudio,
} from '@/components/pico';
import { useTranslation } from '@/i18n/useTranslation';
import { CheckCircle2, Send, ShieldCheck } from 'lucide-react';

/**
 * MVVM View: Panasonic P7501 Async UART Hardware Bench Controller.
 * Composes TopBar, Telemetry Metrics, Granular Diagnostics Studio (Radix Tabs),
 * GDB Server Card, Break-In Modes / Pinout, and Live Serial Console.
 */
export const PicoBenchView: React.FC = () => {
  const { t } = useTranslation();
  const {
    ssot,
    deviceStatus,
    availablePorts,
    selectedPort,
    setSelectedPort,
    transportMode,
    modelCatalog,
    selectedModel,
    selectTargetModel,
    refreshPorts,
    refreshStatus,
    diagnosticHistory,
    latestDiagnostic,
    runningCommandId,
    diagnosticError,
    diagnosticCategory,
    setDiagnosticCategory,
    executeDiagnostic,
    clearDiagnosticHistory,
    logs,
    logContainerRef,
    isErasing,
    eraseResult,
    isBackingUp,
    backupResult,
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
    executeFirstContactBackup,
    sendCommand,
    clearLogs,
  } = usePicoBenchViewModel();

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {backupResult?.success && (
        <div className="rounded-xl bg-cyan-500/10 border border-cyan-500/40 p-4 flex items-center justify-between shadow-xl gap-3">
          <div className="flex items-center gap-3">
            <ShieldCheck className="w-5 h-5 text-cyan-400 shrink-0" />
            <div>
              <h4 className="text-xs font-bold text-cyan-300">
                BACKUP DE 1º CONTATO & DOUBLE-READ CONCLUÍDOS COM SUCESSO!
              </h4>
              <p className="text-[11px] text-slate-300 mt-0.5 font-sans">
                Modelo: {backupResult.detected_model} | NVRAM: {backupResult.nvram_bytes.toLocaleString()} bytes | SRAM: {backupResult.sram_bytes.toLocaleString()} bytes. Integridade verificada via SHA-256.
              </p>
            </div>
          </div>
          <StatusBadge variant="info" label="BACKUP SEGURO" />
        </div>
      )}

      {eraseResult?.success && (
        <div className="rounded-xl bg-emerald-500/10 border border-emerald-500/40 p-4 flex items-center justify-between shadow-xl gap-3">
          <div className="flex items-center gap-3">
            <CheckCircle2 className="w-5 h-5 text-emerald-400 shrink-0" />
            <div>
              <h4 className="text-xs font-bold text-emerald-300">
                INJEÇÃO DE RAM LOADER & APAGAMENTO IN-CIRCUIT CONCLUÍDOS!
              </h4>
              <p className="text-[11px] text-slate-300 mt-0.5 font-sans">
                Executado em {eraseResult.elapsed_ms}ms via P7501 UART (SW_UART / XRESET). NVRAM limpa para 0xFF (65.536 bytes). Alvo pronto para boot.
              </p>
            </div>
          </div>
          <StatusBadge variant="success" label="HARDWARE RESTAURADO" />
        </div>
      )}

      <PicoBenchTopBar
        deviceStatus={deviceStatus}
        availablePorts={availablePorts}
        selectedPort={selectedPort}
        transportMode={transportMode}
        modelCatalog={modelCatalog}
        selectedModel={selectedModel}
        isErasing={isErasing}
        isBackingUp={isBackingUp}
        onSelectPort={setSelectedPort}
        onSelectModel={selectTargetModel}
        onToggleConnection={toggleConnection}
        onInjectAndErase={injectRamLoaderAndErase}
        onFirstContactBackup={executeFirstContactBackup}
        onRefreshPorts={refreshPorts}
        onRefreshStatus={refreshStatus}
        onQuickPincheck={() => {
          void executeDiagnostic('PINCHECK');
        }}
      />

      <PicoBenchMetrics
        deviceStatus={deviceStatus}
        ssot={ssot}
        detectedFlashMb={latestDiagnostic?.detected_flash_mb}
      />

      <PicoDiagnosticsStudio
        ssot={ssot}
        latestResult={latestDiagnostic}
        history={diagnosticHistory}
        runningCommandId={runningCommandId}
        error={diagnosticError}
        selectedCategory={diagnosticCategory}
        onSelectCategory={setDiagnosticCategory}
        onExecute={executeDiagnostic}
        onClearHistory={clearDiagnosticHistory}
      />

      <GdbServerCard
        gdbStatus={gdbStatus}
        registers={am34Registers}
        isStarting={isGdbStarting}
        onStart={startGdbServer}
        onStop={stopGdbServer}
        onLaunchTool={launchExternalTool}
      />

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
        <div className="lg:col-span-5 flex flex-col space-y-4 lg:min-h-[720px]">
          <ColdBreakinSelector
            currentMode={deviceStatus?.breakin_mode ?? 2}
            onSelectMode={changeBreakinMode}
            breakinModes={ssot.breakin_modes}
          />
          <div className="flex-1 min-h-0 flex flex-col">
            <P7501PinoutCard ssot={ssot} />
          </div>
        </div>

        <div className="lg:col-span-7 flex flex-col space-y-3 min-h-[420px] lg:min-h-[720px]">
          <div className="flex-1 min-h-0 flex flex-col">
            <TerminalView logs={logs} onClear={clearLogs} containerRef={logContainerRef} />
          </div>

          <div className="flex items-center gap-2 p-2 rounded-xl bg-bench-900 border border-slate-800 shrink-0">
            <span className="text-slate-500 text-xs font-mono pl-2">{'>'}</span>
            <input
              type="text"
              aria-label={t('pico.title')}
              placeholder="Digite comando CDC (STATUS, RESET, ADC, PINCHECK, DUMP, GDB)..."
              value={commandInput}
              onChange={e => setCommandInput(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && sendCommand()}
              className="flex-1 bg-transparent border-none text-xs text-slate-100 placeholder-slate-500 focus:outline-none font-mono"
            />
            <Button
              type="button"
              variant="default"
              size="sm"
              onClick={sendCommand}
              className="font-bold text-xs gap-1.5"
            >
              <Send className="w-3.5 h-3.5" />
              <span>Enviar</span>
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};
