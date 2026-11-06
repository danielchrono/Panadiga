import {
  NvramCleanReport,
  FlashSectorInfo,
  HddAnalysisReport,
  MeihdfsAllocationReport,
  CryptanalysisReport,
  FrmTocDumpReport,
  CodebookReport,
  PicoDeviceStatus,
  SerialLogEntry,
  ManualSearchResult,
  ManualDocumentMetadata,
  SystemOverview,
  HexDumpRow,
  HexBookmark,
} from '@/models';

// ============================================================================
// 1. SYSTEM OVERVIEW
// ============================================================================

export const mockSystemOverview: SystemOverview = {
  product_model: 'Panasonic DMR-EH55 / DMR-EH55P / DMR-EH55PC',
  soc_processor: 'Matsushita MN2DS0016 / UniPhier Digital SoC',
  core_architecture: 'AM34 (32-bit RISC) + MN103S + MN102 DSP Subsystems',
  os_system: 'Panasonic Custom Embedded OS (Big-Endian ELF / MIPS-like AM34)',
  flash_rom: 'Spansion S29JL064H (8 MB / 64 Mbit, Top-Boot TSOP-48)',
  hdd_specs: 'Hitachi Deskstar IDE/PATA 200 GB (512B/Sector, MEIHDFS)',
  optical_drive: 'Panasonic VXY1945 RAM/DVD±R/RW Multi Drive',
  debug_port: 'CN55001 (DWire32A OCD / On-Chip Debug Single-Wire)',
  error_code: 'U99 (Fatal System Communication Timeout / Corrupt NVRAM Param)',
  error_description: 'Self-Check failure: EEPROM Emulation microsectors in Spansion top sector SA134..SA141 contain invalid boot tokens, causing endless reboot loop.',
  firmware_version: '08.40 (Build 060412)',
  binaries: [
    {
      id: 'hdd-100mb',
      name: 'panasonic DMR EH-55 Firmware hdd_100.bin',
      path: 'firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin',
      exists: true,
      size_bytes: 104857600,
      formatted_size: '100.00 MB',
      expected_role: 'Hitachi PATA 100MB Head Dump (MEIHDFS MBR, Journal & Firmware Partitions)',
      status: 'ready',
    },
    {
      id: 'frm-optical',
      name: 'PANA_DVD.FRM',
      path: 'firmware_and_images/PANA_DVD.FRM',
      exists: true,
      size_bytes: 7531744,
      formatted_size: '7.18 MB',
      expected_role: 'Official Optical Firmware Container (Encrypted AM34/MN103S Payloads)',
      status: 'ready',
    },
    {
      id: 'spansion-flash',
      name: 'spansion_s29jl064h_dump.bin',
      path: 'firmware_and_images/spansion_s29jl064h_dump.bin',
      exists: true,
      size_bytes: 8388608,
      formatted_size: '8.00 MB',
      expected_role: 'Spansion S29JL064H Physical Flash ROM Dump (TSOP-48)',
      status: 'warning',
    },
    {
      id: 'manuals-dir',
      name: 'manuals_and_schematics/ (15 arquivos)',
      path: 'manuals_and_schematics',
      exists: true,
      size_bytes: 65123904,
      formatted_size: '62.1 MB (15 PDFs)',
      expected_role: 'Service Manuals, Circuit Diagrams & Engineering Bulletins',
      status: 'ready',
    },
    {
      id: 'dwire32a-pico',
      name: 'dwire32a_pico_firmware.uf2',
      path: 'dwire32a_jtag_research/build/dwire32a_pico.uf2',
      exists: true,
      size_bytes: 49152,
      formatted_size: '48 KB',
      expected_role: 'Raspberry Pi Pico RP2040 DWire32A Bus Master & OCD Injection',
      status: 'ready',
    },
  ],
};

// ============================================================================
// 2. NVRAM & SPANSION FLASH CLEAN REPORT
// ============================================================================

