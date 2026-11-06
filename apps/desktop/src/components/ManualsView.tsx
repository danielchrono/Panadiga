import React from "react";
import { useManualsSearch } from "../viewmodels/useManualsSearch";
import { ExecutionStatus } from "../types/domain";
import { BookOpen, AlertTriangle, RefreshCw, Search, FileCode } from "lucide-react";

export const ManualsView: React.FC = () => {
  const vm = useManualsSearch();

  const sampleQueries = ["S29JL064H", "MN2DS0016", "IC55007", "CN55001", "U99", "HDD FORMAT", "WP#/ACC"];

  return (
    <div className="space-y-6">
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-sm">
        <div className="flex items-center justify-between pb-4 border-b border-gray-800">
          <div>
            <h2 className="text-xl font-bold text-white flex items-center gap-2">
              <BookOpen className="w-6 h-6 text-emerald-400" />
              Service Manuals, Schematics & Hardware Indexer
            </h2>
            <p className="text-sm text-gray-400 mt-1">
              Busca textual e esquemática indexada de manuais de serviço da família Panasonic DIGA (PDF).
            </p>
          </div>
          <span className="px-3 py-1 rounded-full text-xs font-semibold bg-emerald-950 text-emerald-400 border border-emerald-800">
            PDF Text & Schematics
          </span>
        </div>

        <div className="mt-6">
          <label className="block text-xs font-medium text-gray-300 uppercase tracking-wider mb-2">
            Termo de Pesquisa / Código de Componente / Esquema
          </label>
          <div className="flex gap-3">
            <input
              type="text"
              value={vm.query}
              onChange={(e) => vm.setQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && vm.executeSearch()}
              placeholder="Digite o código (ex: S29JL064H, CN55001, U99...)"
              className="flex-1 bg-gray-950 border border-gray-700 rounded-lg px-4 py-2.5 text-sm text-gray-200 focus:outline-none focus:border-emerald-500 font-mono"
            />
            <button
              onClick={vm.executeSearch}
              disabled={vm.status === ExecutionStatus.RUNNING}
              className="flex items-center gap-2 px-6 py-2.5 rounded-lg text-sm font-semibold bg-emerald-600 text-white hover:bg-emerald-500 transition shadow-lg shadow-emerald-600/20 disabled:opacity-50"
            >
              {vm.status === ExecutionStatus.RUNNING ? (
                <>
                  <RefreshCw className="w-4 h-4 animate-spin" />
                  <span>Buscando...</span>
                </>
              ) : (
                <>
                  <Search className="w-4 h-4" />
                  <span>Pesquisar</span>
                </>
              )}
            </button>
          </div>

          <div className="mt-3 flex items-center gap-2 text-xs text-gray-400 flex-wrap">
            <span className="text-gray-500">Sugestões rápidas:</span>
            {sampleQueries.map((q) => (
              <button
                key={q}
                onClick={() => {
                  vm.setQuery(q);
                }}
                className="px-2 py-0.5 rounded bg-gray-800 hover:bg-gray-700 text-gray-300 font-mono text-[11px]"
              >
                {q}
              </button>
            ))}
          </div>
        </div>
      </div>

      {vm.errorMessage && (
        <div className="bg-rose-950/50 border border-rose-800 rounded-xl p-4 flex items-start gap-3 text-rose-200">
          <AlertTriangle className="w-5 h-5 text-rose-400 shrink-0 mt-0.5" />
          <div className="text-sm">
            <p className="font-semibold">Erro na Pesquisa</p>
            <p className="mt-1 text-xs text-rose-300 font-mono">{vm.errorMessage}</p>
          </div>
        </div>
      )}

      {vm.results.length > 0 && (
        <div className="space-y-4">
          <div className="flex items-center justify-between text-xs text-gray-400">
            <span>{vm.results.length} ocorrência(s) encontrada(s) para "{vm.query}"</span>
          </div>

          {vm.results.map((res, idx) => (
            <div
              key={idx}
              className="bg-gray-900 border border-gray-800 rounded-xl p-5 shadow-sm space-y-2 hover:border-gray-700 transition"
            >
              <div className="flex items-center justify-between">
                <span className="text-sm font-bold text-white flex items-center gap-2">
                  <FileCode className="w-4 h-4 text-emerald-400" />
                  {res.manual_filename}
                </span>
                <span className="px-2.5 py-0.5 rounded bg-gray-800 text-xs font-semibold text-emerald-400">
                  Página {res.page_number}
                </span>
              </div>
              <p className="text-xs text-gray-300 font-mono bg-gray-950 p-3 rounded border border-gray-850 leading-relaxed">
                {res.snippet}
              </p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
