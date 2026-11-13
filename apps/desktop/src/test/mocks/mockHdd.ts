import { HddAnalysisReport, MeihdfsAllocationReport } from '@/models';

export const mockHddAnalysisReport: HddAnalysisReport = {
  image_path: 'firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin',
  file_size_bytes: 104857600,
  file_size_mb: 100.0,
  total_sectors: 204800,
  global_md5: 'e28bbca560beee9c849c719e09d57a92',
  global_sha256: '9f81a7bca31a4773da2846f3323081e7d0cf03bc88a8ea944f2b1d6e87f3910c',
  mbr_tables: {
    lba0_offset: 0,
    lba0_hex_signature: '00 00 00 00 01 00 53 48',
    format_version: 1,
    magic_ascii: 'SH (System Storage Header)',
    disk_volume_guid: '628C6EF5-1B2C-3666-A79B-89E198302194',
    has_standard_mbr_55aa: false,
    zeros_after_header_clean: true,
    lba0_md5: '7d389a9f24b4f5ee35d94e223aa900ef',
    lba0_sha256: 'b1484831617db84f5f84d6aa1192e212faec1911993424d86b77242d5f0e9fa0',
    mirror_lba: 57344,
    mirror_offset: 0x01c00000,
    mirror_is_bit_identical: true,
    mirror_md5: '7d389a9f24b4f5ee35d94e223aa900ef',
    mirror_sha256: 'b1484831617db84f5f84d6aa1192e212faec1911993424d86b77242d5f0e9fa0',
  },
  allocation_and_journal: {
    descriptors_offset: 0x00003000,
    descriptors_size: 4096,
    descriptors_mirror_identical: true,
    descriptors_sha256: '38a9bc71b8ee20109fa7719ddaa9203948501abdd2934091a9f0293481239aa1',
    journal_catalog_offset: 0x00800000,
    journal_total_blocks: 10,
    journal_max_index: 9,
    journal_block_trailers: [
      '57414C5F424C4B30',
      '57414C5F424C4B31',
      '57414C5F424C4B32',
      '57414C5F424C4B33',
      '57414C5F424C4B34',
      '57414C5F424C4B35',
      '57414C5F424C4B36',
      '57414C5F424C4B37',
      '57414C5F424C4B38',
      '57414C5F424C4B39',
    ],
    state_marker_offset: 0x01800000,
    state_marker_hex: '50414E415F53544154455F5245414459',
  },
  partitions: [
    {
      partition_id: 1,
      name: 'Partition 1: AM34 OS, Main System & Codec Executable',
      offset: 0x02000000,
      offset_hex: '0x02000000',
      lba_start: 65536,
      exact_payload_size: 6173264,
      sector_aligned_size: 6173696,
      sectors_count: 12058,
      signature_hex: '51 87 13 8D 5B 0A FB 22',
      entropy: 7.9942,
      exact_md5: '72dae89b21f3089daef3810237aa01e2',
      exact_sha256: '499e1d8820cbf018a77d1219eeea103980bc235e182390aef92348a011234bc9',
      aligned_md5: '8839dfaa192837482930219485720194',
      aligned_sha256: 'aa91283749201928340192834019283401928340192834019283401928340192',
      header_hex_64: '51 87 13 8D 5B 0A FB 22 00 00 00 01 00 5E 32 50 00 00 00 00 00 00 00 00 E5 D1 F3 4A 51 8B 26 90',
    },
    {
      partition_id: 2,
      name: 'Partition 2: Secondary Firmware, Microcode & OSD Fonts',
      offset: 0x04000000,
      offset_hex: '0x04000000',
      lba_start: 131072,
      exact_payload_size: 720896,
      sector_aligned_size: 720896,
      sectors_count: 1408,
      signature_hex: '51 87 13 8D 5B 0A FB 22',
      entropy: 7.8214,
      exact_md5: '3c8919f2010938472910384719283746',
      exact_sha256: '7721839048102938471920394857102938471920394857102938471920394857',
      aligned_md5: '3c8919f2010938472910384719283746',
      aligned_sha256: '7721839048102938471920394857102938471920394857102938471920394857',
      header_hex_64: '51 87 13 8D 5B 0A FB 22 00 00 00 02 00 0B 00 00 00 00 00 00 00 00 00 03 50 65 C8 7B B8 E0 90',
    },
  ],
  density_map: Array.from({ length: 100 }, (_, i) => {
    let pct = 0;
    if (i === 0) pct = 4.2; // MBR & Descriptors
    else if (i >= 8 && i <= 9) pct = 98.5; // Journal WAL
    else if (i === 24) pct = 2.0; // State marker
    else if (i === 28) pct = 4.2; // Mirror MBR
    else if (i >= 32 && i <= 37) pct = 99.8; // Partition 1
    else if (i === 64) pct = 70.4; // Partition 2
    return {
      mb_index: i,
      offset: i * 1048576,
      non_zero_bytes: Math.round((pct / 100) * 1048576),
      percentage: pct,
    };
  }),
};

