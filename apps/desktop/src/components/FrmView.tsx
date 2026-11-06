import React from "react";
import { useFrmAnalyzer } from "../viewmodels/useFrmAnalyzer";
import { ExecutionStatus } from "../types/domain";
import { Key, AlertTriangle, CheckCircle, RefreshCw, Lock, AlertCircle } from "lucide-react";

export const FrmView: React.FC = () => {
  const vm = useFrmAnalyzer();

  return (
    <div className="space-y-6">
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-sm">
        <div className="flex items-center justify-between pb-4 border-b border-gray-800">
          <div>
            <h2 className="text-xl font-bold text-white flex items-center gap-2">
              <Key className="w-6 h-6 text-amber-400" />
              PANA_DVD.FRM Container & Cifra Feistel ECB
            </h2>
            <p className="text-sm text-gray-400 mt-1">
              Desmistificação do arquivo de atualização óptica: por que ele <strong className="text-rose-400">NUNCA</strong> pode ser gravado diretamente na memória Flash Spansion.
            </p>
          </div>
          <span className="px-3 py-1 rounded-full text-xs font-semibold bg-amber-950 text-amber-400 border border-amber-800">
            64-Bit Feistel ECB
          </span>
        </div>

        <div className="mt-6 bg-rose-950/20 border border-rose-900/40 rounded-lg p-4 text-xs text-rose-300 space-y-2">
          <p className="font-bold flex items-center gap-1.5 text-rose-200">
            <AlertCircle className="w-4 h-4 text-rose-400" />
            Por que PANA_DVD.FRM NÃO pode ser gravado diretamente na Flash Spansion:
          </p>
          <ul className="list-disc list-inside space-y-1 text-gray-300">
            <li><strong>Discrepância de Tamanho:</strong> O contêiner possui <code className="text-white">7.531.744 bytes (7,18 MB)</code>, enquanto a Flash Spansion S29JL064H exige exatamente <code className="text-white">8.388.608 bytes (8,00 MB)</code>.</li>
            <li><strong>Criptografia Proprietária:</strong> O payload é cifrado com uma cifra Feistel de 64 bits em modo ECB com 16 rodadas. Gravar na Flash causaria tijolamento imediato (o processador lê código de máquina MN103S puro, não dados cifrados).</li>
            <li><strong>Arquitetura Multi-Processador:</strong> O FRM contém 3 módulos distintos (Sub-sistema OSD, DSP MN102 do Drive de DVD e Sistema Operacional do AM34). O bootloader da Spansion não está presente no pacote de atualização.</li>
          </ul>
        </div>

        <div className="mt-6">
          <label className="block text-xs font-medium text-gray-300 uppercase tracking-wider mb-2">
            Caminho do Arquivo de Firmware Óptico (PANA_DVD.FRM)
          </label>
          <input
            type="text"
            value={vm.frmPath}
            onChange={(e) => vm.setFrmPath(e.target.value)}
            placeholder="firmware_and_images/PANA_DVD.FRM"
            className="w-full bg-gray-950 border border-gray-700 rounded-lg px-4 py-2.5 text-sm text-gray-200 focus:outline-none focus:border-amber-500 font-mono"
          />
        </div>

        <div className="mt-6 flex items-center justify-between">
          <div className="text-xs text-gray-400">
            * O modo ECB causa o clássico "ECB leak" onde repetições de blocos idênticos revelam estruturas do código original.
          </div>
          <div className="flex gap-3">
            {vm.report && (
              <button
                onClick={vm.reset}
                className="px-4 py-2 rounded-lg text-xs font-semibold bg-gray-800 text-gray-300 hover:bg-gray-700 transition"
              >
                Limpar
              </button>
            )}
            <button
              onClick={vm.executeAnalysis}
              disabled={vm.status === ExecutionStatus.RUNNING}
              className="flex items-center gap-2 px-6 py-2 rounded-lg text-sm font-semibold bg-amber-600 text-white hover:bg-amber-500 transition shadow-lg shadow-amber-600/20 disabled:opacity-50"
            >
              {vm.status === ExecutionStatus.RUNNING ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  <span>Analisando Criptografia...</span>
                </>
              ) : (
                <>
                  <Lock className="w-4 h-4" />
                  <span>Executar Criptoanálise Forense</span>
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
            <p className="font-semibold">Erro na Criptoanálise</p>
            <p className="mt-1 text-xs text-rose-300 font-mono">{vm.errorMessage}</p>
          </div>
        </div>
      )}

      {vm.report && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-sm space-y-6">
          <div className="flex items-center justify-between border-b border-gray-800 pb-4">
            <h3 className="text-lg font-bold text-white flex items-center gap-2">
              <CheckCircle className="w-5 h-5 text-amber-400" />
              Resultado da Criptoanálise do Pacote FRM
            </h3>
            <span className="text-xs font-mono text-gray-400">
              Entropia Global de Shannon: {vm.report.global_entropy.toFixed(5)} / 8.00000
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Confirmação de Cifra ECB</span>
              <p className="text-emerald-400 font-bold text-sm mt-1 flex items-center gap-1">
                <CheckCircle className="w-4 h-4" /> Vazamento de Padrão Confirmado
              </p>
            </div>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Blocos Virgin Flash (0xFF*8)</span>
              <p className="text-amber-400 font-mono font-bold text-sm mt-1">
                {vm.report.virgin_flash_block_count.toLocaleString()} ocorrências
              </p>
            </div>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Blocos Nulos (0x00*8)</span>
              <p className="text-blue-400 font-mono font-bold text-sm mt-1">
                {vm.report.null_block_count.toLocaleString()} ocorrências
              </p>
            </div>
          </div>

          <div>
            <h4 className="text-xs font-semibold text-gray-300 uppercase tracking-wider mb-3">
              Dicionário Criptográfico de Blocos Cifrados Frequentes (Codebook)
            </h4>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs font-mono border border-gray-800">
                <thead className="bg-gray-950 text-gray-400 uppercase border-b border-gray-800">
                  <tr>
                    <th className="px-4 py-2">Bloco Cifrado (8 Bytes Hex)</th>
                    <th className="px-4 py-2">Frequência</th>
                    <th className="px-4 py-2">Plaintext Identificado / Significado Forense</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-800 bg-gray-900/50">
                  {vm.report.top_blocks.map((block) => (
                    <tr key={block.hex} className="hover:bg-gray-800/40">
                      <td className="px-4 py-2 text-amber-300 font-bold">{block.hex}</td>
                      <td className="px-4 py-2 text-white">{block.count.toLocaleString()}</td>
                      <td className="px-4 py-2 text-emerald-400">{block.known_plaintext || "Padrão de Código Replicado"}</td>
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
