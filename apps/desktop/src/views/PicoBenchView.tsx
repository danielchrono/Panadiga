import React, { useState } from 'react';
import { usePicoBenchViewModel } from '@/viewmodels/usePicoBenchViewModel';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { TerminalView } from '@/components/TerminalView';
import { ColdBreakinMode } from '@/models';
import {
  Radio,
  Zap,
  Cpu,
  RefreshCw,
  Terminal,
  Play,
  Send,
  Sliders,
  CheckCircle2,
  AlertTriangle,
  RotateCcw,
  Sparkles,
  Server,
  Copy,
  Check,
} from 'lucide-react';

export const PicoBenchView: React.FC = () => {
  const [copied, setCopied] = useState<boolean>(false);
  const {
    deviceStatus,
    availablePorts,
    selectedPort,
    setSelectedPort,
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
    changeBreakinMode,
    toggleConnection,
    injectRamLoaderAndErase,
    sendCommand,
    clearLogs,
    refreshStatus,
  } = usePicoBenchViewModel();

  const breakinModes: Array<{
    mode: ColdBreakinMode;
    title: string;
    description: string;
    timing: string;
    isRecommended?: boolean;
  }> = [
    {
      mode: 0,
      title: 'Modo 0: EXTRG High (Pre-Only)',
      description: 'EXTRG Idle LOW → Active HIGH na liberação de reset. 32 clocks durante reset + 200µs settle.',
      timing: 'Pre-Reset: 32 Clks | Post-Reset: 200µs delay',
    },
    {
      mode: 1,
      title: 'Modo 1: EXTRG Low (Pre-Only)',
      description: 'EXTRG Idle HIGH → Active LOW (gatilho invertido). 32 clocks durante reset.',
      timing: 'Pre-Reset: 32 Clks | Post-Reset: 200µs delay',
    },
    {
      mode: 2,
      title: 'Modo 2: EXTRG High (Pre + Post Burst)',
      description: 'Padrão recomendado para AM34/UniPhier. 32 clocks no reset + burst imediato de 32 clocks pós-reset.',
      timing: 'Pre: 32 Clks | Post: 32 Clks imediatos',
      isRecommended: true,
    },
    {
      mode: 3,
      title: 'Modo 3: EXTRG Low (Pre + Post Burst)',
      description: 'Gatilho invertido com burst duplo pré e pós-reset.',
      timing: 'Pre: 32 Clks | Post: 32 Clks imediatos',
    },
  ];

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
              <p className="text-[11px] text-slate-300 mt-0.5">
                Executado em {eraseResult.elapsed_ms}ms via DWire32A OCD. NVRAM limpa para 0xFF (65.536 bytes). DMR-EH55 pronto para boot.
              </p>
            </div>
          </div>
          <StatusBadge variant="success" label="HARDWARE RESTAURADO" />
        </div>
      )}

      {/* Top Bar: Connection, Port Selection & 1-Click In-Circuit Clean */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
        <div className="flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
            <Radio className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-base font-bold text-slate-100">
                Raspberry Pi Pico (RP2040) DWire32A Bus Master
              </h2>
              {deviceStatus?.connected ? (
                <StatusBadge variant="success" label="CONECTADO" />
              ) : (
                <StatusBadge variant="error" label="DESCONECTADO" />
              )}
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Matsushita AM34 / UniPhier Single-Wire OCD Host Controller via PIO0 State Machine
            </p>
          </div>
        </div>

        {/* Port Picker & In-Circuit Action */}
        <div className="flex flex-wrap items-center gap-3 shrink-0">
          <select
            value={selectedPort}
            onChange={e => setSelectedPort(e.target.value)}
            className="px-3 py-2 rounded-xl bg-slate-950 border border-slate-800 text-xs text-slate-200 focus:outline-none focus:border-cyan-500 font-mono"
          >
            {availablePorts.map(p => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </select>

          <button
            onClick={toggleConnection}
            className={`px-4 py-2 rounded-xl font-bold text-xs transition-colors ${
              deviceStatus?.connected
                ? 'bg-slate-800 hover:bg-slate-750 text-slate-300 border border-slate-700'
                : 'bg-cyan-500 hover:bg-cyan-400 text-slate-950 shadow-lg shadow-cyan-950/40'
            }`}
          >
            {deviceStatus?.connected ? 'Desconectar' : 'Conectar Serial'}
          </button>

          <button
            onClick={injectRamLoaderAndErase}
            disabled={isErasing || !deviceStatus?.connected}
            className="px-5 py-2 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-slate-950 font-bold text-xs flex items-center gap-2 shadow-xl shadow-emerald-950/50 transition-all disabled:opacity-50"
          >
            {isErasing ? (
              <>
                <RefreshCw className="w-4 h-4 animate-spin" />
                <span>Injetando no AM34...</span>
              </>
            ) : (
              <>
                <Sparkles className="w-4 h-4" />
                <span>Injetar RAM Loader & Limpar NVRAM</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Bench Telemetry Instruments */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Tensão VDD (ADC0 / GP26)"
          value={deviceStatus?.vdd_volts.toFixed(2) ?? '0.00'}
          unit="Volts"
          subtitle={
            (deviceStatus?.vdd_volts ?? 0) >= 2.5
              ? 'Trilho de 3.3V Estabilizado (OK)'
              : 'Subtensão / Gravador Desligado'
          }
          variant={(deviceStatus?.vdd_volts ?? 0) >= 2.5 ? 'emerald' : 'rose'}
          badge={
            (deviceStatus?.vdd_volts ?? 0) >= 2.5 ? (
              <StatusBadge variant="success" label="POWERED" />
            ) : (
              <StatusBadge variant="error" label="UNPOWERED" />
            )
          }
        />
        <MetricCard
          title="Estado do Core AM34"
          value={deviceStatus?.core_state ?? 'DISCONNECTED'}
          subtitle="OCD Trap no boot vector (0x80000000)"
          variant="cyan"
          badge={<StatusBadge variant="info" label="OCD CAPTURED" />}
        />
        <MetricCard
          title="Frequência do Barramento"
          value="1.00"
          unit="MHz"
          subtitle="RP2040 PIO SM @ 4.0 MHz (4 clocks/bit)"
          variant="slate"
        />
        <MetricCard
          title="Modo Cold Break-In"
          value={`Modo ${deviceStatus?.breakin_mode ?? 2}`}
          subtitle="Captura síncrona na subida de reset"
          variant="emerald"
          badge={<StatusBadge variant="success" label="RECOMENDADO" />}
        />
      </div>

      {/* GDB Remote Serial Protocol (RSP) Server & CPU Registers */}
      <div className="rounded-2xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 border-b border-slate-800 pb-4">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/30">
              <Server className="w-5 h-5" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h3 className="text-sm font-bold text-slate-100">
                  Servidor GDB Remote RSP (Porta TCP 3333)
                </h3>
                {gdbStatus?.running ? (
                  (gdbStatus.connected_clients ?? 0) > 0 ? (
                    <StatusBadge variant="success" label={`CLIENTE ATIVO (${gdbStatus.connected_clients})`} />
                  ) : (
                    <StatusBadge variant="warning" label="OUVINDO (3333)" />
                  )
                ) : (
                  <StatusBadge variant="neutral" label="OFFLINE" />
                )}
              </div>
              <p className="text-xs text-slate-400 mt-0.5">
                Conecte ferramentas de engenharia reversa (Ghidra Debugger, IDA Pro, gdb-multiarch) ao vivo no AM34.
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2.5 shrink-0">
            <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-[11px] text-slate-300">
              <code>(gdb) target remote 127.0.0.1:3333</code>
              <button
                onClick={() => {
                  navigator.clipboard.writeText('target remote 127.0.0.1:3333');
                  setCopied(true);
                  setTimeout(() => setCopied(false), 2000);
                }}
                className="text-slate-400 hover:text-cyan-400 p-1"
                title="Copiar comando GDB"
              >
                {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              </button>
            </div>

            <button
              onClick={() => (gdbStatus?.running ? stopGdbServer() : startGdbServer())}
              disabled={isGdbStarting}
              className={`px-4 py-2 rounded-xl font-bold text-xs flex items-center gap-2 transition-all ${
                gdbStatus?.running
                  ? 'bg-rose-500/10 hover:bg-rose-500/20 text-rose-300 border border-rose-500/30'
                  : 'bg-purple-600 hover:bg-purple-500 text-white shadow-lg shadow-purple-950/50'
              }`}
            >
              {isGdbStarting ? (
                <>
                  <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                  <span>Iniciando...</span>
                </>
              ) : gdbStatus?.running ? (
                <span>Parar Servidor GDB</span>
              ) : (
                <>
                  <Play className="w-3.5 h-3.5" />
                  <span>Iniciar Servidor GDB</span>
                </>
              )}
            </button>
          </div>
        </div>

        {/* Live AM34 Register Grid */}
        <div className="space-y-2">
          <div className="flex items-center justify-between text-xs text-slate-400">
            <span className="font-semibold text-slate-300 flex items-center gap-1.5">
              <Cpu className="w-3.5 h-3.5 text-cyan-400" />
              <span>Banco de Registradores Matsushita AM34 (Em Tempo Real)</span>
            </span>
            <span className="text-[11px] text-slate-500">
              {am34Registers?.is_halted ? '● CPU Interceptada (OCD HALT)' : '○ CPU em Execução'}
            </span>
          </div>

          <div className="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-12 gap-2 text-center text-xs">
            <div className="p-2 rounded-lg bg-slate-950 border border-cyan-500/30 col-span-2">
              <span className="text-[10px] text-cyan-400 block font-bold">PC (Program Counter)</span>
              <span className="font-mono text-cyan-200 text-xs">
                {am34Registers ? `0x${am34Registers.pc.toString(16).padStart(8, '0').toUpperCase()}` : '0x80000000'}
              </span>
            </div>
            <div className="p-2 rounded-lg bg-slate-950 border border-purple-500/30 col-span-2">
              <span className="text-[10px] text-purple-400 block font-bold">SP (Stack Pointer)</span>
              <span className="font-mono text-purple-200 text-xs">
                {am34Registers ? `0x${am34Registers.sp.toString(16).padStart(8, '0').toUpperCase()}` : '0x0003FFF0'}
              </span>
            </div>
            <div className="p-2 rounded-lg bg-slate-950 border border-slate-800 col-span-2">
              <span className="text-[10px] text-slate-400 block font-bold">PSW (Status Word)</span>
              <span className="font-mono text-slate-300 text-xs">
                {am34Registers ? `0x${am34Registers.psw.toString(16).padStart(8, '0').toUpperCase()}` : '0x00000100'}
              </span>
            </div>
            <div className="p-2 rounded-lg bg-slate-950 border border-slate-800 col-span-2">
              <span className="text-[10px] text-slate-400 block font-bold">MDR (Mult/Div)</span>
              <span className="font-mono text-slate-300 text-xs">
                {am34Registers ? `0x${am34Registers.mdr.toString(16).padStart(8, '0').toUpperCase()}` : '0x00000000'}
              </span>
            </div>
            <div className="p-2 rounded-lg bg-slate-950 border border-slate-800 col-span-2">
              <span className="text-[10px] text-slate-400 block font-bold">D0 (Data 0)</span>
              <span className="font-mono text-slate-300 text-xs">
                {am34Registers ? `0x${am34Registers.d[0].toString(16).padStart(8, '0').toUpperCase()}` : '0x00000000'}
              </span>
            </div>
            <div className="p-2 rounded-lg bg-slate-950 border border-slate-800 col-span-2">
              <span className="text-[10px] text-slate-400 block font-bold">A0 (Addr 0)</span>
              <span className="font-mono text-slate-300 text-xs">
                {am34Registers ? `0x${am34Registers.a[0].toString(16).padStart(8, '0').toUpperCase()}` : '0x00000000'}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Main Grid: Modes Selector & Live Serial Terminal */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Cold Break-In Mode Selector & Pinout */}
        <div className="lg:col-span-5 space-y-6">
          {/* Mode Selector */}
          <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
                  <Sliders className="w-4 h-4 text-cyan-400" />
                  <span>Seletor de Modo de Cold Break-In</span>
                </h3>
                <p className="text-[11px] text-slate-500 font-sans mt-0.5">
                  Garante o travamento da CPU AM34 antes da primeira instrução do bootloader.
                </p>
              </div>
            </div>

            <div className="space-y-2.5">
              {breakinModes.map(m => {
                const isSelected = deviceStatus?.breakin_mode === m.mode;
                return (
                  <div
                    key={m.mode}
                    onClick={() => changeBreakinMode(m.mode)}
                    className={`p-3.5 rounded-xl border cursor-pointer transition-all ${
                      isSelected
                        ? 'bg-cyan-500/10 border-cyan-500/50 shadow-md shadow-cyan-950/20'
                        : 'bg-slate-950 border-slate-800 hover:border-slate-700'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <span className={`text-xs font-bold ${isSelected ? 'text-cyan-400' : 'text-slate-200'}`}>
                        {m.title}
                      </span>
                      {m.isRecommended && (
                        <StatusBadge variant="success" label="RECOMENDADO" />
                      )}
                    </div>
                    <p className="text-[11px] text-slate-400 font-sans leading-relaxed">
                      {m.description}
                    </p>
                    <span className="text-[10px] text-slate-500 block mt-2 font-mono">
                      {m.timing}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>

          {/* DWire32A CN55001 Pinout Card */}
          <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-3">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
              <Zap className="w-4 h-4 text-amber-400" />
              <span>Pinagem Física DWire32A (Conector CN55001)</span>
            </h3>

            <div className="space-y-1.5 text-xs font-mono">
              <div className="flex justify-between py-1 border-b border-slate-800">
                <span className="text-slate-400">Pino 1 (VDD):</span>
                <span className="text-slate-200">GP26 (ADC0) • Sense 3.3V (1k resistor)</span>
              </div>
              <div className="flex justify-between py-1 border-b border-slate-800">
                <span className="text-slate-400">Pino 2 (EXTRG):</span>
                <span className="text-slate-200">GP2 (Pin 4) • External OCD Trigger</span>
              </div>
              <div className="flex justify-between py-1 border-b border-slate-800">
                <span className="text-slate-400">Pino 3 (SCLK):</span>
                <span className="text-slate-200">GP3 (Pin 5) • 1.0 MHz Synchronous Clock</span>
              </div>
              <div className="flex justify-between py-1 border-b border-slate-800">
                <span className="text-slate-400">Pino 4 (SDATA):</span>
                <span className="text-slate-200">GP4 (Pin 6) • Half-Duplex Data (4.7k Pull-up)</span>
              </div>
              <div className="flex justify-between py-1 border-b border-slate-800">
                <span className="text-slate-400">Pino 5 (nRESET):</span>
                <span className="text-slate-200">GP5 (Pin 7) • Open-Drain Target Reset</span>
              </div>
              <div className="flex justify-between py-1">
                <span className="text-slate-400">Pino 6 (GND):</span>
                <span className="text-slate-200">GND (Pin 38) • Terra de Referência Comum</span>
              </div>
            </div>
          </div>
        </div>

        {/* Right: Live Terminal & CDC Command Bar */}
        <div className="lg:col-span-7 flex flex-col h-[650px] space-y-3">
          <div className="flex-1 min-h-0">
            <TerminalView logs={logs} onClear={clearLogs} />
          </div>

          {/* Interactive Command Input */}
          <div className="flex items-center gap-2 p-2 rounded-xl bg-bench-900 border border-slate-800">
            <span className="text-slate-500 text-xs font-mono pl-2">{'>'}</span>
            <input
              type="text"
              placeholder="Digite comando CDC (STATUS, RESET, ADC, PEEK, POKE)..."
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
