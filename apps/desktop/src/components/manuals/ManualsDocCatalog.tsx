import React from 'react';
import { Layers } from 'lucide-react';
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
    <div className="rounded-xl bg-bench-900 border border-slate-800 p-5 shadow-xl space-y-4 font-mono">
      <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
        <Layers className="w-4 h-4 text-cyan-400" />
        <span>{t('manuals.meta.title')}</span>
      </h3>

      {/* Document list */}
      <div className="space-y-2 max-h-[500px] overflow-y-auto pr-1 scrollbar-thin scrollbar-thumb-slate-800">
        {documents.length === 0 ? (
          <div className="p-6 text-center text-xs text-slate-500">
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
                <p className="text-[10px] text-slate-500 line-clamp-2">
                  {doc.description}
                </p>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
