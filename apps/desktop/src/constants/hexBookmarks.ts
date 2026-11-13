import { HexBookmark } from '@/models';

/**
 * Endereços físicos verificados no Hardware Panasonic DMR-EH55 (HDD MEIHDFS, Spansion Flash e PANA_DVD.FRM).
 */
export const DIGA_HEX_BOOKMARKS: HexBookmark[] = [
  {
    id: 'mbr-lba0',
    label: 'LBA 0: MBR Primário ("SH")',
    category: 'MBR',
    offset: 0x00000000,
    offset_hex: '0x00000000',
    description: 'Assinatura proprietária Panasonic MEIHDFS e GUID do disco rígido.',
  },
  {
    id: 'descriptors-lba24',
    label: 'LBA 24: Descritores de Alocação',
    category: 'PARTITION',
    offset: 0x00003000,
    offset_hex: '0x00003000',
    description: 'Tabela de 256 registros MEIHDFS codificados com máscara XOR 0xFF.',
  },
  {
    id: 'journal-wal-8mb',
    label: '8 MB: Catálogo do Journal WAL',
    category: 'JOURNAL',
    offset: 0x00800000,
    offset_hex: '0x00800000',
    description: 'Início do anel de 10 blocos de 128 KB de transações transacionais.',
  },
  {
    id: 'mirror-mbr-28mb',
    label: '28 MB: Espelho LBA 57.344 do MBR',
    category: 'MBR',
    offset: 0x01c00000,
    offset_hex: '0x01C00000',
    description: 'Backup bit-a-bit idêntico ao Setor 0 para recuperação de emergência.',
  },
  {
    id: 'partition1-32mb',
    label: '32 MB: Partição 1 (AM34 OS)',
    category: 'PARTITION',
    offset: 0x02000000,
    offset_hex: '0x02000000',
    description: 'Magic "51 87 13 8D 5B 0A FB 22" e código executável do gravador.',
  },
  {
    id: 'partition2-64mb',
    label: '64 MB: Partição 2 (Fontes/Microcódigo)',
    category: 'PARTITION',
    offset: 0x04000000,
    offset_hex: '0x04000000',
    description: 'Microcódigos dos co-processadores e fontes bitmap OSD.',
  },
  {
    id: 'nvram-sa134',
    label: 'Spansion SA134: NVRAM / U99 Area',
    category: 'FLASH',
    offset: 0x007f0000,
    offset_hex: '0x007F0000',
    description: 'Início dos 8 microsetores de parâmetros da EEPROM emulada na Spansion Flash.',
  },
  {
    id: 'frm-toc-optical',
    label: 'PANA_DVD.FRM: TOC Entry AM34',
    category: 'TOC',
    offset: 0x00000010,
    offset_hex: '0x00000010',
    description: 'Descritor periódico de 48 bytes da CPU AM34 no contêiner óptico.',
  },
];