export const mockNvramCleanReport: NvramCleanReport = {
  input_file: 'firmware_and_images/spansion_s29jl064h_dump.bin',
  output_file: 'firmware_and_images/spansion_s29jl064h_dump_CLEAN.bin',
  backup_file: 'firmware_and_images/nvram_corrupt_backup.bin',
  original_md5: '7b9e4a91c849102c7e09ab12f458bc21',
  original_sha256: 'e8d62c140409a25b29339e088d1d8ef3f24b22c83ef87340ad0098dfec11aa52',
  clean_md5: '4a11f99c2d1b76402ea891001f357aa9',
  clean_sha256: '0a394c8e7710bcf1489e3a6a92ecda46e01768494c8fa72722ee20c5c363dc78',
  total_flash_bytes: 8388608,
  nvram_offset_start: 0x007f0000,
  nvram_offset_end: 0x00800000,
  nvram_non_ff_bytes: 18432,
  nvram_non_zero_bytes: 65536,
  bootloader_immutable: true,
  microsectors: [
    {
      sector_name: 'SA134',
      offset: 0x007f0000,
      active_bytes: 2048,
      total_bytes: 8192,
      header_hex: 'AA554D45494E5652',
    },
    {
      sector_name: 'SA135',
      offset: 0x007f2000,
      active_bytes: 1536,
      total_bytes: 8192,
      header_hex: '45454D554C010020',
    },
    {
      sector_name: 'SA136',
      offset: 0x007f4000,
      active_bytes: 3072,
      total_bytes: 8192,
      header_hex: '53454C4643484B31',
    },
    {
      sector_name: 'SA137',
      offset: 0x007f6000,
      active_bytes: 4096,
      total_bytes: 8192,
      header_hex: '5539395F45525230',
    },
    {
      sector_name: 'SA138',
      offset: 0x007f8000,
      active_bytes: 2560,
      total_bytes: 8192,
      header_hex: '4844445F53455249',
    },
    {
      sector_name: 'SA139',
      offset: 0x007fa000,
      active_bytes: 1024,
      total_bytes: 8192,
      header_hex: '524547494F4E5F55',
    },
    {
      sector_name: 'SA140',
      offset: 0x007fc000,
      active_bytes: 2048,
      total_bytes: 8192,
      header_hex: '43484B53554D5F45',
    },
    {
      sector_name: 'SA141',
      offset: 0x007fe000,
      active_bytes: 2048,
      total_bytes: 8192,
      header_hex: '544553545F544F4B',
    },
  ],
};

// Interactive Flash Map entries representing SA0..SA141
export const mockFlashSectorMap: FlashSectorInfo[] = Array.from({ length: 142 }, (_, i) => {
  const isMicroSector = i >= 134;
  const isBootloader = i < 16;
  const isOsFirmware = i >= 16 && i < 134;

  const startOffset = isMicroSector
    ? 0x007f0000 + (i - 134) * 8192
    : i * 65536;
  const sizeBytes = isMicroSector ? 8192 : 65536;
  const endOffset = startOffset + sizeBytes - 1;

  const sectorType = isMicroSector
    ? 'nvram_microsector'
    : isBootloader
    ? 'bootloader'
    : isOsFirmware
    ? 'os_firmware'
    : 'normal_sector';

  return {
    index: i,
    name: `SA${i}`,
    start_offset: startOffset,
    end_offset: endOffset,
    size_bytes: sizeBytes,
    sector_type: sectorType,
    is_corrupted_area: isMicroSector,
    status: isMicroSector ? 'corrupted' : isBootloader ? 'immutable' : 'clean',
    active_bytes: isMicroSector ? (i === 137 ? 4096 : 2048) : sizeBytes,
    notes: isMicroSector
      ? `Top Parameter Bank: SA${i} (EEMUL / U99 Root Cause)`
      : isBootloader
      ? `Immutable Primary AM34 Bootloader Bank (0x${startOffset.toString(16).toUpperCase()})`
      : `Main OS & Codec Executable Segment`,
  };
});

