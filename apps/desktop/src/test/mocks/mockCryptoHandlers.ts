/**
 * Mock IPC handlers for PANA_DVD.FRM cryptanalysis, Spansion NVRAM, and DIGA model catalog.
 */

import {
  mockCodebookReport,
  mockCryptanalysisReport,
  mockFrmTocDumpReport,
  mockNvramCleanReport,
} from './mockData';

export function handleMockCryptoCommand(cmd: string): unknown | undefined {
  switch (cmd) {
    case 'clean_nvram':
    case 'analyze_flash_dump':
      return mockNvramCleanReport;

    case 'analyze_frm':
      return mockCryptanalysisReport;

    case 'dump_frm_toc':
      return mockFrmTocDumpReport;

    case 'get_codebook':
      return mockCodebookReport;

    case 'list_frm_containers_command':
      return ['firmware_and_images/PANA_DVD.FRM'];

    case 'pack_frm_command':
      return {
        mode: 'sd',
        total_bytes: 67584,
        toc_region_size: 2048,
        modules: [
          {
            index: 0,
            name: 'DIAG01',
            module_offset: 2048,
            module_size: 65536,
            fvu_checksum_u32: 0x12345678,
            list_sum_u16: 0x0abc,
          },
        ],
      };

    case 'unpack_extrhead_command':
      return {
        file_path: 'firmware_and_images/DIGADIAG.FRM',
        header: {
          tag: 'FVU ',
          flag: 0,
          algo: 1,
          dst_len: 1024,
          dst_addr: 0x50400000,
          src_len: 512,
          footer_offset: 576,
          header_checksum: 0x12345678,
          adler_step: 16,
          build_name: 'DIAG01',
        },
        decompressed_bytes: 1024,
        calculated_checksum_hex: '0x12345678',
        header_checksum_hex: '0x12345678',
        checksum_valid: true,
        footer_present: true,
        written_output_path: null,
      };

    case 'get_diga_model_catalog':
      return [
        {
          model_id: 'AUTO',
          series: 'Auto-Detect (Passive Address Aliasing / Binary Signature)',
          flash_size_mb: 8,
          flash_size_bytes: 8388608,
          nvram_base_hex: 'AUTO (0x801F0000 / 0x803F0000 / 0x807F0000 / 0x80FE0000)',
          sector_geometry: 'Auto-Detected Top-Boot / Uniform',
          soc_name: 'UniPhier AM34 / MN103S Auto',
        },
        {
          model_id: 'DMR-EH55',
          series: 'Série EH (HDD+DVD)',
          flash_size_mb: 8,
          flash_size_bytes: 8388608,
          nvram_base_hex: '0x807F0000',
          sector_geometry: 'Top-Boot 8x8KB (SA134..SA141)',
          soc_name: 'Matsushita AM34 / MN103S (VEP79138)',
        },
        {
          model_id: 'DMR-ES15',
          series: 'Série ES (DVD/Combo)',
          flash_size_mb: 8,
          flash_size_bytes: 8388608,
          nvram_base_hex: '0x807F0000',
          sector_geometry: 'Top-Boot 8x8KB (SA134..SA141)',
          soc_name: 'Matsushita AM34 / MN103S (VEP79138)',
        },
        {
          model_id: 'DMR-ES10',
          series: 'Série ES (DVD-RAM)',
          flash_size_mb: 4,
          flash_size_bytes: 4194304,
          nvram_base_hex: '0x803F0000',
          sector_geometry: 'Top-Boot 8x8KB (SA63..SA70)',
          soc_name: 'Matsushita AM33 / MN103S',
        },
        {
          model_id: 'DMR-EH50',
          series: 'Série EH (HDD+DVD)',
          flash_size_mb: 4,
          flash_size_bytes: 4194304,
          nvram_base_hex: '0x803F0000',
          sector_geometry: 'Top-Boot 8x8KB (SA63..SA70)',
          soc_name: 'Matsushita AM33 / MN103S',
        },
        {
          model_id: 'DMR-E85H',
          series: 'Série ExxH (HDD+DVD)',
          flash_size_mb: 2,
          flash_size_bytes: 2097152,
          nvram_base_hex: '0x801F0000',
          sector_geometry: 'Top-Boot 8x8KB (SA31..SA38)',
          soc_name: 'Matsushita AM33 / MN103S',
        },
        {
          model_id: 'DMR-EX75',
          series: 'Série EX (HDD+DVB-T)',
          flash_size_mb: 16,
          flash_size_bytes: 16777216,
          nvram_base_hex: '0x80FE0000',
          sector_geometry: 'Uniform 128KB (SA126..SA127)',
          soc_name: 'Matsushita AM34 / MN103S (VEP79138)',
        },
        {
          model_id: 'DMR-EZ27',
          series: 'Série EZ (DVD+DVB-T)',
          flash_size_mb: 16,
          flash_size_bytes: 16777216,
          nvram_base_hex: '0x80FE0000',
          sector_geometry: 'Uniform 128KB (SA126..SA127)',
          soc_name: 'Matsushita AM34 / MN103S (VEP79138)',
        },
      ];

    default:
      return undefined;
  }
}
