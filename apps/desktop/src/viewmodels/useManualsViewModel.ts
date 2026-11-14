import { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import {
  ManualSearchResult,
  ManualDocumentMetadata,
  ServiceManualCatalogItem,
} from '@/models';
import { invokeCommand } from '@/services/ipc';
import { useTranslation, TranslationKey } from '@/i18n/useTranslation';

function inferManualMetadata(item: ServiceManualCatalogItem, index: number): ManualDocumentMetadata {
  const lower = item.file_name.toLowerCase();
  const category: ManualDocumentMetadata['category'] = lower.includes('schematic')
    ? 'schematic'
    : lower.includes('operating') || lower.includes('user')
    ? 'operating_instructions'
    : lower.includes('quick') || lower.includes('guide') || lower.includes('bulletin')
    ? 'bulletin'
    : 'service_manual';

  const cleanTitle = item.file_name
    .replace(/\.pdf$/i, '')
    .replace(/_/g, ' ')
    .replace(/\b\w/g, c => c.toUpperCase());

  const keyIcs: string[] = [];
  if (lower.includes('eh55')) keyIcs.push('AM34', 'S29JL064H', 'P7501');
  if (lower.includes('mn103') || lower.includes('am3')) keyIcs.push('MN103S', 'AM34');
  if (lower.includes('schematic')) keyIcs.push('VEP79138', 'IC5001', 'Q7514');
  if (keyIcs.length === 0) keyIcs.push('PDF');

  return {
    id: String(index + 1),
    file_name: item.file_name,
    file_path: item.relative_path,
    title: cleanTitle,
    description: `${item.relative_path} (${item.formatted_size})`,
    category,
    size_bytes: item.size_bytes,
    page_count: 0,
    key_ics: keyIcs,
  };
}

export function useManualsViewModel() {
  const { t } = useTranslation();
  const [documents, setDocuments] = useState<ManualDocumentMetadata[]>([]);
  const [searchResults, setSearchResults] = useState<ManualSearchResult[]>([]);
  const [searchQuery, setSearchQuery] = useState<string>('U99');
  const [errorKey, setErrorKey] = useState<TranslationKey | null>(null);
  const [rawError, setRawError] = useState<string | null>(null);
  const [activeTerms] = useState<string[]>([
    'U99',
    'S29JL064',
    'MN103S',
    'P7501',
    'IC5001',
    'NSW5V',
    'HDD FORMAT',
    'SELF-CHECK',
    'RAM DISK',
    'VEP79138',
  ]);
  const [selectedDoc, setSelectedDoc] = useState<ManualDocumentMetadata | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [categoryFilter, setCategoryFilter] = useState<string>('all');

  useEffect(() => {
    let mounted = true;
    invokeCommand<ServiceManualCatalogItem[]>('list_service_manuals')
      .then(items => {
        if (!mounted) return;
        const mapped = items.map(inferManualMetadata);
        setDocuments(mapped);
        setSelectedDoc(prev => prev ?? mapped[0] ?? null);
      })
      .catch(err => {
        if (!mounted) return;
        setRawError(err instanceof Error ? err.message : String(err));
      });
    return () => {
      mounted = false;
    };
  }, []);

  const isMountedRef = useRef(true);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  const executeSearch = useCallback(
    async (queryParam?: string) => {
      const q = (queryParam !== undefined ? queryParam : searchQuery).trim();
      if (!q) {
        if (!isMountedRef.current) return;
        setErrorKey('manuals.search.emptyQuery');
        setRawError(null);
        setSearchResults([]);
        return;
      }

      if (!isMountedRef.current) return;
      setErrorKey(null);
      setRawError(null);
      setLoading(true);
      try {
        const results = await invokeCommand<ManualSearchResult[]>('search_manuals', { query: q });
        if (isMountedRef.current) {
          setSearchResults(results);
        }
      } catch (err) {
        if (isMountedRef.current) {
          setRawError(err instanceof Error ? err.message : String(err));
        }
      } finally {
        if (isMountedRef.current) {
          setLoading(false);
        }
      }
    },
    [searchQuery]
  );

  useEffect(() => {
    executeSearch('U99');
  }, [executeSearch]);

  const filteredDocs = useMemo(() => {
    if (categoryFilter === 'all') return documents;
    return documents.filter(d => d.category === categoryFilter);
  }, [documents, categoryFilter]);

  const selectTerm = useCallback(
    (term: string) => {
      setSearchQuery(term);
      executeSearch(term);
    },
    [executeSearch]
  );

  const errorMessage = errorKey ? t(errorKey) : rawError;

  return {
    query: searchQuery,
    setQuery: setSearchQuery,
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
    errorMessage,
  };
}
