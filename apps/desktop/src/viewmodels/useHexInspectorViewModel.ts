import { useState, useEffect, useCallback } from 'react';
import { HexDumpRow, HexBookmark } from '@/models';
import { invokeCommand } from '@/services/ipc';
import { mockHexBookmarks } from '@/services/mockData';

export function useHexInspectorViewModel() {
  const [currentOffset, setCurrentOffset] = useState<number>(0);
  const [hexRows, setHexRows] = useState<HexDumpRow[]>([]);
  const [bookmarks] = useState<HexBookmark[]>(mockHexBookmarks);
  const [selectedBookmark, setSelectedBookmark] = useState<HexBookmark | null>(mockHexBookmarks[0]);
  const [jumpInput, setJumpInput] = useState<string>('0x00000000');
  const [loading, setLoading] = useState<boolean>(false);
  const rowsPerPage = 24;

  const fetchRows = useCallback(async (offset: number) => {
    setLoading(true);
    try {
      const data = await invokeCommand<HexDumpRow[]>('read_hex_dump', {
        offset,
        count: rowsPerPage,
      });
      setHexRows(data);
    } catch (err) {
      console.error('Failed reading hex dump:', err);
    } finally {
      setLoading(false);
    }
  }, [rowsPerPage]);

  useEffect(() => {
    fetchRows(currentOffset);
  }, [currentOffset, fetchRows]);

  const jumpToOffset = useCallback((offset: number) => {
    const cleanOffset = Math.max(0, offset - (offset % 16));
    setCurrentOffset(cleanOffset);
    setJumpInput(`0x${cleanOffset.toString(16).toUpperCase()}`);
  }, []);

  const jumpToLba = useCallback((lba: number) => {
    const byteOffset = lba * 512;
    jumpToOffset(byteOffset);
  }, [jumpToOffset]);

  const selectBookmark = useCallback((bm: HexBookmark) => {
    setSelectedBookmark(bm);
    jumpToOffset(bm.offset);
  }, [jumpToOffset]);

  const nextPage = useCallback(() => {
    jumpToOffset(currentOffset + rowsPerPage * 16);
  }, [currentOffset, rowsPerPage, jumpToOffset]);

  const prevPage = useCallback(() => {
    jumpToOffset(Math.max(0, currentOffset - rowsPerPage * 16));
  }, [currentOffset, rowsPerPage, jumpToOffset]);

  const handleJumpSubmit = useCallback(() => {
    let parsed = 0;
    const trimmed = jumpInput.trim();
    if (trimmed.toLowerCase().startsWith('0x')) {
      parsed = parseInt(trimmed.substring(2), 16);
    } else {
      parsed = parseInt(trimmed, 10);
    }
    if (!isNaN(parsed)) {
      jumpToOffset(parsed);
    }
  }, [jumpInput, jumpToOffset]);

  return {
    currentOffset,
    currentLba: Math.floor(currentOffset / 512),
    hexRows,
    bookmarks,
    selectedBookmark,
    jumpInput,
    setJumpInput,
    loading,
    selectBookmark,
    jumpToOffset,
    jumpToLba,
    handleJumpSubmit,
    nextPage,
    prevPage,
    refresh: () => fetchRows(currentOffset),
  };
}
