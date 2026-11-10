import { useState, useEffect, useCallback, useMemo } from 'react';
import {
  HddAnalysisReport,
  MeihdfsAllocationReport,
  PartitionExtractionManifest,
} from '@/models';
import { invokeCommand } from '@/services/ipc';
import { logger } from '@/utils/logger';

export type HddTab = 'overview' | 'mbr' | 'meihdfs' | 'journal' | 'density' | 'extract' | 'writer';

export function useHddStorageViewModel() {
  const [hddReport, setHddReport] = useState<HddAnalysisReport | null>(null);
  const [meihdfsReport, setMeihdfsReport] = useState<MeihdfsAllocationReport | null>(null);
  const [activeTab, setActiveTab] = useState<HddTab>('overview');
  const [loading, setLoading] = useState<boolean>(false);
  const [extracting, setExtracting] = useState<boolean>(false);
  const [manifest, setManifest] = useState<PartitionExtractionManifest | null>(null);
  const [tagFilter, setTagFilter] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [imagePath, setImagePath] = useState<string>('');
  const [error, setError] = useState<string | null>(null);

  const loadData = useCallback(async (customPath?: string) => {
    setLoading(true);
    setError(null);
    const targetPath = customPath !== undefined ? customPath : imagePath;
    const args = targetPath.trim() ? { image_path: targetPath.trim() } : {};
    try {
      const [hddData, meihdfsData] = await Promise.all([
        invokeCommand<HddAnalysisReport>('inspect_hdd', args),
        invokeCommand<MeihdfsAllocationReport>('decode_meihdfs_allocations', args),
      ]);
      setHddReport(hddData);
      setMeihdfsReport(meihdfsData);
      if (hddData?.image_path) {
        setImagePath(hddData.image_path);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
      logger.warn('HddStorageViewModel', 'Failed loading HDD forensics', err);
    } finally {
      setLoading(false);
    }
  }, [imagePath]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const extractPartitions = useCallback(async () => {
    setExtracting(true);
    try {
      const args = imagePath.trim() ? { image_path: imagePath.trim() } : {};
      const res = await invokeCommand<PartitionExtractionManifest>('extract_partitions', args);
      setManifest(res);
      setActiveTab('extract');
    } catch (err) {
      logger.error('HddStorageViewModel', 'Failed extracting partitions', err);
    } finally {
      setExtracting(false);
    }
  }, [imagePath]);

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
    imagePath,
    setImagePath,
    error,
    extractPartitions,
    refresh: loadData,
  };
}
