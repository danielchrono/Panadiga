import { useState, useEffect, useCallback, useMemo } from 'react';
import {
  HddAnalysisReport,
  MeihdfsAllocationReport,
  PartitionExtractionManifest,
} from '@/models';
import { invokeCommand } from '@/services/ipc';

export type HddTab = 'overview' | 'mbr' | 'meihdfs' | 'journal' | 'density' | 'extract';

export function useHddStorageViewModel() {
  const [hddReport, setHddReport] = useState<HddAnalysisReport | null>(null);
  const [meihdfsReport, setMeihdfsReport] = useState<MeihdfsAllocationReport | null>(null);
  const [activeTab, setActiveTab] = useState<HddTab>('overview');
  const [loading, setLoading] = useState<boolean>(false);
  const [extracting, setExtracting] = useState<boolean>(false);
  const [manifest, setManifest] = useState<PartitionExtractionManifest | null>(null);
  const [tagFilter, setTagFilter] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState<string>('');

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [hddData, meihdfsData] = await Promise.all([
        invokeCommand<HddAnalysisReport>('inspect_hdd'),
        invokeCommand<MeihdfsAllocationReport>('decode_meihdfs_allocations'),
      ]);
      setHddReport(hddData);
      setMeihdfsReport(meihdfsData);
    } catch (err) {
      console.error('Failed loading HDD forensics:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const extractPartitions = useCallback(async () => {
    setExtracting(true);
    try {
      const res = await invokeCommand<PartitionExtractionManifest>('extract_partitions');
      setManifest(res);
      setActiveTab('extract');
    } catch (err) {
      console.error('Failed extracting partitions:', err);
    } finally {
      setExtracting(false);
    }
  }, []);

  const filteredRecords = useMemo(() => {
    if (!meihdfsReport) return [];
    return meihdfsReport.records.filter(rec => {
      const matchesTag = tagFilter === 'all' || rec.tag_name.includes(tagFilter);
      const matchesSearch =
        searchQuery === '' ||
        rec.raw_hex.includes(searchQuery.toUpperCase()) ||
        rec.inverted_hex.includes(searchQuery.toUpperCase()) ||
        rec.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        rec.index.toString() === searchQuery;
      return matchesTag && matchesSearch;
    });
  }, [meihdfsReport, tagFilter, searchQuery]);

  return {
    hddReport,
    meihdfsReport,
    filteredRecords,
    activeTab,
    setActiveTab,
    loading,
    extracting,
    manifest,
    tagFilter,
    setTagFilter,
    searchQuery,
    setSearchQuery,
    extractPartitions,
    refresh: loadData,
  };
}
