/**
 * IPC Communication Service for Panasonic DIGA Suite
 * Wraps @tauri-apps/api/core with automated fallback to Mock/Demo Mode
 * when running in browser or when hardware is not connected.
 */

import {
  NvramCleanReport,
  HddAnalysisReport,
  MeihdfsAllocationReport,
  CryptanalysisReport,
  FrmTocDumpReport,
  CodebookReport,
  PicoDeviceStatus,
  ManualSearchResult,
  SystemOverview,
  HexDumpRow,
  PartitionExtractionManifest,
  InCircuitNvramEraseResult,
  ColdBreakinMode,
} from '@/models';

import {
  mockSystemOverview,
  mockNvramCleanReport,
  mockHddAnalysisReport,
  mockMeihdfsAllocationReport,
  mockCryptanalysisReport,
  mockFrmTocDumpReport,
  mockCodebookReport,
  mockPicoDeviceStatus,
  mockManualSearchResults,
  generateMockHexDump,
} from './mockData';

// Determine if running inside a Tauri container
export const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

// Global Demo Mode state
let demoModeActive = !isTauriEnvironment();

export const isDemoModeActive = (): boolean => demoModeActive;

export const setDemoMode = (active: boolean): void => {
  demoModeActive = active;
};

/**
 * Universal invoke wrapper with full TypeScript safety
 */
export async function invokeCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  // If in demo mode or running outside Tauri, use realistic mocks
  if (demoModeActive || !isTauriEnvironment()) {
    return handleMockCommand<T>(cmd, args);
  }

  try {
    // Dynamic import to prevent bundler errors if @tauri-apps/api/core is only present at runtime
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<T>(cmd, args);
  } catch (err) {
    console.warn(`[IPC] Tauri command "${cmd}" failed or unavailable. Falling back to Mock data.`, err);
    return handleMockCommand<T>(cmd, args);
  }
}

/**
 * High-fidelity Mock Dispatcher for Workbench Testing
 */
async function handleMockCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  // Simulate realistic NVMe/bus latency
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
        ram_loader_addr: '0x90000000',
        bytes_erased: 65536,
        verification_sha256: '0a394c8e7710bcf1489e3a6a92ecda46e01768494c8fa72722ee20c5c363dc78',
        log_lines: [
          'D_RESET assert (GP5 Low, 10ms)... OK',
          'DWire32A Bus 32-clock wakeup sync burst applied... OK',
          'AM34 core halted in OCD mode (PC=0x80000000).',
          'Streaming ram_loader.bin (1,024 bytes) into I-RAM at 0x90000000... OK',
          'Setting PC=0x90000000 and resuming execution...',
          'Spansion Flash Command 0xAA->0x55->0x80 Sector Erase SA134..SA141 issued.',
          'Polling Spansion RY/BY# (DQ7 Data Polling)... Complete in 840ms.',
          'Verifying 65,536 bytes read back as 0xFF: 100% FACTORY VIRGIN STATE.',
          'Releasing nRESET. DMR-EH55 now boots clean with U99 eliminated!',
        ],
      };
      return eraseResult as unknown as T;
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

    case 'launch_external_tool_command': {
      const tool = (args?.tool as string) || 'desconhecido';
      return `[Modo Demonstração] Ferramenta externa '${tool}' executada com sucesso.` as unknown as T;
    }

    default:
      console.warn(`[MockIPC] Unhandled command: ${cmd}`);
      return {} as unknown as T;
  }
}
