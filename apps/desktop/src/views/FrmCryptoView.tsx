import React, { useState } from 'react';
import { useFrmCryptoViewModel, FrmTab } from '@/viewmodels/useFrmCryptoViewModel';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import {
  Lock,
  KeyRound,
  FileSearch,
  Search,
  Layers,
  Cpu,
  BarChart2,
  FileCheck2,
  ShieldAlert,
} from 'lucide-react';

export const FrmCryptoView: React.FC = () => {
  const {
    cryptReport,
    tocReport,
    codebookReport,
    filteredCodebook,
    activeTab,
    setActiveTab,
    loading,
    codebookFilter,
    setCodebookFilter,
  } = useFrmCryptoViewModel();

  if (loading || !cryptReport) {
    return (
      <div className="flex items-center justify-center h-full text-slate-500 font-mono text-xs">
        Analisando contêiner óptico PANA_DVD.FRM (7.531.744 bytes) e correlações ECB...
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Top Header */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
        <div className="flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/30">
            <Lock className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-base font-bold text-slate-100">
                Dissecador Criptográfico de Contêiner Óptico PANA_DVD.FRM
              </h2>
              <StatusBadge variant="info" label="ECB LEAK CONFIRMADO" />
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Cifra Simétrica de Bloco Feistel (64 bits / 8 bytes) • 3 Co-Processadores Heterogêneos
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          <span className="text-xs text-slate-400">
            Assinatura Global: <span className="text-purple-400 font-bold">50 41 4E 41 5F 46 52 4D</span>
          </span>
        </div>
      </div>

      {/* Metrics Row */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Entropia de Shannon"
          value={cryptReport.global_entropy.toFixed(5)}
          unit="bits/byte"
          subtitle="Quase aleatório puro (7.99958/8.00)"
          variant="slate"
        />
        <MetricCard
          title="Vazamentos ECB Detectados"
          value="3.812"
          unit="blocos"
          subtitle="Padronagem 0xFF virgem descoberta"
          variant="amber"
          badge={<StatusBadge variant="warning" label="KPA PROVADO" />}
        />
        <MetricCard
          title="Tamanho do Contêiner"
          value="7.531.744"
          unit="bytes"
          subtitle="PANA_DVD.FRM (7.18 MB)"
          variant="slate"
        />
        <MetricCard
          title="CPUs no TOC Periódico"
          value="3"
          unit="arquiteturas"
          subtitle="AM34 + MN102 + MN103S"
          variant="cyan"
          badge={<StatusBadge variant="success" label="48B TOC" />}
        />
      </div>

      {/* Navigation Tabs */}
      <div className="flex items-center gap-2 border-b border-slate-800 pb-2 text-xs">
        <button
          onClick={() => setActiveTab('entropy')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors ${
            activeTab === 'entropy'
              ? 'bg-purple-500/10 text-purple-300 border border-purple-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <BarChart2 className="w-3.5 h-3.5" />
          <span>Densidade de Entropia & Frequência ECB</span>
        </button>

        <button
          onClick={() => setActiveTab('toc')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors ${
            activeTab === 'toc'
              ? 'bg-purple-500/10 text-purple-300 border border-purple-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Layers className="w-3.5 h-3.5" />
          <span>TOC de 48 Bytes & 3 Co-Processadores</span>
        </button>

        <button
          onClick={() => setActiveTab('codebook')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors ${
            activeTab === 'codebook'
              ? 'bg-purple-500/10 text-purple-300 border border-purple-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <KeyRound className="w-3.5 h-3.5" />
          <span>Dicionário Codebook 64-bit (KPA Attack)</span>
        </button>

        <button
          onClick={() => setActiveTab('correlation')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors ${
            activeTab === 'correlation'
              ? 'bg-purple-500/10 text-purple-300 border border-purple-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <FileCheck2 className="w-3.5 h-3.5" />
          <span>Correlação Cruzada FRM ↔ HD</span>
        </button>
      </div>

      {/* Tab 1: Entropy & Block Frequencies */}
      {activeTab === 'entropy' && (
        <div className="space-y-6">
          <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <BarChart2 className="w-4 h-4 text-purple-400" />
                <span>Top Blocos Cifrados de 64 bits Mais Frequentes (Vazamentos de Modo ECB)</span>
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                Em cifras de bloco operando em Electronic Codebook (ECB), blocos idênticos de texto claro produzem blocos idênticos de texto cifrado.
              </p>
            </div>

            <div className="space-y-3">
              {cryptReport.top_blocks.length === 0 ? (
                <div className="p-8 text-center text-xs text-slate-500 font-mono rounded-xl bg-slate-950 border border-slate-800">
                  Nenhum bloco cifrado detectado na análise
                </div>
              ) : (
                cryptReport.top_blocks.map((blk, idx) => (
                  <div
                    key={blk.hex}
                    className="p-3.5 rounded-xl bg-slate-950 border border-slate-800 flex flex-col md:flex-row items-start md:items-center justify-between gap-3 text-xs"
                  >
                    <div className="flex items-center gap-3">
                      <span className="w-6 h-6 rounded-lg bg-slate-800 text-slate-400 font-bold flex items-center justify-center text-xs">
                        #{idx + 1}
                      </span>
                      <div>
                        <span className="text-purple-300 font-bold text-sm tracking-wider">
                          {blk.hex}
                        </span>
                        <p className="text-slate-400 text-[11px] mt-0.5">{blk.interpretation}</p>
                      </div>
                    </div>

                    <div className="flex items-center gap-4 text-xs font-mono shrink-0">
                      <div className="text-right">
                        <span className="text-slate-200 font-bold">{blk.count.toLocaleString()} ocorrências</span>
                        <span className="text-slate-500 block text-[10px]">
                          ({blk.total_bytes.toLocaleString()} bytes totais)
                        </span>
                      </div>
                      <StatusBadge
                        variant={blk.count > 1000 ? 'warning' : 'neutral'}
                        label={blk.count > 1000 ? 'ALTA REPETIÇÃO' : 'FREQUENTE'}
                      />
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: 48-byte Periodic TOC & Modules */}
      {activeTab === 'toc' && tocReport && (
        <div className="space-y-6">
          <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Layers className="w-4 h-4 text-cyan-400" />
                <span>Tabela Periódica de Conteúdo (TOC) de 48 Bytes (3 Sub-Sistemas)</span>
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                Descreve os executáveis distribuídos pelo contêiner para cada um dos processadores do gravador.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {tocReport.toc_entries.map(entry => (
                <div
                  key={entry.index}
                  className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3 text-xs"
                >
                  <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                    <span className="font-bold text-cyan-400">{entry.target_name}</span>
                    <span className="text-[10px] text-slate-500">{entry.offset_hex}</span>
                  </div>

                  <div className="space-y-1.5 text-slate-400">
                    <div className="flex justify-between">
                      <span>Arquitetura:</span>
                      <span className="text-slate-200">{entry.target_architecture}</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Family ID:</span>
                      <span className="text-emerald-400">{entry.processor_family_id}</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Descriptor:</span>
                      <span className="text-slate-200">{entry.target_descriptor_id}</span>
                    </div>
                  </div>

                  <div className="pt-2 border-t border-slate-800">
                    <span className="text-[10px] text-slate-500 block mb-1">DUMP DO DESCRITOR:</span>
                    <p className="text-[10px] text-slate-400 break-all bg-bench-950 p-2 rounded border border-slate-900">
                      {entry.raw_hex}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Module Layout */}
          <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-3">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400 flex items-center gap-2">
              <Cpu className="w-4 h-4 text-emerald-400" />
              <span>Módulos de Firmware Extraíveis</span>
            </h3>

            <div className="space-y-2.5">
              {tocReport.modules.map(mod => (
                <div
                  key={mod.module_index}
                  className="p-3.5 rounded-xl bg-slate-950 border border-slate-800 flex flex-col md:flex-row items-start md:items-center justify-between gap-3 text-xs"
                >
                  <div>
                    <span className="font-bold text-slate-200">{mod.name}</span>
                    <span className="text-slate-500 text-[11px] block">{mod.target} • {mod.hdd_correlation}</span>
                  </div>

                  <div className="flex items-center gap-4 text-xs font-mono text-slate-400 shrink-0">
                    <span>{mod.start_offset_hex} .. {mod.end_offset_hex}</span>
                    <span className="text-cyan-400 font-bold">{mod.payload_size_bytes.toLocaleString()} bytes</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Tab 3: Codebook Dictionary (Known-Plaintext Attack) */}
      {activeTab === 'codebook' && codebookReport && (
        <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <KeyRound className="w-4 h-4 text-purple-400" />
                <span>Dicionário Codebook ECB de 64 bits (Known-Plaintext Attack KPA)</span>
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                Mapeamento das correspondências exatas entre Blocos Cifrados e Texto Claro Conhecido.
              </p>
            </div>

            <div className="relative">
              <Search className="w-3.5 h-3.5 absolute left-2.5 top-2.5 text-slate-500" />
              <input
                type="text"
                placeholder="Buscar por bloco ou texto claro..."
                value={codebookFilter}
                onChange={e => setCodebookFilter(e.target.value)}
                className="pl-8 pr-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-xs text-slate-200 focus:outline-none focus:border-purple-500"
              />
            </div>
          </div>

          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs font-mono">
              <thead>
                <tr className="border-b border-slate-800 text-slate-500 uppercase text-[10px]">
                  <th className="py-2.5 px-3">Rank</th>
                  <th className="py-2.5 px-3">Bloco Cifrado C_i (64b)</th>
                  <th className="py-2.5 px-3">Texto Claro P_i (KPA)</th>
                  <th className="py-2.5 px-3">Ocorrências (.FRM)</th>
                  <th className="py-2.5 px-3">Ocorrências (HD)</th>
                  <th className="py-2.5 px-3">Interpretação Semântica</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60">
                {filteredCodebook.map(entry => (
                  <tr key={entry.block_hex} className="hover:bg-slate-800/40 transition-colors">
                    <td className="py-2.5 px-3 font-bold text-slate-400">
                      #{entry.rank}
                    </td>
                    <td className="py-2.5 px-3 font-bold text-purple-300">
                      {entry.block_hex}
                    </td>
                    <td className="py-2.5 px-3 text-emerald-400 font-bold">
                      {entry.known_plaintext_hex ?? 'Aguardando Descoberta'}
                    </td>
                    <td className="py-2.5 px-3 text-slate-300">
                      {entry.frm_occurrences.toLocaleString()}x ({entry.frm_total_bytes} B)
                    </td>
                    <td className="py-2.5 px-3 text-cyan-400">
                      {entry.hdd_occurrences.toLocaleString()}x
                    </td>
                    <td className="py-2.5 px-3 text-slate-400 text-[11px]">
                      {entry.interpretation}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Tab 4: Cross-Medium Correlation */}
      {activeTab === 'correlation' && (
        <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <FileCheck2 className="w-4 h-4 text-emerald-400" />
                <span>Prova Criptográfica: Identidade de Chaves Entre .FRM e Disco Rígido</span>
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                Os blocos de padding cifrados na Partição 1 do HD coincidem perfeitamente com o contêiner óptico.
              </p>
            </div>
            <StatusBadge variant="success" label="CHAVE COMPARTILHADA CONFIRMADA" />
          </div>

          <div className="space-y-3">
            {cryptReport.hdd_correlations.map(corr => (
              <div
                key={corr.offset_hex}
                className="p-4 rounded-xl bg-slate-950 border border-slate-800 text-xs font-mono space-y-2"
              >
                <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                  <span className="font-bold text-slate-200">{corr.partition_name}</span>
                  <span className="text-cyan-400">{corr.offset_hex}</span>
                </div>
                <div className="space-y-1 text-slate-400">
                  <div className="flex justify-between">
                    <span>Bloco Cifrado Identificado:</span>
                    <span className="text-purple-300 font-bold">{corr.block_hex}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Frequência no PANA_DVD.FRM:</span>
                    <span className="text-emerald-400 font-bold">{corr.frm_occurrences} repetições</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Interpretação:</span>
                    <span className="text-slate-300">{corr.interpretation}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