// ============================================================================
// 3. HDD STORAGE & MEIHDFS ANALYSIS
// ============================================================================

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
      header_hex_64: '51 87 13 8D 5B 0A FB 22 00 00 00 02 00 0B 00 00 00 00 00 00 00 00 00 00 03 50 65 C8 7B B8 E0 90',
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

// ============================================================================
// 4. CRYPTOGRAPHIC FORENSICS & PANA_DVD.FRM
// ============================================================================

export const mockCryptanalysisReport: CryptanalysisReport = {
  frm_file: 'firmware_and_images/PANA_DVD.FRM',
  file_size_bytes: 7531744,
  global_entropy: 7.99958,
  md5: '52bfbcfbaee85c2c7f1a5e6789e8a23b',
  sha256: '6097ea3b8c910248a77b81920384729103847291038472910384729103847291',
  is_ecb_leak_confirmed: true,
  top_blocks: [
    {
      hex: '03 50 65 C8 7B B8 E0 90',
      count: 3812,
      total_bytes: 30496,
      interpretation: 'Preenchimento Virgem Flash ROM (0xFF * 8) - Confirmação Direta de Cifra ECB de 64 bits',
    },
    {
      hex: '4D 3C 5D BB 0C 35 77 77',
      count: 1280,
      total_bytes: 10240,
      interpretation: 'Estrutura Fixa de Cabeçalho / Tabela Alinhada Periódica',
    },
    {
      hex: 'E5 D1 F3 4A 51 8B 26 90',
      count: 196,
      total_bytes: 1568,
      interpretation: 'Bloco de Zeros Nulos / Padding (0x00 * 8) - Idêntico ao Padding da Partição 1 do HD',
    },
    {
      hex: '8A 1F C4 D0 11 9B 42 E3',
      count: 74,
      total_bytes: 592,
      interpretation: 'Padrão ECB Cifrado de Alta Repetição (AM34 Subroutine Prologue)',
    },
    {
      hex: '19 72 A0 BB 4E 83 91 FF',
      count: 52,
      total_bytes: 416,
      interpretation: 'Padrão ECB Cifrado de Alta Repetição (DSP Filter Coefficients)',
    },
  ],
  toc_analysis: {
    entry_size_bytes: 48,
    total_examined_records: 10,
    common_subblock_1_hex: '4D 3C 5D BB 0C 35 77 77',
    common_subblock_1_matches: 8,
    common_subblock_2_hex: 'E5 D1 F3 4A 51 8B 26 90',
    common_subblock_2_matches: 6,
  },
  hdd_correlations: [
    {
      partition_name: 'HDD Partition 1 (AM34 OS)',
      offset_hex: '0x02000018',
      block_hex: 'E5 D1 F3 4A 51 8B 26 90',
      interpretation: 'Padding de 8 Zeros idêntico ao encontrado 196 vezes no PANA_DVD.FRM',
      found_in_frm: true,
      frm_occurrences: 196,
    },
    {
      partition_name: 'HDD Partition 2 (Fonts/Microcode)',
      offset_hex: '0x04000018',
      block_hex: '03 50 65 C8 7B B8 E0 90',
      interpretation: 'Flash padding virgem (0xFF*8) compartilhado entre HD e contêiner óptico',
      found_in_frm: true,
      frm_occurrences: 3812,
    },
  ],
};

