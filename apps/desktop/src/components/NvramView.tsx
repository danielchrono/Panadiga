import React from "react";
import { useNvramCleaner } from "../viewmodels/useNvramCleaner";
import { ExecutionStatus } from "../types/domain";
import { ShieldCheck, AlertTriangle, CheckCircle, RefreshCw, FileText } from "lucide-react";

export const NvramView: React.FC = () => {
  const vm = useNvramCleaner();

  return (
    <div className="space-y-6">
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-sm">
        <div className="flex items-center justify-between pb-4 border-b border-gray-800">
          <div>
            <h2 className="text-xl font-bold text-white flex items-center gap-2">
              <ShieldCheck className="w-6 h-6 text-emerald-400" />
              Spansion S29JL064H NVRAM Sanitizer (Solução Definitiva U99)
            </h2>
            <p className="text-sm text-gray-400 mt-1">
              Sanitização cirúrgica dos 64 KB de emulação EEPROM (<code className="text-blue-300">0x007F0000..0x007FFFFF</code>)
              com preservação bit-a-bit garantida de 100% do Bootloader e OS (<code className="text-blue-300">8.323.072 bytes</code>).
            </p>
          </div>
          <span className="px-3 py-1 rounded-full text-xs font-semibold bg-emerald-950 text-emerald-400 border border-emerald-800">
            TSOP-48 / 8 MB NOR Flash
          </span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
          <div>
            <label className="block text-xs font-medium text-gray-300 uppercase tracking-wider mb-2">
              Arquivo Binário de Entrada (Dump da Flash Spansion)
            </label>
            <input
              type="text"
              value={vm.inputPath}
              onChange={(e) => vm.setInputPath(e.target.value)}
              placeholder="firmware_and_images/spansion_s29jl064h_dump.bin"
              className="w-full bg-gray-950 border border-gray-700 rounded-lg px-4 py-2.5 text-sm text-gray-200 focus:outline-none focus:border-blue-500 font-mono"
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-gray-300 uppercase tracking-wider mb-2">
              Arquivo Limpo de Saída (Opcional)
            </label>
            <input
              type="text"
              value={vm.outputPath}
              onChange={(e) => vm.setOutputPath(e.target.value)}
              placeholder="Gerado automaticamente como *_CLEAN.bin"
              className="w-full bg-gray-950 border border-gray-700 rounded-lg px-4 py-2.5 text-sm text-gray-200 focus:outline-none focus:border-blue-500 font-mono"
            />
          </div>
        </div>

        <div className="mt-6 flex items-center justify-between">
          <div className="text-xs text-gray-400">
            * Nenhum dump externo é necessário. 99,22% da Flash original é mantida intacta.
          </div>
          <div className="flex gap-3">
            {vm.report && (
              <button
                onClick={vm.reset}
                className="px-4 py-2 rounded-lg text-xs font-semibold bg-gray-800 text-gray-300 hover:bg-gray-700 transition"
              >
                Limpar Resultado
              </button>
            )}
            <button
              onClick={vm.executeSanitization}
              disabled={vm.status === ExecutionStatus.RUNNING}
              className="flex items-center gap-2 px-6 py-2 rounded-lg text-sm font-semibold bg-blue-600 text-white hover:bg-blue-500 transition shadow-lg shadow-blue-600/20 disabled:opacity-50"
            >
              {vm.status === ExecutionStatus.RUNNING ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  <span>Sanitizando Flash...</span>
                </>
              ) : (
                <>
                  <ShieldCheck className="w-4 h-4" />
                  <span>Sanitizar e Gerar Binário</span>
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
            <p className="font-semibold">Erro na Execução da Sanitização</p>
            <p className="mt-1 text-xs text-rose-300 font-mono">{vm.errorMessage}</p>
          </div>
        </div>
      )}

      {vm.report && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-sm space-y-6">
          <div className="flex items-center justify-between border-b border-gray-800 pb-4">
            <h3 className="text-lg font-bold text-white flex items-center gap-2">
              <CheckCircle className="w-5 h-5 text-emerald-400" />
              Relatório Forense de Sanitização da Flash
            </h3>
            <span className="text-xs font-mono text-gray-400">
              {vm.report.total_flash_bytes.toLocaleString()} bytes verificados
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Bootloader & OS</span>
              <p className="text-emerald-400 font-bold text-sm mt-1 flex items-center gap-1">
                <CheckCircle className="w-4 h-4" /> 100% Imutável
              </p>
            </div>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Bytes Sanitizados para 0xFF</span>
              <p className="text-white font-mono font-bold text-sm mt-1">
                {vm.report.nvram_non_ff_bytes.toLocaleString()} / 65,536 bytes
              </p>
            </div>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Faixa de Memória Sanitizada</span>
              <p className="text-blue-400 font-mono text-xs mt-1">
                0x007F0000 - 0x007FFFFF
              </p>
            </div>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Backup NVRAM Gerado</span>
              <p className="text-amber-400 font-mono text-xs mt-1 truncate">
                {vm.report.backup_file}
              </p>
            </div>
          </div>

          <div className="space-y-2">
            <h4 className="text-xs font-semibold text-gray-300 uppercase tracking-wider">
              Assinaturas Criptográficas (Hashes de Integridade)
            </h4>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3 space-y-2 font-mono text-xs">
              <div className="flex justify-between">
                <span className="text-gray-400">Original MD5:</span>
                <span className="text-gray-300">{vm.report.original_md5}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Original SHA-256:</span>
                <span className="text-gray-300 truncate max-w-lg">{vm.report.original_sha256}</span>
              </div>
              <div className="flex justify-between pt-2 border-t border-gray-800">
                <span className="text-emerald-400 font-bold">Clean MD5:</span>
                <span className="text-white">{vm.report.clean_md5}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-emerald-400 font-bold">Clean SHA-256:</span>
                <span className="text-white truncate max-w-lg">{vm.report.clean_sha256}</span>
              </div>
            </div>
          </div>

          <div>
            <h4 className="text-xs font-semibold text-gray-300 uppercase tracking-wider mb-3">
              Auditoria dos 8 Micro-Setores (Top Boot SA134..SA141 - 8 KB cada)
            </h4>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs font-mono border border-gray-800">
                <thead className="bg-gray-950 text-gray-400 uppercase border-b border-gray-800">
                  <tr>
                    <th className="px-4 py-2">Setor</th>
                    <th className="px-4 py-2">Endereço Físico</th>
                    <th className="px-4 py-2">Bytes Sujos (Não-0xFF)</th>
                    <th className="px-4 py-2">Ocupação</th>
                    <th className="px-4 py-2">Header Hex</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-800 bg-gray-900/50">
                  {vm.report.microsectors.map((sec) => (
                    <tr key={sec.sector_name} className="hover:bg-gray-800/40">
                      <td className="px-4 py-2 text-white font-bold">{sec.sector_name}</td>
                      <td className="px-4 py-2 text-blue-400">0x{sec.offset.toString(16).toUpperCase()}</td>
                      <td className="px-4 py-2 text-amber-300">{sec.active_bytes} / {sec.total_bytes}</td>
                      <td className="px-4 py-2 text-gray-300">
                        {((sec.active_bytes / sec.total_bytes) * 100).toFixed(1)}%
                      </td>
                      <td className="px-4 py-2 text-gray-400">{sec.header_hex}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          <div className="bg-blue-950/30 border border-blue-800/60 rounded-lg p-4 text-xs text-blue-200 flex items-start gap-3">
            <FileText className="w-5 h-5 text-blue-400 shrink-0 mt-0.5" />
            <div>
              <p className="font-bold">Instruções para Gravação em Gravador Universal (XGecu T48 / TL866II / SVOD4):</p>
              <p className="mt-1">
                Grave o arquivo limpo gerado diretamente na Spansion S29JL064H. Ao ser religado, o processador Panasonic
                AM34 detectará que os micro-setores SA134..SA141 estão em estado virgem (<code className="text-white">0xFF</code>)
                e executará a recriação limpa das tabelas de calibração de fábrica, restaurando o aparelho do loop U99.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
