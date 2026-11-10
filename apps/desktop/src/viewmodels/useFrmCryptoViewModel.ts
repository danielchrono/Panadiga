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
  const [error, setError] = useState<string | null>(null);
  const [codebookFilter, setCodebookFilter] = useState<string>('');

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const results = await Promise.allSettled([
        invokeCommand<CryptanalysisReport>('analyze_frm'),
        invokeCommand<FrmTocDumpReport>('dump_frm_toc'),
        invokeCommand<CodebookReport>('get_codebook'),
      ]);

      if (results[0].status === 'fulfilled') {
        setCryptReport(results[0].value);
      } else {
        logger.error('FrmCryptoViewModel', 'Failed analyzing FRM entropy', results[0].reason);
      }

      if (results[1].status === 'fulfilled') {
        setTocReport(results[1].value);
      } else {
        logger.error('FrmCryptoViewModel', 'Failed dumping FRM TOC', results[1].reason);
      }

      if (results[2].status === 'fulfilled') {
        setCodebookReport(results[2].value);
      } else {
        logger.error('FrmCryptoViewModel', 'Failed getting codebook', results[2].reason);
      }

      // If primary cryptReport failed, record error
      if (results[0].status === 'rejected') {
        const err = results[0].reason;
        setError(err instanceof Error ? err.message : String(err));
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
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
    error,
    codebookFilter,
    setCodebookFilter,
    refresh: loadData,
  };
}