export const mockFrmTocDumpReport: FrmTocDumpReport = {
  frm_file: 'firmware_and_images/PANA_DVD.FRM',
  file_size_bytes: 7531744,
  global_header_hex: '50 41 4E 41 5F 46 52 4D 00 01 08 40 00 00 00 03',
  toc_invariants_valid: true,
  toc_entries: [
    {
      index: 0,
      offset_hex: '0x00000010',
      target_name: 'AM34_MAIN_OS',
      target_architecture: 'Matsushita AM34 (UniPhier 32-bit RISC)',
      processor_family_id: '0x34A',
      target_descriptor_id: 'IC5001_MAIN_DIGITAL',
      raw_hex: '34 41 00 01 00 00 10 00 00 5E 32 50 4D 3C 5D BB 0C 35 77 77 E5 D1 F3 4A 51 8B 26 90 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00',
      subblocks: ['4D 3C 5D BB 0C 35 77 77', 'E5 D1 F3 4A 51 8B 26 90'],
    },
    {
      index: 1,
      offset_hex: '0x00000040',
      target_name: 'MN102_FRONT_PANEL',
      target_architecture: 'Panasonic MN102 (16-bit Sub-Processor)',
      processor_family_id: '0x102',
      target_descriptor_id: 'IC55007_FRONT_KEY',
      raw_hex: '01 02 00 02 00 5E 42 50 00 04 00 00 4D 3C 5D BB 0C 35 77 77 03 50 65 C8 7B B8 E0 90 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00',
      subblocks: ['4D 3C 5D BB 0C 35 77 77', '03 50 65 C8 7B B8 E0 90'],
    },
    {
      index: 2,
      offset_hex: '0x00000070',
      target_name: 'MN103S_DRIVE_DSP',
      target_architecture: 'Panasonic MN103S (Optical Drive Servo DSP)',
      processor_family_id: '0x103',
      target_descriptor_id: 'IC5002_OPTICAL_SERVO',
      raw_hex: '01 03 00 03 00 62 42 50 00 07 00 00 4D 3C 5D BB 0C 35 77 77 E5 D1 F3 4A 51 8B 26 90 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00',
      subblocks: ['4D 3C 5D BB 0C 35 77 77', 'E5 D1 F3 4A 51 8B 26 90'],
    },
  ],
  modules: [
    {
      module_index: 0,
      name: 'AM34 System Firmware Image',
      target: 'UniPhier AM34 Core (Digital Board)',
      start_offset_hex: '0x00001000',
      end_offset_hex: '0x005E4250',
      payload_size_bytes: 6173264,
      padding_size_bytes: 432,
      hdd_correlation: 'Corresponde exatamente à Partição 1 do HD (LBA 65536)',
    },
    {
      module_index: 1,
      name: 'MN102 Front Panel Microcode',
      target: 'MN102 Microcontroller',
      start_offset_hex: '0x005E4250',
      end_offset_hex: '0x00624250',
      payload_size_bytes: 262144,
      padding_size_bytes: 0,
      hdd_correlation: 'Microcódigo do Display VFD e botões de painel',
    },
    {
      module_index: 2,
      name: 'MN103S Optical Mecha DSP Code',
      target: 'MN103S DVD Servo DSP',
      start_offset_hex: '0x00624250',
      end_offset_hex: '0x00694250',
      payload_size_bytes: 458752,
      padding_size_bytes: 0,
      hdd_correlation: 'Código de controle da óptica laser e buffer RAM',
    },
  ],
};

export const mockCodebookReport: CodebookReport = {
  frm_file: 'firmware_and_images/PANA_DVD.FRM',
  hdd_file: 'firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin',
  total_64bit_blocks_analyzed: 941468,
  unique_blocks: 84210,
  is_ecb_leak_confirmed: true,
  cross_medium_key_identity_proven: true,
  dictionary: [
    {
      rank: 1,
      block_hex: '03 50 65 C8 7B B8 E0 90',
      known_plaintext_hex: 'FF FF FF FF FF FF FF FF',
      interpretation: 'Preenchimento Virgem de Flash ROM (0xFF * 8) - Prova KPA Incontestável',
      frm_occurrences: 3812,
      frm_total_bytes: 30496,
      hdd_occurrences: 512,
      hdd_total_bytes: 4096,
      sample_locations: ['PANA_DVD.FRM:0x005E4230', 'HDD_LBA:131074'],
    },
    {
      rank: 2,
      block_hex: '4D 3C 5D BB 0C 35 77 77',
      known_plaintext_hex: '00 00 00 00 00 00 00 01',
      interpretation: 'Cabeçalho de Tabela Periódica / Marcador de Bloco Alinhado',
      frm_occurrences: 1280,
      frm_total_bytes: 10240,
      hdd_occurrences: 128,
      hdd_total_bytes: 1024,
      sample_locations: ['PANA_DVD.FRM:0x00000018', 'PANA_DVD.FRM:0x00000048'],
    },
    {
      rank: 3,
      block_hex: 'E5 D1 F3 4A 51 8B 26 90',
      known_plaintext_hex: '00 00 00 00 00 00 00 00',
      interpretation: 'Bloco Nulo de Zeros / Padding (0x00 * 8)',
      frm_occurrences: 196,
      frm_total_bytes: 1568,
      hdd_occurrences: 48,
      hdd_total_bytes: 384,
      sample_locations: ['PANA_DVD.FRM:0x00000020', 'HDD_LBA:65538'],
    },
    {
      rank: 4,
      block_hex: '8A 1F C4 D0 11 9B 42 E3',
      known_plaintext_hex: '27 BD FF E0 AF BF 00 1C',
      interpretation: 'AM34 Prólogo de Função (`addiu sp, sp, -32; sw ra, 28(sp)`)',
      frm_occurrences: 74,
      frm_total_bytes: 592,
      hdd_occurrences: 74,
      hdd_total_bytes: 592,
      sample_locations: ['PANA_DVD.FRM:0x00001040', 'HDD_LBA:65540'],
    },
  ],
};

