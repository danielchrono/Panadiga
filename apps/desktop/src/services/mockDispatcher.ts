/**
 * High-Fidelity Mock Command Dispatcher & Reactive Stream Simulator
 * Single Responsibility: Emulates native Rust/Tauri responses and push streams
 * when running in Browser/Workbench Demo Mode or headless Vitest/Playwright suites.
 */

import {
  ColdBreakinMode,
  InCircuitNvramEraseResult,
  PartitionExtractionManifest,
} from '@/models';
import {
  generateMockHexDump,
  mockCodebookReport,
  mockCryptanalysisReport,
  mockFrmTocDumpReport,
  mockHddAnalysisReport,
  mockManualSearchResults,
  mockMeihdfsAllocationReport,
  mockNvramCleanReport,
  mockPicoDeviceStatus,
  mockSystemOverview,
} from './mockData';
import { logger } from '@/utils/logger';

export async function handleMockCommand<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  await new Promise(resolve => setTimeout(resolve, 80));

  switch (cmd) {
    case 'get_system_overview':
      return mockSystemOverview as unknown as T;

    case 'clean_nvram':
    case 'analyze_flash_dump':
      return mockNvramCleanReport as unknown as T;

    case 'inspect_hdd':
      return mockHddAnalysisReport as unknown as T;

    case 'decode_meihdfs_allocations':
      return mockMeihdfsAllocationReport as unknown as T;

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
      return manifest as unknown as T;
    }

    case 'analyze_frm':
      return mockCryptanalysisReport as unknown as T;

    case 'dump_frm_toc':
      return mockFrmTocDumpReport as unknown as T;

    case 'get_codebook':
      return mockCodebookReport as unknown as T;

    case 'pico_get_status':
      return mockPicoDeviceStatus as unknown as T;

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
      ] as unknown as T;

    case 'pico_set_breakin_mode': {
      const mode = (args?.mode as ColdBreakinMode) ?? 2;
      return {
        ...mockPicoDeviceStatus,
        breakin_mode: mode,
      } as unknown as T;
    }

    case 'pico_inject_ram_loader': {
      const eraseResult: InCircuitNvramEraseResult = {
        success: true,
        elapsed_ms: 1420,
        voltage_before: 3.28,
        voltage_after: 3.27,
        ram_loader_addr: '0x00004000',
        bytes_erased: 65536,
        verification_sha256: '0a394c8e7710bcf1489e3a6a92ecda46e01768494c8fa72722ee20c5c363dc78',
        log_lines: [
          'Multi-Mode HANDSHAKE (Mode 2 -> 0 -> 3 -> 1): AM34 core halted (PC=0x80000000, DCSR=0x00000001).',
          'Pre-Erase Live Backup: Saved nvram_live_backup.bin (0x807F0000) & sram_secrets.bin (0x00000000).',
          'INJECT_LOADER CONFIRM: Streaming 80-byte JEDEC RAM loader into SRAM at 0x00004000... OK',
          'Setting PC=0x00004000 and executing ERASE_NVRAM CONFIRM...',
          'Spansion Flash Command 0xAA->0x55->0x80 Sector Erase SA134..SA141 issued.',
          'Polling Spansion RY/BY# (DQ7 Data Polling)... Complete in 840ms.',
          'Verifying 65,536 bytes read back as 0xFF: 100% FACTORY VIRGIN STATE.',
          'Releasing nRESET. DMR-EH55 now boots clean with U99 eliminated!',
        ],
      };
      return eraseResult as unknown as T;
    }

    case 'pico_send_serial_command': {
      const cmdStr = ((args?.command as string) || '').trim().toUpperCase();
      if (cmdStr === 'STATUS' || cmdStr === 'VDD') {
        return 'VDD=3.28V | OCD=HALTED | PC=0x80000000 | MODE=2' as unknown as T;
      }
      if (cmdStr === 'RESET') {
        return 'D_RESET pulsed low for 10ms. Target reset asserted.' as unknown as T;
      }
      if (cmdStr === 'ADC') {
        return 'ADC0 Raw: 4072 (3.281 V). Rail within tolerance (+-5%).' as unknown as T;
      }
      if (cmdStr === 'GDB') {
        return 'GDB Remote Serial Protocol mode active.' as unknown as T;
      }
      if (cmdStr === 'HANDSHAKE') {
        return 'DCSR: 0x00000001 | [HANDSHAKE] SUCCESS! AM34 core HALTED (Mode 2).' as unknown as T;
      }
      return `ACK: ${cmdStr} (Instrução executada no PIO0 @ 0x00004000)` as unknown as T;
    }

    case 'search_manuals': {
      const query = (args?.query as string)?.toUpperCase();
      if (!query) {
        return mockManualSearchResults as unknown as T;
      }
      const filtered = mockManualSearchResults.filter(item => {
        return (
          item.title.toUpperCase().includes(query) ||
          item.file_name.toUpperCase().includes(query) ||
          Object.keys(item.matches).some(k => k.toUpperCase().includes(query)) ||
          item.snippets.some(s => s.context.toUpperCase().includes(query))
        );
      });
      return filtered as unknown as T;
    }

    case 'read_hex_dump': {
      const offset = (args?.offset as number) ?? 0;
      const count = (args?.count as number) ?? 32;
      return generateMockHexDump(offset, count) as unknown as T;
    }

    case 'list_pico_serial_ports': {
      return [
        'MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]',
        'MOCK_RP2350 [RP2350 Pico 2 DWire32A Bench Simulator - 520KB SRAM]',
        'COM4 [Raspberry Pi Pico (RP2040) DWire32A - VID:2E8A PID:000A]',
      ] as unknown as T;
    }

    case 'start_gdb_bridge_command': {
      return {
        running: true,
        port_name: (args?.port_name as string) || 'MOCK_RP2040',
        listen_addr: (args?.listen_addr as string) || '127.0.0.1:3333',
        connected_clients: 0,
        last_event: 'Ouvindo conexões GDB em 127.0.0.1:3333',
      } as unknown as T;
    }

    case 'stop_gdb_bridge_command': {
      return {
        running: false,
        port_name: '',
        listen_addr: '127.0.0.1:3333',
        connected_clients: 0,
        last_event: 'Servidor GDB finalizado',
      } as unknown as T;
    }

    case 'get_gdb_bridge_status_command': {
      return {
        running: false,
        port_name: 'MOCK_RP2040',
        listen_addr: '127.0.0.1:3333',
        connected_clients: 0,
        last_event: 'Servidor GDB offline',
      } as unknown as T;
    }

    case 'get_am34_registers_command': {
      return {
        d: [0x00000000, 0x00000000, 0x00000000, 0x00000000],
        a: [0x00000000, 0x00000000, 0x00000000, 0x0003FFF0],
        mdr: 0x00000000,
        psw: 0x00000100,
        pc: 0x80000000,
        sp: 0x0003FFF0,
        is_halted: true,
      } as unknown as T;
    }

    case 'get_gdb_telemetry_command': {
      const baseTime = Date.now();
      return [
        {
          timestamp_ms: baseTime - 500,
          direction: 'INBOUND',
          command_type: 'qSupported',
          raw_packet: '$qSupported:multiprocess+;swbreak+#df',
          checksum_valid: true,
        },
        {
          timestamp_ms: baseTime - 400,
          direction: 'OUTBOUND',
          command_type: 'RspPayload',
          raw_packet: '$PacketSize=1000;qXfer:features:read+#d7',
          checksum_valid: true,
        },
        {
          timestamp_ms: baseTime - 300,
          direction: 'INBOUND',
          command_type: 'HaltReason',
          raw_packet: '$?#3f',
          checksum_valid: true,
        },
        {
          timestamp_ms: baseTime - 200,
          direction: 'OUTBOUND',
          command_type: 'RspPayload',
          raw_packet: '$S05#b8',
          checksum_valid: true,
        },
        {
          timestamp_ms: baseTime - 100,
          direction: 'INBOUND',
          command_type: 'ReadRegs',
          raw_packet: '$g#67',
          checksum_valid: true,
        },
      ] as unknown as T;
    }

    case 'launch_external_tool_command': {
      const tool = (args?.tool as string) || 'desconhecido';
      return `[Modo Demonstração] Ferramenta externa '${tool}' executada com sucesso.` as unknown as T;
    }

    case 'query_rag_engine': {
      const q = (args?.query as string) || '';
      return {
        query: q,
        answer:
          'Error U99 indicates NVRAM corruption on the Spansion S29JL064H NOR Flash (micro-sectors SA134..SA141). Erase sectors SA134..SA141 (0x007F0000..0x007FFFFF) to restore virgin 0xFF state while preserving bootloader IPL at SA0..SA133.',
        confidence: 0.98,
        groundedness_score: 1.0,
        is_abstained: false,
        abstention_reason: null,
        citations: [
          {
            doc: 'manuals_and_schematics/dmr-eh55/service/panasonic_dmr_eh55p9_eh55pc9_service_manual_north_america.pdf',
            section: 'Self-Diagnostic and Error Codes',
            lines: 'P.21-22',
            excerpt: 'U99: System microprocessor communication fault or NVRAM checksum mismatch.',
          },
        ],
        safety: {
          is_safe: true,
          violations: [],
          alerts: [],
        },
        critic: {
          is_faithful: true,
          ungrounded_entities: [],
        },
        retrieved_chunks_count: 3,
        execution_time_ms: 12,
      } as unknown as T;
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
      ] as unknown as T;

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
      } as unknown as T;

    case 'pico_detect_bootsel_drives':
      return ['D:\\ (RPI-RP2)'] as unknown as T;

    case 'pico_trigger_bootsel_reboot':
      return 'Dispositivo reiniciado em modo BOOTSEL' as unknown as T;

    case 'pico_flash_firmware':
      return {
        success: true,
        target_board: 'Raspberry Pi Pico (RP2040)',
        mount_path: 'D:\\',
        bytes_written: 119296,
        message: 'Firmware panasonic_dwire32a.uf2 gravado com sucesso no drive BOOTSEL.',
      } as unknown as T;

    default:
      logger.warn('MockIPC', `Unhandled command: ${cmd}`);
      return {} as unknown as T;
  }
}

export function subscribeMockEventStream<T>(
  eventName: string,
  handler: (payload: T) => void
): () => void {
  if (eventName === 'pico_telemetry') {
    const interval = setInterval(() => {
      handler({ ...mockPicoDeviceStatus } as unknown as T);
    }, 1500);
    return () => clearInterval(interval);
  }
  return () => { };
}
