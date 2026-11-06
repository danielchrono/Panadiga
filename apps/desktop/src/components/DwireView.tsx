import React from "react";
import { useDwireBridge } from "../viewmodels/useDwireBridge";
import { DwireBreakInMode, ExecutionStatus } from "../types/domain";
import { Terminal, AlertTriangle, CheckCircle, RefreshCw, Zap, Cpu, ArrowRight } from "lucide-react";

export const DwireView: React.FC = () => {
  const vm = useDwireBridge();

  return (
    <div className="space-y-6">
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-sm">
        <div className="flex items-center justify-between pb-4 border-b border-gray-800">
          <div>
            <h2 className="text-xl font-bold text-white flex items-center gap-2">
              <Terminal className="w-6 h-6 text-purple-400" />
              Panasonic DWire32A In-Circuit Debug Bridge (Raspberry Pi Pico)
            </h2>
            <p className="text-sm text-gray-400 mt-1">
              Interface síncrona half-duplex para controle do processador MN2DS0016 (AM34) diretamente na placa mãe VEP79138 sem dessoldar a Flash.
            </p>
          </div>
          <span className="px-3 py-1 rounded-full text-xs font-semibold bg-purple-950 text-purple-400 border border-purple-800">
            RP2040 PIO / Rota B
          </span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
          <div className="bg-gray-950 border border-gray-800 rounded-lg p-4 space-y-2">
            <h4 className="text-xs font-semibold text-gray-300 uppercase tracking-wider flex items-center gap-1.5">
              <Zap className="w-4 h-4 text-purple-400" />
              Conexões Físicas (Raspberry Pi Pico &rarr; Conector CN55001)
            </h4>
            <div className="overflow-x-auto text-xs font-mono">
              <table className="w-full text-left">
                <thead className="text-gray-500 border-b border-gray-800">
                  <tr>
                    <th className="py-1">Pico GPIO</th>
                    <th className="py-1">Sinal CN55001</th>
                    <th className="py-1">Circuito de Proteção</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-850 text-gray-300">
                  <tr>
                    <td className="py-1 text-purple-300 font-bold">GP2 (Pin 4)</td>
                    <td className="py-1">EXTRG (Pin 2)</td>
                    <td className="py-1 text-gray-400">Resistor série 100 &Omega;</td>
                  </tr>
                  <tr>
                    <td className="py-1 text-purple-300 font-bold">GP3 (Pin 5)</td>
                    <td className="py-1">SCLK (Pin 3)</td>
                    <td className="py-1 text-gray-400">Resistor série 47..100 &Omega;</td>
                  </tr>
                  <tr>
                    <td className="py-1 text-purple-300 font-bold">GP4 (Pin 6)</td>
                    <td className="py-1">SDATA (Pin 4)</td>
                    <td className="py-1 text-gray-400">100 &Omega; série + 4k7 pull-up 3.3V</td>
                  </tr>
                  <tr>
                    <td className="py-1 text-purple-300 font-bold">GP5 (Pin 7)</td>
                    <td className="py-1">nRESET (Pin 5)</td>
                    <td className="py-1 text-gray-400">Open-Drain (Drive Low / Input Hi-Z)</td>
                  </tr>
                  <tr>
                    <td className="py-1 text-purple-300 font-bold">GP26 (ADC0)</td>
                    <td className="py-1">VDD 3.3V (Pin 6)</td>
                    <td className="py-1 text-gray-400">Monitor de tensão analógico</td>
                  </tr>
                  <tr>
                    <td className="py-1 text-purple-300 font-bold">GND (Pin 38)</td>
                    <td className="py-1">GND (Pin 1)</td>
                    <td className="py-1 text-gray-400">Referência de terra comum</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div className="bg-gray-950 border border-gray-800 rounded-lg p-4 space-y-4">
            <div>
              <label className="block text-xs font-medium text-gray-300 uppercase tracking-wider mb-2">
                Modo de Sincronização Cold Break-in (Captura da CPU no Reset)
              </label>
              <select
                value={vm.mode}
                onChange={(e) => vm.setMode(Number(e.target.value) as DwireBreakInMode)}
                className="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2 text-xs text-gray-200 focus:outline-none focus:border-purple-500 font-mono"
              >
                <option value={DwireBreakInMode.MODE_0}>Modo 0: EXTRG Low &rarr; High | 32 clocks pre-reset</option>
                <option value={DwireBreakInMode.MODE_1}>Modo 1: EXTRG High &rarr; Low | 32 clocks pre-reset</option>
                <option value={DwireBreakInMode.MODE_2}>Modo 2: EXTRG Low &rarr; High | 32 clocks pre + post burst</option>
                <option value={DwireBreakInMode.MODE_3}>Modo 3: EXTRG High &rarr; Low | 32 clocks pre + post burst</option>
              </select>
            </div>

            <div className="p-3 rounded-lg bg-gray-900/60 border border-gray-800 text-xs text-gray-400 space-y-1">
              <span className="font-semibold text-gray-300 flex items-center gap-1">
                <Cpu className="w-3.5 h-3.5 text-purple-400" />
                Vantagem da Rota B:
              </span>
              <p>
                Elimina o estresse térmico na placa VEP79138 e o risco de romper trilhas ou ilhas na dessoldagem do encapsulamento TSOP-48.
              </p>
            </div>
          </div>
        </div>

        <div className="mt-6 flex items-center justify-between">
          <div className="text-xs text-gray-400">
            * O pino 14 (WP#/ACC) da Spansion deve estar em 3.3V durante operações de escrita/apagamento.
          </div>
          <div className="flex gap-3">
            {vm.stats && (
              <button
                onClick={vm.reset}
                className="px-4 py-2 rounded-lg text-xs font-semibold bg-gray-800 text-gray-300 hover:bg-gray-700 transition"
              >
                Limpar
              </button>
            )}
            <button
              onClick={vm.executeScan}
              disabled={vm.status === ExecutionStatus.RUNNING}
              className="flex items-center gap-2 px-6 py-2 rounded-lg text-sm font-semibold bg-purple-600 text-white hover:bg-purple-500 transition shadow-lg shadow-purple-600/20 disabled:opacity-50"
            >
              {vm.status === ExecutionStatus.RUNNING ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  <span>Sondando Barramento DWire32A...</span>
                </>
              ) : (
                <>
                  <Terminal className="w-4 h-4" />
                  <span>Escanear Opcodes OCD</span>
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      {vm.errorMessage && (
        <div className="bg-rose-950/50 border border-rose-800 rounded-xl p-4 flex items-start gap-3 text-rose-200">
          <AlertTriangle className="w-5 h-5 text-rose-400 shrink-0 mt-0.5" />
          <div className="text-sm">
            <p className="font-semibold">Erro de Conexão com a Sonda DWire32A</p>
            <p className="mt-1 text-xs text-rose-300 font-mono">{vm.errorMessage}</p>
          </div>
        </div>
      )}

      {vm.stats && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-sm space-y-6">
          <div className="flex items-center justify-between border-b border-gray-800 pb-4">
            <h3 className="text-lg font-bold text-white flex items-center gap-2">
              <CheckCircle className="w-5 h-5 text-purple-400" />
              Status do Barramento e Respostas Capturadas
            </h3>
            <span className="text-xs font-mono text-purple-300">
              Tensão Alvo VDD: {vm.stats.vdd_volts.toFixed(2)} V ({vm.stats.is_powered ? "Alimentado" : "Desligado"})
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Total de Opcodes Sondados</span>
              <p className="text-white font-mono font-bold text-sm mt-1">{vm.stats.total_probes}</p>
            </div>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Barramento Flutuando (0xFF)</span>
              <p className="text-gray-400 font-mono font-bold text-sm mt-1">{vm.stats.open_bus_high}</p>
            </div>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Barramento Aterrado (0x00)</span>
              <p className="text-gray-400 font-mono font-bold text-sm mt-1">{vm.stats.open_bus_low}</p>
            </div>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Respostas Ativas (Hits)</span>
              <p className="text-emerald-400 font-mono font-bold text-sm mt-1">{vm.stats.hits_detected}</p>
            </div>
          </div>

          <div>
            <h4 className="text-xs font-semibold text-gray-300 uppercase tracking-wider mb-3">
              Opcodes Candidatos Identificados no MN2DS0016
            </h4>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs font-mono border border-gray-800">
                <thead className="bg-gray-950 text-gray-400 uppercase border-b border-gray-800">
                  <tr>
                    <th className="px-4 py-2">Opcode Tx (Hex)</th>
                    <th className="px-4 py-2">Resposta Rx (Hex)</th>
                    <th className="px-4 py-2">Classificação Funcional</th>
                    <th className="px-4 py-2">Ação</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-850 bg-gray-900/50">
                  {vm.stats.candidate_hits.map((hit) => (
                    <tr key={hit.opcode_hex} className="hover:bg-gray-800/40">
                      <td className="px-4 py-2 text-purple-300 font-bold">{hit.opcode_hex}</td>
                      <td className="px-4 py-2 text-emerald-400">{hit.response_hex}</td>
                      <td className="px-4 py-2 text-gray-300">{hit.classification}</td>
                      <td className="px-4 py-2 text-blue-400">
                        <button className="flex items-center gap-1 hover:underline">
                          Injetar Payload <ArrowRight className="w-3 h-3" />
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
