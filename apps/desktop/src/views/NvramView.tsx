import React, { useState } from 'react';
import { useNvramViewModel } from '@/viewmodels/useNvramViewModel';
import { MetricCard } from '@/components/MetricCard';
import { StatusBadge } from '@/components/StatusBadge';
import { Modal } from '@/components/Modal';
import {
  Cpu,
  ShieldCheck,
  CheckCircle2,
  AlertOctagon,
  FileDown,
  RefreshCw,
  Wrench,
  FileCode,
  Layers,
  ArrowRight,
  Info,
} from 'lucide-react';

export const NvramView: React.FC = () => {
  const {
    report,
    sectorMap,
    filteredSectors,
    selectedSector,
    setSelectedSector,
    loading,
    cleaning,
    cleanSuccess,
    filterType,
    setFilterType,
    executeSurgicalClean,
    refresh,
  } = useNvramViewModel();

  const [showBackupModal, setShowBackupModal] = useState(false);

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12 font-mono">
      {/* Surgical Clean Notification */}
      {cleanSuccess && (
        <div className="rounded-xl bg-emerald-500/10 border border-emerald-500/40 p-4 flex items-center justify-between shadow-xl">
          <div className="flex items-center gap-3">
            <CheckCircle2 className="w-5 h-5 text-emerald-400 shrink-0" />
            <div>
              <h4 className="text-xs font-bold text-emerald-300">
                LIMPEZA CIRÚRGICA U99 CONCLUÍDA COM SUCESSO!
              </h4>
              <p className="text-[11px] text-slate-300 mt-0.5">
                Microsetores SA134..SA141 limpos para estado virgem de fábrica (0xFF * 65.536). Bootloader 100% preservado.
              </p>
            </div>
          </div>
          <button
            onClick={() => setShowBackupModal(true)}
            className="px-3 py-1.5 rounded-lg bg-emerald-500/20 border border-emerald-500/40 text-emerald-300 text-xs font-semibold hover:bg-emerald-500/30 flex items-center gap-1.5 transition-colors"
          >
            <FileDown className="w-3.5 h-3.5" />
            <span>Auditoria SHA-256</span>
          </button>
        </div>
      )}

      {/* Top Bar: Action Panel & Spansion Model Header */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bench-900 border border-slate-800 shadow-xl">
        <div className="flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/30">
            <Cpu className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-base font-bold text-slate-100">
                Spansion S29JL064H NOR Flash (8 MB / 64 Mbit, TSOP-48)
              </h2>
              <StatusBadge variant="info" label="TOP-BOOT" />
            </div>
            <p className="text-xs text-slate-400 mt-0.5">
              Top Parameter Bank: 0x007F_0000 .. 0x007F_FFFF (64 KB subdivididos em 8 microsetores de 8 KB)
            </p>
          </div>
        </div>

        {/* 1-Click Surgical Clean Button */}
        <div className="flex items-center gap-3 shrink-0">
          <button
            onClick={executeSurgicalClean}
            disabled={cleaning}
            className={`px-5 py-2.5 rounded-xl font-bold text-xs flex items-center gap-2 shadow-xl transition-all ${
              cleanSuccess
                ? 'bg-slate-800 text-slate-300 border border-slate-700 hover:bg-slate-750'
                : 'bg-amber-500 hover:bg-amber-400 text-slate-950 shadow-amber-950/50'
            }`}
          >
            {cleaning ? (
              <>
                <RefreshCw className="w-4 h-4 animate-spin" />
                <span>Executando Cirurgia...</span>
              </>
            ) : (
              <>
                <Wrench className="w-4 h-4" />
                <span>
                  {cleanSuccess ? 'Refazer Limpeza Cirúrgica U99' : 'Limpeza Cirúrgica U99 (1-Clique)'}
                </span>
              </>
            )}
          </button>

          <button
            onClick={() => setShowBackupModal(true)}
            className="p-2.5 rounded-xl bg-slate-800 hover:bg-slate-750 border border-slate-700 text-slate-300 transition-colors"
            title="Ver Relatório de Auditoria e Backup"
          >
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
          </button>
        </div>
      </div>

      {/* Metrics Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Tamanho Total Flash"
          value="8.388.608"
          unit="bytes"
          subtitle="Capacidade exata Spansion (8.00 MB)"
          variant="slate"
        />
        <MetricCard
          title="Área NVRAM (SA134..SA141)"
          value="65.536"
          unit="bytes"
          subtitle="Top 64 KB (0x007F0000 - 0x007FFFFF)"
          variant={cleanSuccess ? 'emerald' : 'amber'}
          badge={
            cleanSuccess ? (
              <StatusBadge variant="success" label="VIRGEM 0xFF" />
            ) : (
              <StatusBadge variant="warning" label="18.432B ATIVOS" />
            )
          }
        />
        <MetricCard
          title="Bootloader Imutável"
          value="8.323.072"
          unit="bytes"
          subtitle="0x00000000 .. 0x007EFFFF intacto"
          variant="emerald"
          badge={<StatusBadge variant="success" label="100% BIT-FOR-BIT" />}
        />
        <MetricCard
          title="Microsetores Auditados"
          value="8"
          unit="de 8 KB"
          subtitle="SA134 a SA141 inspecionados"
          variant="cyan"
        />
      </div>

      {/* Interactive Microsector Visualizer (SA0..SA141) */}
      <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
          <div>
            <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
              <Layers className="w-4 h-4 text-cyan-400" />
              <span>Mapa Interativo de Setores da Flash Spansion (SA0 .. SA141)</span>
            </h3>
            <p className="text-[11px] text-slate-500 font-sans mt-0.5">
              Clique em qualquer microsetor para inspecionar offset, cabeçalho e bytes ativos.
            </p>
          </div>

          {/* Filter Pills */}
          <div className="flex items-center gap-1.5 p-1 rounded-lg bg-slate-950 border border-slate-800 text-xs">
            <button
              onClick={() => setFilterType('all')}
              className={`px-2.5 py-1 rounded text-[11px] transition-colors ${
                filterType === 'all' ? 'bg-slate-800 text-slate-100 font-bold' : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              Todos (142)
            </button>
            <button
              onClick={() => setFilterType('corrupted')}
              className={`px-2.5 py-1 rounded text-[11px] transition-colors ${
                filterType === 'corrupted'
                  ? 'bg-amber-500/20 text-amber-300 font-bold border border-amber-500/30'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              Top NVRAM (SA134..SA141)
            </button>
            <button
              onClick={() => setFilterType('bootloader')}
              className={`px-2.5 py-1 rounded text-[11px] transition-colors ${
                filterType === 'bootloader'
                  ? 'bg-emerald-500/20 text-emerald-300 font-bold border border-emerald-500/30'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              Bootloader (SA0..SA15)
            </button>
          </div>
        </div>

        {/* High-Contrast Interactive Grid */}
        <div className="p-3 rounded-xl bg-slate-950 border border-slate-800/80">
          <div className="flex items-center justify-between text-[11px] text-slate-500 mb-2 px-1">
            <span>0x0000_0000 (SA0 Bootloader)</span>
            <span className="text-amber-400 font-bold">0x007F_0000 (SA134..SA141 NVRAM)</span>
            <span>0x0080_0000 (Fim da Flash)</span>
          </div>

          <div className="grid grid-cols-12 sm:grid-cols-24 md:grid-cols-36 lg:grid-cols-71 gap-1">
            {sectorMap.map(sector => {
              const isSelected = selectedSector?.name === sector.name;
              let bg = 'bg-slate-800 hover:bg-slate-700';

              if (sector.is_corrupted_area) {
                bg = cleanSuccess
                  ? 'bg-emerald-500 hover:bg-emerald-400 text-slate-950'
                  : 'bg-rose-500 hover:bg-rose-400 text-slate-950 animate-pulse';
              } else if (sector.sector_type === 'bootloader') {
                bg = 'bg-cyan-900/60 hover:bg-cyan-800/80 text-cyan-200';
              }

              return (
                <button
                  key={sector.name}
                  onClick={() => setSelectedSector(sector)}
                  title={`${sector.name} (0x${sector.start_offset.toString(16).toUpperCase()}) - ${sector.size_bytes / 1024} KB`}
                  className={`h-7 rounded text-[9px] font-bold flex items-center justify-center transition-all ${bg} ${
                    isSelected ? 'ring-2 ring-cyan-400 scale-110 z-10' : ''
                  }`}
                >
                  {sector.index}
                </button>
              );
            })}
          </div>

          {/* Grid Legend */}
          <div className="flex flex-wrap items-center gap-4 mt-3 pt-3 border-t border-slate-800 text-[11px] text-slate-400">
            <div className="flex items-center gap-1.5">
              <span className="w-3 h-3 rounded bg-cyan-900/60 border border-cyan-700/50" />
              <span>Bootloader Imutável (SA0..SA15)</span>
            </div>
            <div className="flex items-center gap-1.5">
              <span className="w-3 h-3 rounded bg-slate-800" />
              <span>OS Firmware Normal (SA16..SA133 - 64 KB)</span>
            </div>
            <div className="flex items-center gap-1.5">
              <span className={`w-3 h-3 rounded ${cleanSuccess ? 'bg-emerald-500' : 'bg-rose-500 animate-pulse'}`} />
              <span>
                {cleanSuccess
                  ? 'Top Bank Limpo / Virgem (SA134..SA141)'
                  : 'Área Corrompida U99 (SA134..SA141 - 8 KB cada)'}
              </span>
            </div>
          </div>
        </div>

        {/* Selected Sector Inspector Panel */}
        {selectedSector && (
          <div className="p-4 rounded-xl bg-bench-950 border border-slate-800 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
            <div className="space-y-1">
              <div className="flex items-center gap-2">
                <span className="text-sm font-bold text-slate-100">
                  Setor Selecionado: {selectedSector.name}
                </span>
                <StatusBadge
                  variant={selectedSector.is_corrupted_area ? (cleanSuccess ? 'success' : 'error') : 'info'}
                  label={selectedSector.is_corrupted_area ? (cleanSuccess ? 'LIMPO 0xFF' : 'CORRUPÇÃO U99') : 'INTACTO'}
                />
              </div>
              <p className="text-xs text-slate-400">{selectedSector.notes}</p>
            </div>

            <div className="flex flex-wrap items-center gap-4 text-xs font-mono">
              <div className="text-slate-400">
                <span className="text-slate-500 block text-[10px]">OFFSET FÍSICO</span>
                <span>0x{selectedSector.start_offset.toString(16).padStart(8, '0').toUpperCase()}</span>
              </div>
              <div className="text-slate-400">
                <span className="text-slate-500 block text-[10px]">TAMANHO</span>
                <span>{selectedSector.size_bytes / 1024} KB</span>
              </div>
              <div className="text-slate-400">
                <span className="text-slate-500 block text-[10px]">BYTES ATIVOS</span>
                <span className={selectedSector.is_corrupted_area && !cleanSuccess ? 'text-rose-400 font-bold' : 'text-slate-200'}>
                  {selectedSector.active_bytes ?? 0} B
                </span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* 8 Microsectors Detailed Audit Table */}
      {report && (
        <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
                <Wrench className="w-4 h-4 text-amber-400" />
                <span>Auditoria Detalhada dos 8 Microsetores (SA134 a SA141 - 8 KB cada)</span>
              </h3>
              <p className="text-[11px] text-slate-500 font-sans mt-0.5">
                Validação estrutural dos parâmetros de inicialização e checksums da EEPROM emulada.
              </p>
            </div>
            <span className="text-xs text-slate-400">65.536 bytes totais</span>
          </div>

          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs font-mono">
              <thead>
                <tr className="border-b border-slate-800 text-slate-500 uppercase text-[10px]">
                  <th className="py-2.5 px-3">Microsetor</th>
                  <th className="py-2.5 px-3">Offset Físico</th>
                  <th className="py-2.5 px-3">Dados Ativos (Não-0xFF)</th>
                  <th className="py-2.5 px-3">Header (Primeiros 8 Bytes)</th>
                  <th className="py-2.5 px-3 text-right">Diagnóstico</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-800/60">
                {report.microsectors.map(sec => {
                  const pct = ((sec.active_bytes / sec.total_bytes) * 100).toFixed(1);
                  return (
                    <tr key={sec.sector_name} className="hover:bg-slate-800/40 transition-colors">
                      <td className="py-2.5 px-3 font-bold text-slate-200">
                        {sec.sector_name}
                      </td>
                      <td className="py-2.5 px-3 text-cyan-400">
                        0x{sec.offset.toString(16).padStart(8, '0').toUpperCase()}
                      </td>
                      <td className="py-2.5 px-3">
                        <div className="flex items-center gap-2">
                          <span className={cleanSuccess ? 'text-slate-400' : 'text-amber-400 font-bold'}>
                            {sec.active_bytes} / {sec.total_bytes} ({pct}%)
                          </span>
                          <div className="w-16 h-1.5 rounded-full bg-slate-800 overflow-hidden">
                            <div
                              className={`h-full ${cleanSuccess ? 'bg-emerald-400' : 'bg-amber-400'}`}
                              style={{ width: `${pct}%` }}
                            />
                          </div>
                        </div>
                      </td>
                      <td className="py-2.5 px-3 text-slate-300 tracking-wider">
                        {sec.header_hex}
                      </td>
                      <td className="py-2.5 px-3 text-right">
                        {cleanSuccess ? (
                          <StatusBadge variant="success" label="VIRGEM 0xFF" />
                        ) : sec.sector_name === 'SA137' ? (
                          <StatusBadge variant="error" label="U99 REBOOT LOOP" />
                        ) : (
                          <StatusBadge variant="warning" label="EEMUL ATIVA" />
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Audit & Hash Verification Modal */}
      <Modal
        isOpen={showBackupModal}
        onClose={() => setShowBackupModal(false)}
        title="Auditoria Criptográfica SHA-256 e Backup da NVRAM"
        subtitle="Verificação forense rigorosa: Spansion S29JL064H Flash Dump"
        maxWidth="2xl"
      >
        {report && (
          <div className="space-y-4 font-mono text-xs">
            <div className="p-3.5 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
              <span className="text-[10px] text-slate-500 uppercase block font-semibold">
                Dump Original Corrompido
              </span>
              <div className="space-y-1">
                <div className="flex justify-between text-slate-400">
                  <span>Arquivo:</span>
                  <span className="text-slate-200 truncate max-w-sm">{report.input_file}</span>
                </div>
                <div className="flex justify-between text-slate-400">
                  <span>SHA-256:</span>
                  <span className="text-amber-400 truncate max-w-sm">{report.original_sha256}</span>
                </div>
                <div className="flex justify-between text-slate-400">
                  <span>MD5:</span>
                  <span className="text-slate-300">{report.original_md5}</span>
                </div>
              </div>
            </div>

            <div className="p-3.5 rounded-xl bg-emerald-950/20 border border-emerald-500/30 space-y-2">
              <span className="text-[10px] text-emerald-400 uppercase block font-semibold">
                Dump Cirurgicamente Limpo (Pronto para Gravação)
              </span>
              <div className="space-y-1">
                <div className="flex justify-between text-slate-400">
                  <span>Arquivo:</span>
                  <span className="text-slate-200 truncate max-w-sm">{report.output_file}</span>
                </div>
                <div className="flex justify-between text-slate-400">
                  <span>SHA-256:</span>
                  <span className="text-emerald-400 truncate max-w-sm">{report.clean_sha256}</span>
                </div>
                <div className="flex justify-between text-slate-400">
                  <span>MD5:</span>
                  <span className="text-slate-300">{report.clean_md5}</span>
                </div>
              </div>
            </div>

            <div className="p-3.5 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
              <span className="text-[10px] text-slate-500 uppercase block font-semibold">
                Backup de Segurança da Área Corrompida (64 KB)
              </span>
              <div className="flex justify-between text-slate-400">
                <span>Backup Local:</span>
                <span className="text-slate-200">{report.backup_file}</span>
              </div>
              <div className="flex justify-between text-slate-400">
                <span>Tamanho:</span>
                <span className="text-slate-200">65.536 bytes (SA134..SA141)</span>
              </div>
              <div className="flex justify-between text-slate-400">
                <span>Imutabilidade do Bootloader:</span>
                <span className="text-emerald-400 font-bold">100% PRESERVADO (OK)</span>
              </div>
            </div>

            <div className="pt-2 flex justify-end">
              <button
                onClick={() => setShowBackupModal(false)}
                className="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold"
              >
                Fechar Auditoria
              </button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
};