// 256 records decoded with XOR 0xFF
export const mockMeihdfsAllocationReport: MeihdfsAllocationReport = {
  source_path: 'firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin',
  is_full_hdd_image: true,
  is_mirror_verified: true,
  descriptors_sha256: '38a9bc71b8ee20109fa7719ddaa9203948501abdd2934091a9f0293481239aa1',
  total_records: 256,
  tag_distribution: {
    'Extent Record (0x20)': 112,
    'Directory Descriptor (0x30)': 38,
    'Root Volume Anchor (0x01)': 4,
    'WAL Journal Pointer (0x10)': 10,
    'Unassigned / Free (0x00)': 92,
  },
  records: Array.from({ length: 256 }, (_, idx) => {
    let rawTag = 0xff;
    let decodedTag = 0x00;
    let tagName = 'Unassigned / Free';
    let desc = 'Registro vazio ou liberado na tabela de alocação MEIHDFS';
    let extId: number | undefined;
    let secCount: number | undefined;
    let lbaOff: number | undefined;

    if (idx === 0) {
      rawTag = 0xfe;
      decodedTag = 0x01;
      tagName = 'Root Volume Anchor';
      desc = 'Âncora principal da partição MEIHDFS (Panasonic DIGA Master Root)';
    } else if (idx >= 1 && idx <= 10) {
      rawTag = 0xef;
      decodedTag = 0x10;
      tagName = 'WAL Journal Pointer';
      desc = `Ponteiro do Bloco de Transação Journal WAL #${idx - 1} (Offset 0x00820000)`;
    } else if (idx >= 11 && idx <= 80) {
      rawTag = 0xdf;
      decodedTag = 0x20;
      tagName = 'Extent Record';
      extId = 100 + idx;
      secCount = 2048 + (idx * 512);
      lbaOff = 65536 + (idx * 1024);
      desc = `Segmento Contíguo Extent #${extId} - ${secCount} setores (LBA ${lbaOff})`;
    } else if (idx >= 81 && idx <= 118) {
      rawTag = 0xcf;
      decodedTag = 0x30;
      tagName = 'Directory Descriptor';
      desc = `Diretório do Catálogo MEIHDFS - Nó #${idx - 80}`;
    }

    const rawHex = `${rawTag.toString(16).padStart(2, '0').toUpperCase()} 12 34 56 78 9A BC DE F0 11 22 33 44 55 66 77`;
    const invertedHex = `${decodedTag.toString(16).padStart(2, '0').toUpperCase()} ED CB A9 87 65 43 21 0F EE DD CC BB AA 99 88`;

    return {
      index: idx,
      offset_in_table: idx * 16,
      raw_hex: rawHex,
      inverted_hex: invertedHex,
      raw_tag: rawTag,
      decoded_tag: decodedTag,
      tag_name: tagName,
      extent_id: extId,
      sector_count: secCount,
      lba_offset: lbaOff,
      flags: 0x0001,
      tail_hex: '44 55 66 77',
      description: desc,
    };
  }),
  extents: [
    {
      extent_id: 1,
      extent_id_hex: '0x0001',
      header_record_index: 11,
      total_sectors: 12058,
      total_bytes: 6173696,
      segments_count: 2,
      segments: [
        {
          record_index: 11,
          subtag: 0x21,
          sector_count: 8192,
          lba_offset_hint: 65536,
          flags: 0x0001,
          tail_hex: '00 00 00 00',
        },
        {
          record_index: 12,
          subtag: 0x22,
          sector_count: 3866,
          lba_offset_hint: 73728,
          flags: 0x0000,
          tail_hex: '00 00 00 00',
        },
      ],
    },
    {
      extent_id: 2,
      extent_id_hex: '0x0002',
      header_record_index: 13,
      total_sectors: 1408,
      total_bytes: 720896,
      segments_count: 1,
      segments: [
        {
          record_index: 13,
          subtag: 0x21,
          sector_count: 1408,
          lba_offset_hint: 131072,
          flags: 0x0001,
          tail_hex: '00 00 00 00',
        },
      ],
    },
  ],
  journal_blocks: Array.from({ length: 10 }, (_, i) => ({
    block_index: i,
    start_lba: 16384 + (i * 256),
    start_offset_hex: `0x${(0x00820000 + (i * 131072)).toString(16).toUpperCase()}`,
    is_active_header: true,
    header_hex_16: `57 41 4C 5F 42 4C 4B 3${i} 00 00 01 0${i} 00 00 02 00`,
    transaction_seq: 1042 + i,
    pool_size: 131072,
    checksum_crc: `0x${(0xA1B2C3D0 + i * 17).toString(16).toUpperCase()}`,
    trailer_offset_hex: `0x${(0x00820000 + ((i + 1) * 131072) - 16).toString(16).toUpperCase()}`,
    active_payload_sectors: 128,
  })),
};
