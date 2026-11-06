import React from "react";
import { useHddForensics } from "../viewmodels/useHddForensics";
import { ExecutionStatus } from "../types/domain";
import { HardDrive, AlertTriangle, CheckCircle, RefreshCw, Layers } from "lucide-react";

export const HddView: React.FC = () => {
  const vm = useHddForensics();

  return (
    <div className="space-y-6">
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-sm">
        <div className="flex items-center justify-between pb-4 border-b border-gray-800">
          <div>
            <h2 className="text-xl font-bold text-white flex items-center gap-2">
              <HardDrive className="w-6 h-6 text-cyan-400" />
              Panasonic MEIHDFS Storage & Partition Forensics
            </h2>
            <p className="text-sm text-gray-400 mt-1">
              Inspeção forense do disco rígido proprietário Matsushita/Panasonic: assinatura MBR (<code className="text-cyan-300">SH</code>),
              tabelas espelho (LBA 0 vs LBA 57344) e partições de microcódigo.
            </p>
          </div>
          <span className="px-3 py-1 rounded-full text-xs font-semibold bg-cyan-950 text-cyan-400 border border-cyan-800">
            MEIHDFS / IDE 3.5"
          </span>
        </div>

        <div className="mt-6">
          <label className="block text-xs font-medium text-gray-300 uppercase tracking-wider mb-2">
            Caminho da Imagem Raw do Disco Rígido (LBA 0 a N)
          </label>
          <input
            type="text"
            value={vm.imagePath}
            onChange={(e) => vm.setImagePath(e.target.value)}
            placeholder="firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"
            className="w-full bg-gray-950 border border-gray-700 rounded-lg px-4 py-2.5 text-sm text-gray-200 focus:outline-none focus:border-cyan-500 font-mono"
          />
        </div>

        <div className="mt-6 flex items-center justify-between">
          <div className="text-xs text-gray-400">
            * O DMR-EH55 não utiliza partição MBR do padrão IBM PC (evita que o Windows formate o disco ao plugar).
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
              onClick={vm.executeInspection}
              disabled={vm.status === ExecutionStatus.RUNNING}
              className="flex items-center gap-2 px-6 py-2 rounded-lg text-sm font-semibold bg-cyan-600 text-white hover:bg-cyan-500 transition shadow-lg shadow-cyan-600/20 disabled:opacity-50"
            >
              {vm.status === ExecutionStatus.RUNNING ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  <span>Inspecionando MBR & Partições...</span>
                </>
              ) : (
                <>
                  <HardDrive className="w-4 h-4" />
                  <span>Analisar Estrutura de Disco</span>
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
            <p className="font-semibold">Falha na Inspeção do Disco</p>
            <p className="mt-1 text-xs text-rose-300 font-mono">{vm.errorMessage}</p>
          </div>
        </div>
      )}

      {vm.report && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-sm space-y-6">
          <div className="flex items-center justify-between border-b border-gray-800 pb-4">
            <h3 className="text-lg font-bold text-white flex items-center gap-2">
              <CheckCircle className="w-5 h-5 text-cyan-400" />
              Resultado da Análise Forense MEIHDFS
            </h3>
            <span className="text-xs font-mono text-gray-400">
              {vm.report.file_size_bytes.toLocaleString()} bytes ({vm.report.total_sectors.toLocaleString()} setores LBA)
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Assinatura MBR Proprietária</span>
              <p className="text-white font-mono font-bold text-sm mt-1">
                ASCII: "{vm.report.mbr_tables.magic_ascii}" (System Storage Header)
              </p>
            </div>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Espelho MBR (Mirror)</span>
              <p className="text-emerald-400 font-mono font-bold text-sm mt-1 flex items-center gap-1">
                <CheckCircle className="w-4 h-4" /> LBA {vm.report.mbr_tables.mirror_lba} (100% Idêntico)
              </p>
            </div>
            <div className="bg-gray-950 border border-gray-800 rounded-lg p-3">
              <span className="text-xs text-gray-400">Partições Operacionais</span>
              <p className="text-cyan-400 font-mono font-bold text-sm mt-1">
                {vm.report.partitions.length} partições detectadas
              </p>
            </div>
          </div>

          <div>
            <h4 className="text-xs font-semibold text-gray-300 uppercase tracking-wider mb-3 flex items-center gap-1.5">
              <Layers className="w-4 h-4 text-cyan-400" />
              Tabela de Partições Panasonic Detectadas no Disco
            </h4>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs font-mono border border-gray-800">
                <thead className="bg-gray-950 text-gray-400 uppercase border-b border-gray-800">
                  <tr>
                    <th className="px-4 py-2">Partição</th>
                    <th className="px-4 py-2">LBA Inicial</th>
                    <th className="px-4 py-2">Offset Físico</th>
                    <th className="px-4 py-2">Tamanho</th>
                    <th className="px-4 py-2">Header Magic</th>
                    <th className="px-4 py-2">SHA-256 Hash</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-800 bg-gray-900/50">
                  {vm.report.partitions.map((part) => (
                    <tr key={part.partition_index} className="hover:bg-gray-800/40">
                      <td className="px-4 py-2 text-white font-bold">Partição #{part.partition_index}</td>
                      <td className="px-4 py-2 text-cyan-400">LBA {part.start_lba}</td>
                      <td className="px-4 py-2 text-gray-300">0x{part.byte_offset.toString(16).toUpperCase()}</td>
                      <td className="px-4 py-2 text-amber-300">{(part.size_bytes / (1024 * 1024)).toFixed(2)} MB</td>
                      <td className="px-4 py-2 text-emerald-400">{part.header_magic}</td>
                      <td className="px-4 py-2 text-gray-400 truncate max-w-xs">{part.exact_sha256}</td>
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
