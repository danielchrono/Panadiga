import { useState, useCallback } from 'react';
import { invokeCommand } from '@/services/ipc';
import { RagQueryResult } from '@/models';
import { ExecutionStatus } from '@/types/domain';

export function useRagViewModel(initialQuery: string = 'How to recover error U99 on Spansion Flash?') {
  const [ragQuery, setRagQuery] = useState<string>(initialQuery);
  const [ragStatus, setRagStatus] = useState<ExecutionStatus>(ExecutionStatus.IDLE);
  const [ragResult, setRagResult] = useState<RagQueryResult | null>(null);
  const [ragError, setRagError] = useState<string | null>(null);

  const handleRagSubmit = useCallback(async () => {
    if (!ragQuery.trim()) return;
    setRagStatus(ExecutionStatus.RUNNING);
    setRagError(null);

    try {
      const res = await invokeCommand<RagQueryResult>('query_rag_engine', { query: ragQuery });
      setRagResult(res);
      setRagStatus(ExecutionStatus.SUCCESS);
    } catch (err) {
      setRagError(err instanceof Error ? err.message : String(err));
      setRagStatus(ExecutionStatus.ERROR);
    }
  }, [ragQuery]);

  return {
    ragQuery,
    setRagQuery,
    ragStatus,
    ragResult,
    ragError,
    handleRagSubmit,
  };
}
