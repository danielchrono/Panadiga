/**
 * Mock IPC handlers for HDD MEIHDFS inspection, extraction, and physical drive operations.
 */

import { PartitionExtractionManifest } from '@/models';
import {
  mockHddAnalysisReport,
  mockMeihdfsAllocationReport,
} from '../mockData';

export function handleMockHddCommand(
  cmd: string,
  args?: Record<string, unknown>
): unknown | undefined {
  switch (cmd) {
    case 'inspect_hdd':
      return mockHddAnalysisReport;

    case 'decode_meihdfs_allocations':
      return mockMeihdfsAllocationReport;

    case 'extract_partitions': {
      const manifest: PartitionExtractionManifest = {
        timestamp: new Date().toISOString(),
        sourceImage: 'firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin',
        outputDirectory: 'firmware_and_images/extracted_partitions/',
        manifestSha256Sums:
          '499e1d8820cbf018a77d1219eeea103980bc235e182390aef92348a011234bc9 *partition_1_am34_os.bin\n' +
          '7721839048102938471920394857102938471920394857102938471920394857 *partition_2_fonts_microcode.bin',
        extractedPartitions: [
          {
            id: 1,
            name: 'Partition 1 (AM34 Main Firmware)',
            filename: 'partition_1_am34_os.bin',
            offsetHex: '0x02000000',
            sizeBytes: 6173264,
            sha256: '499e1d8820cbf018a77d1219eeea103980bc235e182390aef92348a011234bc9',
            md5: '72dae89b21f3089daef3810237aa01e2',
            entropy: 7.9942,
          },
          {
            id: 2,
            name: 'Partition 2 (OSD Fonts & Microcode)',
            filename: 'partition_2_fonts_microcode.bin',
            offsetHex: '0x04000000',
            sizeBytes: 720896,
            sha256: '7721839048102938471920394857102938471920394857102938471920394857',
            md5: '3c8919f2010938472910384719283746',
            entropy: 7.8214,
          },
        ],
      };
      return manifest;
    }

    case 'list_physical_drives':
      return [
        {
          device_path: '\\\\.\\PhysicalDrive0',
          display_name: 'Physical Drive 0 (Host Windows NVMe, 512.0 GB)',
          size_bytes: 512110190592,
          size_gb: 512.0,
          is_system_drive: true,
          safe_to_write: false,
          lockout_reason: 'LOCKED: Windows Host Operating System Boot Drive (PhysicalDrive0)',
        },
        {
          device_path: '\\\\.\\PhysicalDrive1',
          display_name: 'Physical Drive 1 (PATA/IDE Panasonic DIGA Target, 80.0 GB)',
          size_bytes: 80026361856,
          size_gb: 80.0,
          is_system_drive: false,
          safe_to_write: true,
          lockout_reason: null,
        },
      ];

    case 'format_hdd_meihdfs':
    case 'write_hdd_image':
    case 'restore_hdd_mirrors':
      return {
        device_path: (args?.devicePath as string) || '\\\\.\\PhysicalDrive1',
        operation: cmd,
        bytes_written: 104857600,
        sectors_written: 204800,
        success: true,
        message: 'Operação física de gravação MEIHDFS executada com sucesso.',
      };

    default:
      return undefined;
  }
}