// ============================================================================
// 5. RASPBERRY PI PICO / DWIRE32A BENCH CONTROL
// ============================================================================

export const mockPicoDeviceStatus: PicoDeviceStatus = {
  connected: true,
  port: 'COM4 (RP2040 DWire32A Interface)',
  baudRate: 115200,
  vdd_volts: 3.28,
  target_powered: true,
  breakin_mode: 2,
  core_state: 'HALTED',
  bus_freq_hz: 1000000, // 1.0 MHz
  firmware_version: 'v1.4.2-dwire32a-panadiga',
  chip_id: 'RP2040-B2',
  total_probes: 256,
  hits_detected: 4,
  open_bus_high: 218,
  open_bus_low: 34,
  ram_loader_injected: false,
  last_keepalive: new Date().toLocaleTimeString(),
};

export const mockInitialSerialLogs: SerialLogEntry[] = [
  {
    id: '1',
    timestamp: '22:28:01.102',
    level: 'info',
    source: 'HOST',
    message: 'Iniciando handshake com Raspberry Pi Pico na porta COM4...',
  },
  {
    id: '2',
    timestamp: '22:28:01.145',
    level: 'success',
    source: 'PICO',
    message: 'RP2040 conectado! Firmware v1.4.2-dwire32a inicializado via PIO0.',
  },
  {
    id: '3',
    timestamp: '22:28:01.150',
    level: 'rx',
    source: 'PICO',
    message: 'ADC0 VDD Sense: 3.28V (Alimentação do DMR-EH55 confirmada acima de 2.50V).',
  },
  {
    id: '4',
    timestamp: '22:28:01.200',
    level: 'info',
    source: 'HOST',
    message: 'Aplicando Cold Break-In Modo 2 (EXTRG High / Pre+Post Burst 32 Clocks)...',
  },
  {
    id: '5',
    timestamp: '22:28:01.240',
    level: 'success',
    source: 'TARGET',
    message: 'AM34 Core capturado com sucesso! OCD Status: HALTED (PC: 0x80000000).',
  },
];

// ============================================================================
// 6. SERVICE MANUALS & SCHEMATICS (15 PDFs)
// ============================================================================

