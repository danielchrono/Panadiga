import React from 'react';
import { useTranslation } from '@/i18n/useTranslation';
import { StatusBadge } from '@/components/StatusBadge';
import { ExecutionStatus } from '@/models';
import { useRagViewModel } from '@/viewmodels/useRagViewModel';
import {
  Bot,
  ShieldCheck,
  ShieldAlert,
  Sparkles,
  CheckCircle2,
  FileText,
} from 'lucide-react';

interface RagAssistantPanelProps {
  viewModel?: ReturnType<typeof useRagViewModel>;
}

export const RagAssistantPanel: React.FC<RagAssistantPanelProps> = ({ viewModel: externalVm }) => {
  const { t } = useTranslation();
  const internalVm = useRagViewModel();
  const vm = externalVm || internalVm;

  const {
    ragQuery,
    setRagQuery,
    ragStatus,
    ragResult,
    ragError,
    handleRagSubmit,
  } = vm;

  return (
    <div className="rounded-2xl bg-slate-950 border border-cyan-500/30 p-6 shadow-2xl space-y-5 font-mono">
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 border-b border-slate-800 pb-4">
        <div className="flex items-center gap-3">
          <div className="p-2.5 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/30">
            <Bot className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="text-sm font-bold text-slate-100">
                {t('rag.title')}
              </h3>
              <StatusBadge variant="success" label={t('rag.badge')} />
            </div>
            <p className="text-xs text-slate-400">
              {t('rag.subtitle')}
            </p>
          </div>
        </div>
        <div className="text-xs text-cyan-400 font-mono flex items-center gap-1.5">
          <ShieldCheck className="w-4 h-4 text-emerald-400" />
          <span>{t('rag.groundedBadge')}</span>
        </div>
      </div>

      {/* Input Bar */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
        <div className="relative flex-1">
          <input
            type="text"
            placeholder={t('rag.inputPlaceholder')}
            value={ragQuery}
            onChange={e => setRagQuery(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && handleRagSubmit()}
            aria-label="Pergunta técnica para o assistente de IA"
            className="w-full pl-4 pr-4 py-2.5 rounded-xl bg-bench-900 border border-slate-800 text-xs text-slate-100 placeholder-slate-500 focus:outline-none focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500/50 font-mono"
          />
        </div>

        <button
          onClick={handleRagSubmit}
          disabled={ragStatus === ExecutionStatus.RUNNING}
          aria-label={t('rag.askButton')}
          className="px-5 py-2.5 rounded-xl bg-cyan-500 hover:bg-cyan-400 disabled:opacity-50 text-slate-950 font-bold text-xs flex items-center justify-center gap-2 transition-colors shrink-0 focus:outline-none focus:ring-2 focus:ring-cyan-400"
        >
          <Sparkles className="w-4 h-4" />
          <span>{ragStatus === ExecutionStatus.RUNNING ? t('rag.asking') : t('rag.askButton')}</span>
        </button>
      </div>

      {/* Error Alert Display */}
      {ragError && (
        <div className="p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs flex items-center gap-2">
          <ShieldAlert className="w-4 h-4 text-rose-400 shrink-0" />
          <span>{ragError}</span>
        </div>
      )}

      {/* RAG Answer Display */}
      {ragResult && (
        <div className="space-y-4 pt-2">
          {/* Safety & Invariant Status Bar */}
          <div className="flex flex-wrap items-center justify-between gap-3 p-3 rounded-xl bg-bench-900 border border-slate-800 text-xs">
            <div className="flex items-center gap-2">
              {ragResult.safety.is_safe ? (
                <span className="flex items-center gap-1.5 text-emerald-400 font-bold">
                  <CheckCircle2 className="w-4 h-4" /> {t('rag.safetyPassed')}
                </span>
              ) : (
                <span className="flex items-center gap-1.5 text-rose-400 font-bold">
                  <ShieldAlert className="w-4 h-4" /> {t('rag.safetyBlocked')}
                </span>
              )}
            </div>
            <div className="flex items-center gap-4 text-slate-400 text-[11px]">
              <span>{t('rag.confidence')} <strong className="text-cyan-300">{(ragResult.confidence * 100).toFixed(0)}%</strong></span>
              <span>{t('rag.latency')} <strong className="text-slate-200">{ragResult.execution_time_ms.toFixed(1)}ms</strong></span>
              <span>{t('rag.chunksCount')} <strong className="text-slate-200">{ragResult.retrieved_chunks_count}</strong></span>
            </div>
          </div>

          {/* Answer Text Card */}
          <div className="p-4 rounded-xl bg-slate-900/80 border border-slate-800 text-xs text-slate-200 leading-relaxed space-y-3 whitespace-pre-line font-sans">
            {ragResult.answer}
          </div>

          {/* FactCritic Audit Report */}
          {ragResult.critic && (
            <div className="p-3.5 rounded-xl bg-bench-900 border border-slate-800 text-xs space-y-2">
              <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                <span className="font-bold text-slate-300 flex items-center gap-1.5">
                  <ShieldCheck className="w-4 h-4 text-cyan-400" />
                  {t('rag.criticReportTitle')}
                </span>
                <StatusBadge
                  variant={ragResult.critic.is_faithful ? 'success' : 'warning'}
                  label={ragResult.critic.is_faithful ? t('rag.groundedBadge') : t('rag.hallucinationWarning')}
                />
              </div>
              {ragResult.critic.ungrounded_entities.length === 0 ? (
                <div className="text-emerald-400 text-[11px] font-mono">Nenhuma entidade não fundamentada</div>
              ) : (
                <div className="text-rose-400 text-[11px] space-y-1">
                  <span className="font-bold">{t('rag.ungroundedEntities')}</span>
                  {ragResult.critic.ungrounded_entities.map((ue, idx) => (
                    <div key={idx} className="font-mono pl-2">• {ue}</div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Citations List */}
          <div className="space-y-2 pt-1">
            <h4 className="text-xs font-bold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
              <FileText className="w-4 h-4 text-amber-400" />
              {t('rag.citationsTitle')}
            </h4>
            {ragResult.citations.length === 0 ? (
              <div className="p-3 rounded-xl bg-bench-900 border border-slate-800/80 text-xs text-slate-500">
                Nenhuma citação documentada
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {ragResult.citations.map((c, idx) => (
                  <div
                    key={idx}
                    className="p-3 rounded-xl bg-bench-900 border border-slate-800/80 text-xs space-y-1"
                  >
                    <div className="flex items-center justify-between text-[11px] font-bold text-cyan-400">
                      <span className="truncate max-w-[200px]">{c.doc}</span>
                      <span className="text-slate-500 font-mono">{t('rag.line')} {c.lines}</span>
                    </div>
                    <div className="text-[11px] text-slate-400 font-mono truncate">{c.section}</div>
                    <p className="text-[11px] text-slate-300 italic pt-1 border-t border-slate-800/60 font-sans">
                      "{c.excerpt}"
                    </p>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
