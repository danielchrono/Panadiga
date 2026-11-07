/**
 * @file useManualsSearch.ts
 * @brief ViewModel for the Technical Documentation & Schematics Search module.
 */

import { useState, useCallback } from "react";
import { ManualSearchResult, ExecutionStatus } from "../types/domain";
import { digaApi } from "../services/digaApi";
import { useTranslation, TranslationKey } from "../i18n/useTranslation";

export interface ManualsSearchViewModel {
  readonly query: string;
  readonly status: ExecutionStatus;
  readonly errorMessage: string | null;
  readonly results: readonly ManualSearchResult[];
  readonly setQuery: (query: string) => void;
  readonly executeSearch: () => Promise<void>;
  readonly reset: () => void;
}

export function useManualsSearch(): ManualsSearchViewModel {
  const { t } = useTranslation();
  const [query, setQuery] = useState<string>("S29JL064H");
  const [status, setStatus] = useState<ExecutionStatus>(ExecutionStatus.IDLE);
  const [errorKey, setErrorKey] = useState<TranslationKey | null>(null);
  const [rawError, setRawError] = useState<string | null>(null);
  const [results, setResults] = useState<readonly ManualSearchResult[]>([]);

  const reset = useCallback(() => {
    setStatus(ExecutionStatus.IDLE);
    setErrorKey(null);
    setRawError(null);
    setResults([]);
  }, []);

  const executeSearch = useCallback(async () => {
    if (!query || query.trim().length === 0) {
      setErrorKey("manuals.search.emptyQuery");
      setRawError(null);
      setStatus(ExecutionStatus.ERROR);
      return;
    }

    setStatus(ExecutionStatus.RUNNING);
    setErrorKey(null);
    setRawError(null);

    const result = await digaApi.searchManuals(query);

    if (result.success) {
      setResults(result.data);
      setStatus(ExecutionStatus.SUCCESS);
    } else {
      setErrorKey(null);
      setRawError(result.error);
      setStatus(ExecutionStatus.ERROR);
    }
  }, [query]);

  const errorMessage = errorKey ? t(errorKey) : rawError;

  return {
    query,
    status,
    errorMessage,
    results,
    setQuery,
    executeSearch,
    reset,
  };
}