export const mockManualDocuments: ManualDocumentMetadata[] = [
  {
    id: '1',
    file_name: 'panasonic_dmr-eh55p9_eh55pc9.pdf',
    file_path: 'manuals_and_schematics/panasonic_dmr-eh55p9_eh55pc9.pdf',
    title: 'Panasonic DMR-EH55P9 / EH55PC9 Service Manual (Main Digital PCB)',
    description: 'Manual técnico completo com esquemático da placa principal VEP79138, barramento DWire32A CN55001 e Flash Spansion IC5001.',
    category: 'service_manual',
    size_bytes: 6177046,
    page_count: 114,
    key_ics: ['MN2DS0016', 'S29JL064H', 'CN55001', 'IC5001', 'IC55007'],
  },
  {
    id: '2',
    file_name: 'c2c7f1.pdf',
    file_path: 'manuals_and_schematics/c2c7f1.pdf',
    title: 'DMR-EH55 Block Diagrams & Interconnect Schematics',
    description: 'Diagramas em blocos detalhando barramentos IDE/PATA, RAM drive, alimentação e circuito de reset D_RESET.',
    category: 'schematic',
    size_bytes: 8655525,
    page_count: 82,
    key_ics: ['IC5001', 'IC5002', 'VEP79138', 'D_RESET'],
  },
  {
    id: '3',
    file_name: 'cb6cee.pdf',
    file_path: 'manuals_and_schematics/cb6cee.pdf',
    title: 'DMR-EH55 Schematic Circuit Board Overviews',
    description: 'Layout das trilhas de bancada, pontos de teste TP e conectores de depuração serial.',
    category: 'schematic',
    size_bytes: 5287663,
    page_count: 64,
    key_ics: ['CN55001', 'TP5501', 'TP5502', 'S29JL064H'],
  },
  {
    id: '4',
    file_name: 'dmreh55.pdf',
    file_path: 'manuals_and_schematics/dmreh55.pdf',
    title: 'DMR-EH55 Quick Service Guide & Self-Check Matrix',
    description: 'Tabela de códigos de erro de bancada: U99 (Self-Check loop), HDD Format, e rotina de reinicialização forçada.',
    category: 'bulletin',
    size_bytes: 2772445,
    page_count: 48,
    key_ics: ['U99', 'SELF-CHECK', 'HDD FORMAT', 'SERVICE MODE'],
  },
  {
    id: '5',
    file_name: 'panasonic_dmr-eh55ec.pdf',
    file_path: 'manuals_and_schematics/panasonic_dmr-eh55ec.pdf',
    title: 'Panasonic DMR-EH55EC European Service Manual',
    description: 'Especificações técnicas para os modelos europeus (PAL / SECAM) com descrição da memória EEPROM e NVRAM.',
    category: 'service_manual',
    size_bytes: 5638504,
    page_count: 98,
    key_ics: ['EEPROM', 'NVRAM', 'MN2DS0016'],
  },
  {
    id: '6',
    file_name: 'panasonic_dmr-eh55ee.pdf',
    file_path: 'manuals_and_schematics/panasonic_dmr-eh55ee.pdf',
    title: 'Panasonic DMR-EH55EE Eastern Europe Service Supplement',
    description: 'Suplemento de diagnóstico de bancada para reparo de fonte e placa de sinais.',
    category: 'service_manual',
    size_bytes: 5385311,
    page_count: 76,
    key_ics: ['FIRMWARE', 'UPDATE', 'FLASH'],
  },
  {
    id: '7',
    file_name: '6097ea.pdf',
    file_path: 'manuals_and_schematics/6097ea.pdf',
    title: 'DMR-EH55 Microprocessor Control & Signal Path',
    description: 'Mapeamento de registradores de controle do microprocessador e barramentos de clock.',
    category: 'schematic',
    size_bytes: 5136699,
    page_count: 58,
    key_ics: ['MN103', 'AM34', 'CLK_GEN'],
  },
  {
    id: '8',
    file_name: '9e8a23.pdf',
    file_path: 'manuals_and_schematics/9e8a23.pdf',
    title: 'DMR-EH55 Power Supply & Servo Drive Circuit',
    description: 'Circuito da fonte de alimentação chaveada e acionamento dos motores da unidade óptica.',
    category: 'schematic',
    size_bytes: 6015970,
    page_count: 70,
    key_ics: ['STR-G6653', 'SERVO_DSP'],
  },
  {
    id: '9',
    file_name: 'a5e678.pdf',
    file_path: 'manuals_and_schematics/a5e678.pdf',
    title: 'DMR-EH55 Audio/Video Processing Subsystem',
    description: 'Processamento de vídeo analógico/digital, decodificadores MPEG e saída HDMI.',
    category: 'schematic',
    size_bytes: 5062896,
    page_count: 62,
    key_ics: ['ADV7320', 'MN2DS0016'],
  },
  {
    id: '10',
    file_name: 'philips-dmr-eh55.pdf',
    file_path: 'manuals_and_schematics/philips-dmr-eh55.pdf',
    title: 'Panasonic DIGA Diagnostic Cross-Reference Guide',
    description: 'Guia de calibração de laser de bancada e procedimentos de troca de disco rígido IDE.',
    category: 'service_manual',
    size_bytes: 4337730,
    page_count: 54,
    key_ics: ['HDD_EXCHANGE', 'LASER_CAL', 'SERVICE_DISK'],
  },
];

