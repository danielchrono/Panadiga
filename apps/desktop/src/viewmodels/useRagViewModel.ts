import { useState, useCallback } from 'react';
import { digaApi } from '@/services/digaApi';
import { RagQueryResult, ExecutionStatus } from '@/types/domain';

export function useRagViewModel(initialQuery: string = 'How to recover error U99 on Spansion Flash?') {
  const [ragQuery, setRagQuery] = useState<string>(initialQuery);
  const [ragStatus, setRagStatus] = useState<ExecutionStatus>(ExecutionStatus.IDLE);
  const [ragResult, setRagResult] = useState<RagQueryResult | null>(null);
  const [ragError, setRagError] = useState<string | null>(null);

  const handleRagSubmit = useCallback(async () => {
    if (!ragQuery.trim()) return;
    setRagStatus(ExecutionStatus.RUNNING);
    setRagError(null);

    const res = await digaApi.queryRag(ragQuery);
    if (res.success) {
      setRagResult(res.data);
      setRagStatus(ExecutionStatus.SUCCESS);
    } else {
      setRagError(res.error);
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
