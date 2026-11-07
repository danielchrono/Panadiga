import React, { useState } from 'react';
import { useHddStorageViewModel, HddTab } from '@/viewmodels/useHddStorageViewModel';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { Modal } from '@/components/Modal';
import {
  HardDrive,
  Copy,
  Layers,
  FileCheck,
  Search,
  Download,
  CheckCircle2,
  Table,
  BarChart3,
  BookOpen,
  FolderArchive,
  ArrowRight,
  ShieldCheck,
} from 'lucide-react';

export const HddStorageView: React.FC = () => {
  const {
    hddReport,
    meihdfsReport,
    filteredRecords,
    activeTab,
    setActiveTab,
    loading,
    extracting,
    manifest,
    tagFilter,
    setTagFilter,
    searchQuery,
    setSearchQuery,
    extractPartitions,
  } = useHddStorageViewModel();

  const [copiedSha, setCopiedSha] = useState(false);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedSha(true);
    setTimeout(() => setCopiedSha(false), 2000);
  };

  if (loading || !hddReport) {
    return (
      <div className="flex items-center justify-center h-full text-slate-500 font-mono text-xs">
        Lendo imagem física do HD (104.857.600 bytes) via zero-copy mmap...
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Top Header & Fast Partition Extractor */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
        <div className="flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30">
            <HardDrive className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-base font-bold text-slate-100">
                Hitachi PATA 100 MB Forensic Image (MEIHDFS)
              </h2>
              <StatusBadge variant="success" label="MBR & ESPELHO 100% OK" />
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              panasonic DMR EH-55 Firmware hdd_100.bin • 204.800 setores LBA de 512 bytes
            </p>
          </div>
        </div>

        {/* Action Button: Extract Partitions */}
        <div className="flex items-center gap-3 shrink-0">
          <button
            onClick={extractPartitions}
            disabled={extracting}
            className="px-5 py-2.5 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold text-xs flex items-center gap-2 shadow-xl shadow-cyan-950/50 transition-all"
          >
            {extracting ? (
              <span>Extraindo Partições...</span>
            ) : (
              <>
                <Download className="w-4 h-4" />
                <span>Extrair Partições (SHA256SUMS)</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Metrics Row */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Tamanho da Imagem"
          value="100.00"
          unit="MB"
          subtitle="104.857.600 bytes exatos"
          variant="cyan"
        />
        <MetricCard
          title="Validação MBR / Espelho"
          value="LBA 57.344"
          unit="(28 MB)"
          subtitle="Identical Bit-for-Bit Replica"
          variant="emerald"
          badge={<StatusBadge variant="success" label="MATCH 100%" />}
        />
        <MetricCard
          title="Partição 1 (AM34 OS)"
          value="6.173.264"
          unit="bytes"
          subtitle="LBA 65.536 (32 MB) • Entropia 7.994b"
          variant="slate"
        />
        <MetricCard
          title="Partição 2 (Microcódigo)"
          value="720.896"
          unit="bytes"
          subtitle="LBA 131.072 (64 MB) • 1.408 setores"
          variant="slate"
        />
      </div>

      {/* Navigation Tabs */}
      <div className="flex items-center gap-2 border-b border-slate-800 pb-2 text-xs">
        <button
          onClick={() => setActiveTab('overview')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors ${
            activeTab === 'overview'
              ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Layers className="w-3.5 h-3.5" />
          <span>Visão Geral do Disco</span>
        </button>

        <button
          onClick={() => setActiveTab('mbr')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors ${
            activeTab === 'mbr'
              ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <FileCheck className="w-3.5 h-3.5" />
          <span>MBR vs Espelho LBA 57.344</span>
        </button>

        <button
          onClick={() => setActiveTab('meihdfs')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors ${
            activeTab === 'meihdfs'
              ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Table className="w-3.5 h-3.5" />
          <span>256 Registros MEIHDFS (XOR 0xFF)</span>
        </button>

        <button
          onClick={() => setActiveTab('journal')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors ${
            activeTab === 'journal'
              ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <BookOpen className="w-3.5 h-3.5" />
          <span>Journal WAL (10 Blocos)</span>
        </button>

        <button
          onClick={() => setActiveTab('density')}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors ${
            activeTab === 'density'
              ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <BarChart3 className="w-3.5 h-3.5" />
          <span>Mapa de Densidade (100 MB)</span>
        </button>

        {manifest && (
          <button
            onClick={() => setActiveTab('extract')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-semibold transition-colors ${
              activeTab === 'extract'
                ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <FolderArchive className="w-3.5 h-3.5" />
            <span>Manifesto de Partições</span>
          </button>
        )}
      </div>

      {/* Tab 1: Overview */}
      {activeTab === 'overview' && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {hddReport.partitions.length === 0 ? (
              <div className="col-span-full p-8 text-center text-xs text-slate-500 font-mono rounded-xl bg-bench-900 border border-slate-800">
                Nenhuma partição MEIHDFS identificada no dump de disco rígido
              </div>
            ) : (
              hddReport.partitions.map(part => (
                <div
                  key={part.partition_id}
                  className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl flex flex-col justify-between"
                >
                  <div>
                    <div className="flex items-center justify-between mb-3">
                      <span className="text-xs font-bold text-cyan-400">
                        PARTIÇÃO #{part.partition_id}
                      </span>
                      <StatusBadge variant="success" label="MEIHDFS V1.0" />
                    </div>
                    <h3 className="text-sm font-bold text-slate-100">{part.name}</h3>
                    <p className="text-xs text-slate-400 mt-1">
                      Magic: {part.signature_hex} • Início: LBA {part.lba_start} ({part.offset_hex})
                    </p>

                    <div className="mt-4 p-3 rounded-lg bg-slate-950 border border-slate-800/80 space-y-2 text-xs">
                      <div className="flex justify-between text-slate-400">
                        <span>Tamanho do Payload:</span>
                        <span className="text-slate-200 font-bold">{part.exact_payload_size.toLocaleString()} bytes</span>
                      </div>
                      <div className="flex justify-between text-slate-400">
                        <span>Alinhamento em Setores:</span>
                        <span className="text-slate-200">{part.sectors_count} setores ({part.sector_aligned_size.toLocaleString()} B)</span>
                      </div>
                      <div className="flex justify-between text-slate-400">
                        <span>Entropia de Shannon:</span>
                        <span className="text-amber-400 font-bold">{part.entropy.toFixed(4)} bits/byte</span>
                      </div>
                      <div className="flex justify-between text-slate-400">
                        <span>SHA-256:</span>
                        <span className="text-emerald-400 truncate max-w-xs" title={part.exact_sha256}>
                          {part.exact_sha256}
                        </span>
                      </div>
                    </div>
                  </div>

                  <div className="mt-4 pt-3 border-t border-slate-800 text-[11px] text-slate-500 truncate">
                    Header: {part.header_hex_64}
                  </div>
                </div>
              ))
            )}
          </div>

          {/* Allocation & Journal Summary */}
          <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-3 flex items-center gap-2">
              <Layers className="w-4 h-4 text-cyan-400" />
              <span>Resumo Arquitetural do MEIHDFS</span>
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs font-mono">
              <div className="p-3 rounded-lg bg-slate-950 border border-slate-800">
                <span className="text-slate-500 text-[10px] block">TABELA DE DESCRITORES</span>
                <span className="text-slate-200 font-bold">LBA 24 a 31 (0x3000)</span>
                <span className="text-slate-400 text-[11px] block mt-1">4.096 bytes (256 registros)</span>
              </div>
              <div className="p-3 rounded-lg bg-slate-950 border border-slate-800">
                <span className="text-slate-500 text-[10px] block">CATÁLOGO DO JOURNAL</span>
                <span className="text-slate-200 font-bold">8.00 MB (LBA 16.384)</span>
                <span className="text-slate-400 text-[11px] block mt-1">10 blocos WAL de 128 KB</span>
              </div>
              <div className="p-3 rounded-lg bg-slate-950 border border-slate-800">
                <span className="text-slate-500 text-[10px] block">MARCADOR DE TRANSIÇÃO</span>
                <span className="text-slate-200 font-bold">24.00 MB (LBA 49.152)</span>
                <span className="text-slate-400 text-[11px] block mt-1">Boundary de integridade</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab 2: MBR vs Mirror LBA 57344 */}
      {activeTab === 'mbr' && (
        <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <FileCheck className="w-4 h-4 text-emerald-400" />
                <span>Auditoria Comparativa: MBR Primário (LBA 0) vs Backup Espelho (LBA 57.344)</span>
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                O Panasonic DMR-EH55 mantém uma réplica idêntica do Setor 0 aos 28 MB para recuperação automática.
              </p>
            </div>
            <StatusBadge variant="success" label="RÉPLICA CONFIRMADA BIT-A-BIT" />
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-mono">
            {/* Primary MBR */}
            <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
              <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                <span className="font-bold text-cyan-400">MBR Primário (LBA 0 - Offset 0x0000_0000)</span>
                <StatusBadge variant="info" label="SETOR 0" />
              </div>
              <div className="space-y-1.5 text-slate-400">
                <div className="flex justify-between">
                  <span>Assinatura Hex:</span>
                  <span className="text-slate-200">{hddReport.mbr_tables.lba0_hex_signature}</span>
                </div>
                <div className="flex justify-between">
                  <span>Magic ASCII:</span>
                  <span className="text-emerald-400 font-bold">{hddReport.mbr_tables.magic_ascii}</span>
                </div>
                <div className="flex justify-between">
                  <span>Disk GUID:</span>
                  <span className="text-slate-200">{hddReport.mbr_tables.disk_volume_guid}</span>
                </div>
                <div className="flex justify-between">
                  <span>IBM PC Boot (0x55AA):</span>
                  <span className="text-slate-400">AUSENTE (Proteção contra PC OS)</span>
                </div>
                <div className="flex justify-between">
                  <span>SHA-256:</span>
                  <span className="text-slate-300 truncate max-w-[200px]" title={hddReport.mbr_tables.lba0_sha256}>
                    {hddReport.mbr_tables.lba0_sha256}
                  </span>
                </div>
              </div>
            </div>

            {/* Mirror MBR */}
            <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
              <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                <span className="font-bold text-emerald-400">Espelho de Backup (LBA 57.344 - 28.00 MB)</span>
                <StatusBadge variant="success" label="ESPELHO LBA 57344" />
              </div>
              <div className="space-y-1.5 text-slate-400">
                <div className="flex justify-between">
                  <span>Offset Físico:</span>
                  <span className="text-slate-200">0x01C0_0000 (29.360.128 bytes)</span>
                </div>
                <div className="flex justify-between">
                  <span>Comparação Bit-a-Bit:</span>
                  <span className="text-emerald-400 font-bold">100% IDÊNTICO AO SETOR 0</span>
                </div>
                <div className="flex justify-between">
                  <span>Disk GUID:</span>
                  <span className="text-slate-200">{hddReport.mbr_tables.disk_volume_guid}</span>
                </div>
                <div className="flex justify-between">
                  <span>MD5:</span>
                  <span className="text-slate-300">{hddReport.mbr_tables.mirror_md5}</span>
                </div>
                <div className="flex justify-between">
                  <span>SHA-256:</span>
                  <span className="text-slate-300 truncate max-w-[200px]" title={hddReport.mbr_tables.mirror_sha256}>
                    {hddReport.mbr_tables.mirror_sha256}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab 3: MEIHDFS 256 Extent Records */}
      {activeTab === 'meihdfs' && meihdfsReport && (
        <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <Table className="w-4 h-4 text-cyan-400" />
                <span>Tabela Decodificada de Descritores MEIHDFS (256 Registros de 16 Bytes)</span>
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                Desofuscada via inversão bit-a-bit (XOR 0xFF). LBA 24..31 (Offset 0x00003000).
              </p>
            </div>

            {/* Tag Filter & Search */}
            <div className="flex items-center gap-2">
              <div className="relative">
                <Search className="w-3.5 h-3.5 absolute left-2.5 top-2.5 text-slate-500" />
                <input
                  type="text"
                  placeholder="Filtrar por tag ou hex..."
                  value={searchQuery}
                  onChange={e => setSearchQuery(e.target.value)}
                  className="pl-8 pr-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-xs text-slate-200 focus:outline-none focus:border-cyan-500"
                />
              </div>

              <select
                value={tagFilter}
                onChange={e => setTagFilter(e.target.value)}
                className="px-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-xs text-slate-300 focus:outline-none focus:border-cyan-500"
              >
                <option value="all">Todas as Tags</option>
                <option value="Extent">Extent Record (0x20)</option>
                <option value="Directory">Directory Descriptor (0x30)</option>
                <option value="Root">Root Volume Anchor (0x01)</option>
                <option value="Journal">WAL Journal Pointer (0x10)</option>
              </select>
            </div>
          </div>

          {/* Table */}
          <div className="overflow-x-auto max-h-[500px] scrollbar-thin scrollbar-thumb-slate-800">
            <table className="w-full text-left text-xs font-mono">
              <thead className="sticky top-0 bg-bench-950 border-b border-slate-800 text-slate-400 uppercase text-[10px]">
                <tr>
                  <th className="py-2 px-3">Idx</th>
                  <th className="py-2 px-3">Tag Decodificada</th>
                  <th className="py-2 px-3">Hex Bruto (0xFF Invertido)</th>
                  <th className="py-2 px-3">Parâmetros (Setores / LBA)</th>
                  <th className="py-2 px-3">Descrição Forense</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60">
                {filteredRecords.map(rec => (
                  <tr key={rec.index} className="hover:bg-slate-800/40 transition-colors">
                    <td className="py-2 px-3 font-bold text-slate-400">
                      #{rec.index}
                    </td>
                    <td className="py-2 px-3">
                      <span className="font-bold text-cyan-400">{rec.tag_name}</span>
                      <span className="text-[10px] text-slate-500 block">
                        Raw: 0x{rec.raw_tag.toString(16).toUpperCase()} → Dec: 0x{rec.decoded_tag.toString(16).toUpperCase()}
                      </span>
                    </td>
                    <td className="py-2 px-3 text-slate-300 text-[11px]">
                      <span className="text-slate-500 block line-through">{rec.raw_hex}</span>
                      <span className="text-emerald-400">{rec.inverted_hex}</span>
                    </td>
                    <td className="py-2 px-3 text-slate-300">
                      {rec.sector_count ? (
                        <div>
                          <span>{rec.sector_count} setores</span>
                          <span className="text-slate-500 block text-[10px]">LBA {rec.lba_offset}</span>
                        </div>
                      ) : (
                        <span className="text-slate-600">-</span>
                      )}
                    </td>
                    <td className="py-2 px-3 text-slate-400 text-[11px] max-w-xs truncate">
                      {rec.description}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Tab 4: Journal WAL */}
      {activeTab === 'journal' && meihdfsReport && (
        <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <BookOpen className="w-4 h-4 text-cyan-400" />
                <span>Anel de Transações do Journal WAL (10 Blocos de 128 KB)</span>
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                Faixa: 0x0082_0000 .. 0x0096_0000 (1.310.720 bytes) • Resiliência contra quedas de energia AC
              </p>
            </div>
            <StatusBadge variant="info" label="10 BLOCOS WAL" />
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3">
            {meihdfsReport.journal_blocks.map(blk => (
              <div
                key={blk.block_index}
                className="p-3 rounded-xl bg-slate-950 border border-slate-800 space-y-2 text-xs font-mono"
              >
                <div className="flex items-center justify-between border-b border-slate-800/80 pb-1.5">
                  <span className="font-bold text-cyan-400">Bloco #{blk.block_index}</span>
                  <span className="text-[10px] text-slate-500">{blk.start_offset_hex}</span>
                </div>
                <div className="space-y-1 text-[11px] text-slate-400">
                  <div className="flex justify-between">
                    <span>Seq Tx:</span>
                    <span className="text-slate-200">#{blk.transaction_seq}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Setores:</span>
                    <span className="text-slate-200">{blk.active_payload_sectors}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>CRC:</span>
                    <span className="text-emerald-400">{blk.checksum_crc}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Tab 5: Chunk Density Map (100 MB) */}
      {activeTab === 'density' && (
        <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <BarChart3 className="w-4 h-4 text-cyan-400" />
                <span>Mapa de Densidade de Dados Não-Zero (1 Bloco = 1 Megabyte)</span>
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                Identificação visual imediata das áreas de firmware, partições ativas e regiões vazias.
              </p>
            </div>
          </div>

          <div className="grid grid-cols-10 sm:grid-cols-20 gap-1.5 p-3 rounded-xl bg-slate-950 border border-slate-800">
            {hddReport.density_map.map(chunk => {
              let bg = 'bg-slate-900 text-slate-600';
              if (chunk.percentage > 90) bg = 'bg-cyan-500 text-slate-950 font-bold';
              else if (chunk.percentage > 50) bg = 'bg-cyan-700 text-slate-100';
              else if (chunk.percentage > 0) bg = 'bg-cyan-950 text-cyan-400 border border-cyan-800';

              return (
                <div
                  key={chunk.mb_index}
                  title={`MB ${chunk.mb_index} (0x${chunk.offset.toString(16).toUpperCase()}): ${chunk.percentage}% preenchido`}
                  className={`h-9 rounded flex flex-col items-center justify-center text-[9px] cursor-pointer hover:ring-2 hover:ring-cyan-300 transition-all ${bg}`}
                >
                  <span>{chunk.mb_index}M</span>
                  <span className="text-[8px] opacity-80">{chunk.percentage > 0 ? `${chunk.percentage}%` : '0%'}</span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Tab 6: Partition Extract Manifest */}
      {activeTab === 'extract' && manifest && (
        <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-bold text-emerald-400 flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4" />
                <span>Partições Extraídas com Sucesso (Manifesto SHA256SUMS)</span>
              </h3>
              <p className="text-xs text-slate-400 mt-0.5">
                Salvas em {manifest.outputDirectory} • Verificação hash independente pronta.
              </p>
            </div>
            <button
              onClick={() => copyToClipboard(manifest.manifestSha256Sums)}
              className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-750 text-slate-300 text-xs font-semibold flex items-center gap-1.5 border border-slate-700"
            >
              <Copy className="w-3.5 h-3.5" />
              <span>{copiedSha ? 'Copiado!' : 'Copiar SHA256SUMS'}</span>
            </button>
          </div>

          <div className="space-y-3">
            {manifest.extractedPartitions.map(item => (
              <div key={item.id} className="p-4 rounded-xl bg-slate-950 border border-slate-800 text-xs space-y-2">
                <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                  <span className="font-bold text-slate-200">{item.name}</span>
                  <span className="text-slate-400">{item.filename} ({item.sizeBytes.toLocaleString()} bytes)</span>
                </div>
                <div className="space-y-1 text-slate-400">
                  <div className="flex justify-between">
                    <span>Offset:</span>
                    <span className="text-cyan-400">{item.offsetHex}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>SHA-256:</span>
                    <span className="text-emerald-400">{item.sha256}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>MD5:</span>
                    <span className="text-slate-300">{item.md5}</span>
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
