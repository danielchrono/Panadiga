import React from 'react';
import { Layers, FileCode } from 'lucide-react';
import { ManualDocumentMetadata } from '@/models';
import { useTranslation } from '@/i18n/useTranslation';

interface ManualsDocCatalogProps {
  documents: ManualDocumentMetadata[];
  selectedDoc: ManualDocumentMetadata | null;
  setSelectedDoc: (doc: ManualDocumentMetadata) => void;
}

export const ManualsDocCatalog: React.FC<ManualsDocCatalogProps> = ({
  documents,
  selectedDoc,
  setSelectedDoc,
}) => {
  const { t } = useTranslation();

  return (
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl h-[680px] flex flex-col font-mono">
      <div className="flex items-center justify-between pb-3 border-b border-slate-800 shrink-0">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
          <Layers className="w-4 h-4 text-cyan-400" />
          <span>{t('manuals.meta.title')} ({documents.length})</span>
        </h3>
        <span className="text-[10px] text-slate-500 font-mono">
          Catálogo DIGA
        </span>
      </div>

      {/* Document list */}
      <div className="flex-1 overflow-y-auto pt-3 space-y-2 pr-1 scrollbar-thin scrollbar-thumb-slate-800">
        {documents.length === 0 ? (
          <div className="py-20 text-center text-xs text-slate-500">
            Nenhum manual indexado
          </div>
        ) : (
          documents.map(doc => {
            const isSelected = selectedDoc?.id === doc.id;
            return (
              <div
                key={doc.id}
                role="button"
                tabIndex={0}
                aria-pressed={isSelected}
                onClick={() => setSelectedDoc(doc)}
                onKeyDown={e => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    setSelectedDoc(doc);
                  }
                }}
                className={`p-3 rounded-xl border cursor-pointer transition-all focus:outline-none focus:ring-1 focus:ring-blue-500/50 ${
                  isSelected
                    ? 'bg-blue-500/10 border-blue-500/40 text-blue-200'
                    : 'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700 hover:text-slate-200'
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="text-xs font-bold truncate max-w-[200px]">
                    {doc.title}
                  </span>
                  <span className="text-[10px] text-slate-500 shrink-0">
                    {doc.page_count} {t('manuals.results.pages')}
                  </span>
                </div>

                <div className="flex items-center justify-between text-[10px] text-slate-500">
                  <span className="font-mono">{doc.file_name}</span>
                  <span className="capitalize">{doc.category}</span>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Selected doc detail footer */}
      {selectedDoc && (
        <div className="mt-3 pt-3 border-t border-slate-800 shrink-0 text-xs space-y-1">
          <div className="flex items-center gap-1.5 text-blue-400 font-bold text-[11px]">
            <FileCode className="w-3.5 h-3.5" />
            <span className="truncate">{selectedDoc.title}</span>
          </div>
          <p className="text-[10px] text-slate-400 truncate">
            {selectedDoc.file_path}
          </p>
        </div>
      )}
    </div>
  );
};
