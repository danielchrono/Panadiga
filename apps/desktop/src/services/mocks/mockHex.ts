import { HexBookmark, HexDumpRow } from '@/models';

export const mockHexBookmarks: HexBookmark[] = [
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
    offset_hex: '0x01c00000',
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
    description: 'Início dos 8 microsetores de parâmetros corrompidos da Spansion Flash.',
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

export const generateMockHexDump = (startOffset: number, rowCount: number = 32): HexDumpRow[] => {
  const rows: HexDumpRow[] = [];

  for (let r = 0; r < rowCount; r++) {
    const offset = startOffset + r * 16;
    const bytes: number[] = [];

    // Synthesize realistic bytes based on target address
    for (let c = 0; c < 16; c++) {
      const cur = offset + c;
      if (cur < 16) {
        // LBA 0 MBR signature: 00 00 00 00 01 00 53 48 ...
        const mbrSig = [0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x53, 0x48, 0x62, 0x8c, 0x6e, 0xf5, 0x1b, 0x2c, 0x36, 0x66];
        bytes.push(mbrSig[cur] ?? 0x00);
      } else if (cur >= 0x02000000 && cur < 0x02000010) {
        // Partition 1 magic: 51 87 13 8D 5B 0A FB 22 ...
        const p1Sig = [0x51, 0x87, 0x13, 0x8d, 0x5b, 0x0a, 0xfb, 0x22, 0x00, 0x00, 0x00, 0x01, 0x00, 0x5e, 0x32, 0x50];
        bytes.push(p1Sig[cur - 0x02000000]);
      } else if (cur >= 0x007f0000 && cur < 0x007f0010) {
        // SA134 NVRAM header: AA 55 4D 45 49 4E 56 52 ...
        const nvSig = [0xaa, 0x55, 0x4d, 0x45, 0x49, 0x4e, 0x56, 0x52, 0x01, 0x00, 0x20, 0x00, 0x55, 0x99, 0xff, 0xff];
        bytes.push(nvSig[cur - 0x007f0000]);
      } else {
        // Deterministic pseudo-random bytes based on offset
        const pseudo = ((cur * 37 + 101) ^ (cur >> 8)) & 0xff;
        bytes.push(pseudo);
      }
    }

    const hex1 = bytes.slice(0, 8).map(b => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');
    const hex2 = bytes.slice(8, 16).map(b => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');
    const ascii = bytes.map(b => (b >= 32 && b <= 126 ? String.fromCharCode(b) : '.')).join('');

    rows.push({
      offset,
      offset_hex: `0x${offset.toString(16).padStart(8, '0').toUpperCase()}`,
      bytes,
      hex_part1: hex1,
      hex_part2: hex2,
      ascii,
    });
  }

  return rows;
};
