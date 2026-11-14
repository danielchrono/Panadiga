import enUSJson from '@/locales/en-US.json';
import ptBRJson from '@/locales/pt-BR.json';
import type { HexBookmark } from '@/models';
import { SPANSION_NVRAM_BASE_OFFSET, type SupportedLocale } from './hardwareSpecs';

type BookmarkLocaleKey = keyof typeof ptBRJson.hex.bookmarkItems;

interface HexBookmarkBlueprint {
  readonly id: string;
  readonly localeKey: BookmarkLocaleKey;
  readonly category: HexBookmark['category'];
  readonly offset: number;
  readonly offset_hex: string;
}

const HEX_BOOKMARK_BLUEPRINTS: readonly HexBookmarkBlueprint[] = Object.freeze([
  { id: 'mbr-lba0', localeKey: 'mbr_lba0', category: 'MBR', offset: 0x00000000, offset_hex: '0x00000000' },
  {
    id: 'descriptors-lba24',
    localeKey: 'descriptors_lba24',
    category: 'PARTITION',
    offset: 0x00003000,
    offset_hex: '0x00003000',
  },
  {
    id: 'journal-wal-8mb',
    localeKey: 'journal_wal_8mb',
    category: 'JOURNAL',
    offset: 0x00800000,
    offset_hex: '0x00800000',
  },
  {
    id: 'mirror-mbr-28mb',
    localeKey: 'mirror_mbr_28mb',
    category: 'MBR',
    offset: 0x01c00000,
    offset_hex: '0x01C00000',
  },
  {
    id: 'partition1-32mb',
    localeKey: 'partition1_32mb',
    category: 'PARTITION',
    offset: 0x02000000,
    offset_hex: '0x02000000',
  },
  {
    id: 'partition2-64mb',
    localeKey: 'partition2_64mb',
    category: 'PARTITION',
    offset: 0x04000000,
    offset_hex: '0x04000000',
  },
  {
    id: 'nvram-sa134',
    localeKey: 'nvram_sa134',
    category: 'FLASH',
    offset: SPANSION_NVRAM_BASE_OFFSET,
    offset_hex: '0x007F0000',
  },
  {
    id: 'frm-toc-optical',
    localeKey: 'frm_toc_optical',
    category: 'TOC',
    offset: 0x00000010,
    offset_hex: '0x00000010',
  },
]);

/**
 * Pure builder returning physical DMR-EH55 hex bookmarks localized via i18next dictionaries.
 */
export function getHexForensicBookmarks(locale: SupportedLocale = 'pt-BR'): HexBookmark[] {
  const dict = locale === 'en-US' ? enUSJson.hex.bookmarkItems : ptBRJson.hex.bookmarkItems;
  return HEX_BOOKMARK_BLUEPRINTS.map(bp => ({
    id: bp.id,
    label: dict[bp.localeKey].label,
    category: bp.category,
    offset: bp.offset,
    offset_hex: bp.offset_hex,
    description: dict[bp.localeKey].description,
  }));
}

export const HEX_FORENSIC_BOOKMARKS: readonly HexBookmark[] = Object.freeze(
  getHexForensicBookmarks('pt-BR')
);

export const DIGA_HEX_BOOKMARKS: HexBookmark[] = getHexForensicBookmarks('pt-BR');
