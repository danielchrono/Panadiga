import { useState, useEffect, useCallback, useMemo } from 'react';
import { ManualSearchResult, ManualDocumentMetadata } from '@/models';
import { invokeCommand } from '@/services/ipc';
import { mockManualDocuments } from '@/services/mockData';

export function useManualsViewModel() {
  const [documents] = useState<ManualDocumentMetadata[]>(mockManualDocuments);
  const [searchResults, setSearchResults] = useState<ManualSearchResult[]>([]);
  const [searchQuery, setSearchQuery] = useState<string>('U99');
  const [activeTerms] = useState<string[]>([
    'U99',
    'S29JL064',
    'MN2DS',
    'CN55001',
    'IC5001',
    'IC55007',
    'HDD FORMAT',
    'SELF-CHECK',
    'RAM DISK',
    'VEP79138',
  ]);
  const [selectedDoc, setSelectedDoc] = useState<ManualDocumentMetadata | null>(mockManualDocuments[0]);
  const [loading, setLoading] = useState<boolean>(false);
  const [categoryFilter, setCategoryFilter] = useState<string>('all');

  const executeSearch = useCallback(async (query: string) => {
    setLoading(true);
    try {
      const results = await invokeCommand<ManualSearchResult[]>('search_manuals', { query });
      setSearchResults(results);
    } catch (err) {
      console.error('Failed searching manuals:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    executeSearch('U99');
  }, [executeSearch]);

  const filteredDocs = useMemo(() => {
    if (categoryFilter === 'all') return documents;
    return documents.filter(d => d.category === categoryFilter);
  }, [documents, categoryFilter]);

  const selectTerm = useCallback((term: string) => {
    setSearchQuery(term);
    executeSearch(term);
  }, [executeSearch]);

  return {
    documents: filteredDocs,
    allDocuments: documents,
    searchResults,
    searchQuery,
    setSearchQuery,
    activeTerms,
    selectedDoc,
    setSelectedDoc,
    loading,
    categoryFilter,
    setCategoryFilter,
    executeSearch,
    selectTerm,
  };
}
