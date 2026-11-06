/**
 * Hex Inspector & Binary Navigation Models
 */

export interface HexDumpRow {
  offset: number;
  offset_hex: string;
  bytes: number[];
  hex_part1: string; // 8 bytes
  hex_part2: string; // 8 bytes
  ascii: string;
}

export interface HexBookmark {
  id: string;
  label: string;
  category: 'MBR' | 'PARTITION' | 'JOURNAL' | 'FLASH' | 'FRM' | 'TOC';
  offset: number;
  offset_hex: string;
  description: string;
}

export interface HexInspectionState {
  file_path: string;
  file_name: string;
  file_size: number;
  current_offset: number;
  lba_number?: number;
  bytes_per_row: 16;
  total_rows: number;
  cursor_byte_offset?: number;
  selection_range?: [number, number];
}
