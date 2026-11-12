/**
 * Mock IPC handlers for PANA_DVD.FRM cryptanalysis, Spansion NVRAM, and DIGA model catalog.
 */

import {
  mockCodebookReport,
  mockCryptanalysisReport,
  mockFrmTocDumpReport,
  mockNvramCleanReport,
} from '../mockData';

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
          soc_name: 'MN2DS0016',
        },
        {
          model_id: 'DMR-ES15',
          series: 'Série ES (DVD/Combo)',
          flash_size_mb: 8,
          flash_size_bytes: 8388608,
          nvram_base_hex: '0x807F0000',
          sector_geometry: 'Top-Boot 8x8KB (SA134..SA141)',
          soc_name: 'MN2DS0016',
        },
        {
          model_id: 'DMR-ES10',
          series: 'Série ES (DVD-RAM)',
          flash_size_mb: 4,
          flash_size_bytes: 4194304,
          nvram_base_hex: '0x803F0000',
          sector_geometry: 'Top-Boot 8x8KB (SA63..SA70)',
          soc_name: 'MN2DS0009',
        },
        {
          model_id: 'DMR-EH50',
          series: 'Série EH (HDD+DVD)',
          flash_size_mb: 4,
          flash_size_bytes: 4194304,
          nvram_base_hex: '0x803F0000',
          sector_geometry: 'Top-Boot 8x8KB (SA63..SA70)',
          soc_name: 'MN2DS0009',
        },
        {
          model_id: 'DMR-E85H',
          series: 'Série ExxH (HDD+DVD)',
          flash_size_mb: 2,
          flash_size_bytes: 2097152,
          nvram_base_hex: '0x801F0000',
          sector_geometry: 'Top-Boot 8x8KB (SA31..SA38)',
          soc_name: 'MN2DS0004',
        },
        {
          model_id: 'DMR-EX75',
          series: 'Série EX (HDD+DVB-T)',
          flash_size_mb: 16,
          flash_size_bytes: 16777216,
          nvram_base_hex: '0x80FE0000',
          sector_geometry: 'Uniform 128KB (SA126..SA127)',
          soc_name: 'MN2DS0016',
        },
        {
          model_id: 'DMR-EZ27',
          series: 'Série EZ (DVD+DVB-T)',
          flash_size_mb: 16,
          flash_size_bytes: 16777216,
          nvram_base_hex: '0x80FE0000',
          sector_geometry: 'Uniform 128KB (SA126..SA127)',
          soc_name: 'MN2DS0018',
        },
      ];

    default:
      return undefined;
  }
}
