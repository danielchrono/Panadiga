import { useState, useEffect, useCallback, useMemo } from 'react';
import {
  CryptanalysisReport,
  FrmTocDumpReport,
  CodebookReport,
} from '@/models';
import { invokeCommand } from '@/services/ipc';
import { logger } from '@/utils/logger';

export type FrmTab = 'entropy' | 'toc' | 'codebook' | 'correlation';

export function useFrmCryptoViewModel() {
  const [cryptReport, setCryptReport] = useState<CryptanalysisReport | null>(null);
  const [tocReport, setTocReport] = useState<FrmTocDumpReport | null>(null);
  const [codebookReport, setCodebookReport] = useState<CodebookReport | null>(null);
  const [activeTab, setActiveTab] = useState<FrmTab>('entropy');
  const [loading, setLoading] = useState<boolean>(false);
  const [codebookFilter, setCodebookFilter] = useState<string>('');

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [crypt, toc, codebook] = await Promise.all([
        invokeCommand<CryptanalysisReport>('analyze_frm'),
        invokeCommand<FrmTocDumpReport>('dump_frm_toc'),
        invokeCommand<CodebookReport>('get_codebook'),
      ]);
      setCryptReport(crypt);
      setTocReport(toc);
      setCodebookReport(codebook);
    } catch (err) {
      logger.error('FrmCryptoViewModel', 'Failed loading crypto forensic reports', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const filteredCodebook = useMemo(() => {
    if (!codebookReport) return [];
    if (!codebookFilter.trim()) return codebookReport.dictionary;

    const query = codebookFilter.trim().toUpperCase();
    return codebookReport.dictionary.filter(entry => {
      return (
        entry.block_hex.toUpperCase().includes(query) ||
        (entry.known_plaintext_hex && entry.known_plaintext_hex.toUpperCase().includes(query)) ||
        entry.interpretation.toUpperCase().includes(query)
      );
    });
  }, [codebookReport, codebookFilter]);

  return {
    cryptReport,
    tocReport,
    codebookReport,
    filteredCodebook,
    activeTab,
    setActiveTab,
    loading,
    codebookFilter,
    setCodebookFilter,
    refresh: loadData,
  };
}
