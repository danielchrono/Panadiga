/**
 * @file useManualsSearch.ts
 * @brief ViewModel for the Technical Documentation & Schematics Search module.
 */

import { useState, useCallback } from "react";
import { ManualSearchResult, ExecutionStatus } from "../types/domain";
import { digaApi } from "../services/digaApi";

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
  const [query, setQuery] = useState<string>("S29JL064H");
  const [status, setStatus] = useState<ExecutionStatus>(ExecutionStatus.IDLE);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [results, setResults] = useState<readonly ManualSearchResult[]>([]);

  const reset = useCallback(() => {
    setStatus(ExecutionStatus.IDLE);
    setErrorMessage(null);
    setResults([]);
  }, []);

  const executeSearch = useCallback(async () => {
    if (!query || query.trim().length === 0) {
      setErrorMessage("Please enter a keyword to search.");
      setStatus(ExecutionStatus.ERROR);
      return;
    }

    setStatus(ExecutionStatus.RUNNING);
    setErrorMessage(null);

    const result = await digaApi.searchManuals(query);

    if (result.success) {
      setResults(result.data);
      setStatus(ExecutionStatus.SUCCESS);
    } else {
      setErrorMessage(result.error);
      setStatus(ExecutionStatus.ERROR);
    }
  }, [query]);

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