export const mockManualSearchResults: ManualSearchResult[] = [
  {
    file_name: 'panasonic_dmr-eh55p9_eh55pc9.pdf',
    file_path: 'manuals_and_schematics/panasonic_dmr-eh55p9_eh55pc9.pdf',
    title: 'Panasonic DMR-EH55P9 / EH55PC9 Service Manual',
    category: 'service_manual',
    total_pages: 114,
    total_match_count: 24,
    matches: {
      U99: [12, 14, 45, 48],
      S29JL064: [32, 33, 35, 78],
      CN55001: [41, 42, 88],
      MN2DS: [28, 29, 30, 31, 85],
    },
    snippets: [
      {
        term: 'U99',
        page: 14,
        context: 'When U99 is displayed on the FL display, the system microcomputer detected communication timeout during Self-Check. Check EEPROM emulation parameters in flash memory IC5001.',
      },
      {
        term: 'CN55001',
        page: 42,
        context: 'CN55001 is the DWire32A On-Chip Debug (OCD) connector for factory programming and boundary scan. Pin 1: VDD (3.3V), Pin 2: EXTRG, Pin 3: SCLK, Pin 4: SDATA, Pin 5: nRESET, Pin 6: GND.',
      },
      {
        term: 'S29JL064',
        page: 32,
        context: 'IC5001 is a Spansion S29JL064H 64-Mbit (8MB) NOR Flash memory. Sectors SA134..SA141 are allocated for system configuration parameter storage.',
      },
    ],
  },
  {
    file_name: 'dmreh55.pdf',
    file_path: 'manuals_and_schematics/dmreh55.pdf',
    title: 'DMR-EH55 Quick Service Guide & Self-Check Matrix',
    category: 'bulletin',
    total_pages: 48,
    total_match_count: 18,
    matches: {
      U99: [3, 4, 7, 18],
      'SELF-CHECK': [3, 5, 8, 12, 19],
      'HDD FORMAT': [22, 23],
    },
    snippets: [
      {
        term: 'U99',
        page: 4,
        context: 'ERROR CODE U99: Occurs during boot sequence when checksum validation of NVRAM sector fails. The unit enters an infinite restart cycle.',
      },
      {
        term: 'HDD FORMAT',
        page: 22,
        context: 'Entering Service Mode 7 initiates MEIHDFS HDD low-level format. MBR header at LBA 0 ("SH") is initialized and mirrored at LBA 57344.',
      },
    ],
  },
  {
    file_name: 'c2c7f1.pdf',
    file_path: 'manuals_and_schematics/c2c7f1.pdf',
    title: 'DMR-EH55 Block Diagrams & Interconnect Schematics',
    category: 'schematic',
    total_pages: 82,
    total_match_count: 15,
    matches: {
      CN55001: [15, 16],
      IC5001: [12, 13, 24],
      VEP79138: [1, 2, 8],
    },
    snippets: [
      {
        term: 'CN55001',
        page: 16,
        context: 'SCHEMATIC DIAGRAM VEP79138: Detail of CN55001 showing series damping resistors (47R on SCLK, 100R on SDATA) and 4.7k pull-up to 3.3V rail.',
      },
    ],
  },
];

// ============================================================================
// 7. REALISTIC HEX DUMP SAMPLES FOR HEX INSPECTOR
// ============================================================================

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
