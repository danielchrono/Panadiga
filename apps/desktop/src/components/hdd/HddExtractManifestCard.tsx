import React from 'react';
import { CheckCircle2, Copy } from 'lucide-react';
import { PartitionExtractionManifest } from '@/models';

export interface HddExtractManifestCardProps {
  manifest: PartitionExtractionManifest;
  copiedSha: boolean;
  onCopySha: (text: string) => void;
}

export const HddExtractManifestCard: React.FC<HddExtractManifestCardProps> = ({
  manifest,
  copiedSha,
  onCopySha,
}) => {
  return (
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
          onClick={() => onCopySha(manifest.manifestSha256Sums)}
          className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-750 text-slate-300 text-xs font-semibold flex items-center gap-1.5 border border-slate-700 focus:outline-none focus:ring-2 focus:ring-slate-400"
          aria-label="Copiar manifesto de hashes SHA256SUMS"
        >
          <Copy className="w-3.5 h-3.5" />
          <span>{copiedSha ? 'Copiado!' : 'Copiar SHA256SUMS'}</span>
        </button>
      </div>

      <div className="space-y-3">
        {manifest.extractedPartitions.length === 0 ? (
          <div className="py-8 text-center text-xs text-slate-500 font-mono">
            Nenhuma partição extraída
          </div>
        ) : (
          manifest.extractedPartitions.map(item => (
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
          ))
        )}
      </div>
    </div>
  );
};
