/**
 * Mock IPC handlers for Raspberry Pi Pico P7501 Async UART hardware link, BOOTSEL flashing, and GDB RSP bridge.
 */

import { ColdBreakinMode, FlashReport, InCircuitNvramEraseResult } from '@/models';
import { emitLocalEvent } from '@/services/ipc';
import { simulateMockDiagnostic } from './mockDiagnostics';
import { mockNvramCleanReport, mockPicoDeviceStatus } from './mockData';

function createMockPicoStateStore() {
  let connected = false;
  let port = '';
  let bootselDetected = false;

  return Object.freeze({
    isConnected: () => connected,
    getPort: () => port,
    isBootselDetected: () => bootselDetected,
    setConnection: (nextConnected: boolean, nextPort?: string) => {
      connected = nextConnected;
      if (nextPort !== undefined) {
        port = nextPort;
      }
    },
    setBootselDetected: (detected: boolean) => {
      bootselDetected = detected;
    },
  });
}

const mockPicoState = createMockPicoStateStore();

export function buildMockPicoStatus(fallbackPort = ''): typeof mockPicoDeviceStatus {
  const connected = mockPicoState.isConnected();
  const port = mockPicoState.getPort() || fallbackPort;
  return {
    ...mockPicoDeviceStatus,
    connected,
    port,
    vdd_volts: connected ? 3.28 : 0.0,
    target_powered: connected,
    core_state: connected ? 'HALTED' : 'DISCONNECTED',
  };
}

export function handleMockPicoCommand(
  cmd: string,
  args?: Record<string, unknown>
): unknown | undefined {
  switch (cmd) {
    case 'pico_connect_and_test': {
      const portName = (args?.portName as string) || (args?.port_name as string) || 'MOCK_RP2040';
      const isRp2350 = portName.includes('2350');
      mockPicoState.setConnection(true, portName);
      return {
        connected: true,
        port_name: portName,
        baud_rate: 115200,
        vdd_voltage: 3.28,
        target_powered: true,
        firmware_banner: `PANADIGA P7501 ASYNC UART ENGINE v2.0 (${isRp2350 ? 'RP2350' : 'RP2040'}, 520 KB SRAM)`,
        message: `Conectado ao dispositivo em ${portName}. Canal P7501 Async UART (115200 8N1) pronto.`,
      };
    }

    case 'pico_get_status': {
      const argPort = (args?.portName as string) || (args?.port_name as string) || '';
      return buildMockPicoStatus(argPort);
    }

    case 'pico_set_breakin_mode': {
      const mode = (args?.mode as ColdBreakinMode) ?? 2;
      return {
        ...mockPicoDeviceStatus,
        breakin_mode: mode,
      };
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
          'Releasing XRESET. DMR-EH55 now boots clean with U99 eliminated!',
        ],
      };
      return eraseResult;
    }

    case 'pico_send_serial_command': {
      const cmdStr = ((args?.command as string) || '').trim().toUpperCase();
      if (cmdStr === 'STATUS' || cmdStr === 'VDD') {
        return 'VDD=3.28V | UART=115200-8N1 | MODE=P7501 | LEGACY_OCD=STANDBY';
      }
      if (cmdStr === 'RESET') {
        return 'D_RESET pulsed low for 10ms. Target reset asserted.';
      }
      if (cmdStr === 'ADC') {
        return 'ADC0 Raw: 4072 (3.281 V). Rail within tolerance (+-5%).';
      }
      if (cmdStr === 'GDB') {
        return 'GDB Remote Serial Protocol mode active.';
      }
      if (cmdStr === 'HANDSHAKE') {
        return 'DCSR: 0x00000001 | [HANDSHAKE] SUCCESS! AM34 core HALTED (Mode 2).';
      }
      return `ACK: ${cmdStr} (Instrução executada no PIO0 @ 0x00004000)`;
    }

    case 'list_pico_serial_ports':
      // In real runtime, the backend provides actual COM ports.
      // Returning an empty list forces the UI to rely on the Tauri command.
      return [];

    case 'start_gdb_bridge_command':
      return {
        running: true,
        port_name: (args?.portName as string) || (args?.port_name as string) || 'MOCK_RP2040',
        listen_addr: (args?.listenAddr as string) || (args?.listen_addr as string) || '127.0.0.1:3333',
        connected_clients: 0,
        last_event: 'Ouvindo conexões GDB em 127.0.0.1:3333',
      };

    case 'stop_gdb_bridge_command':
      return {
        running: false,
        port_name: '',
        listen_addr: '127.0.0.1:3333',
        connected_clients: 0,
        last_event: 'Servidor GDB finalizado',
      };

    case 'get_gdb_bridge_status_command':
      return {
        running: false,
        port_name: 'MOCK_RP2040',
        listen_addr: '127.0.0.1:3333',
        connected_clients: 0,
        last_event: 'Servidor GDB offline',
      };

    case 'get_am34_registers_command':
      return {
        d: [0x00000000, 0x00000000, 0x00000000, 0x00000000],
        a: [0x00000000, 0x00000000, 0x00000000, 0x0003FFF0],
        mdr: 0x00000000,
        psw: 0x00000100,
        pc: 0x80000000,
        sp: 0x0003FFF0,
        is_halted: true,
      };

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
      ];
    }

    case 'pico_first_contact_backup':
      return {
        success: true,
        detected_model: 'DMR-EH55 (8MB NOR Flash | EEMUL 0x807F0000)',
        nvram_bytes: 65536,
        nvram_sha256: '0a394c8e7710bcf1489e3a6a92ecda46e01768494c8fa72722ee20c5c363dc78',
        double_read_verified: true,
        sram_bytes: 32768,
        ipl_bytes: 131072,
        log_lines: [
          'Handshake Mode 2 OK: Matsushita AM34 core halted @ 0x80000000',
          'Autodetected Model: DMR-EH55 (8MB Top-Boot Spansion S29JL064H)',
          'Double-Read NVRAM @ 0x807F0000 (64KB): Hash matches (verified zero cable error)',
          'Dumped SRAM secret keys @ 0x00000000 (32KB)',
          'First-contact safety backup complete. Target ready for surgical operations.',
        ],
      };

    case 'pico_detect_bootsel_drives':
      return mockPicoState.isBootselDetected() ? ['D:\\ (RPI-RP2)'] : [];

    case 'pico_trigger_bootsel_reboot':
      mockPicoState.setBootselDetected(true);
      mockPicoState.setConnection(false);
      return 'Dispositivo reiniciado em modo BOOTSEL';

    case 'pico_flash_firmware': {
      mockPicoState.setBootselDetected(false);
      const report: FlashReport = {
        success: true,
        target_drive: 'D:\\',
        board_family: 'Raspberry Pi Pico (RP2040)',
        uf2_size_bytes: 119296,
        blocks_written: 233,
        sha256_hash: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
        message:
          'Firmware panasonic_p7501_uart_unified.uf2 (P7501 Async UART 115200 8N1) gravado com sucesso no drive BOOTSEL.',
      };
      return report;
    }

    case 'pico_execute_diagnostic': {
      const commandId = (args?.commandId as string) || (args?.command_id as string) || 'STATUS';
      const arg = (args?.arg as string | undefined) ?? undefined;
      const result = simulateMockDiagnostic(commandId, arg);
      emitLocalEvent('pico://diagnostic-result', result);
      emitLocalEvent('pico://serial-log', {
        id: `rx-${Date.now()}`,
        timestamp: new Date().toLocaleTimeString('pt-BR', { hour12: false }),
        source: 'PICO',
        level: result.success ? 'rx' : 'error',
        message: result.summary,
      });
      return result;
    }

    case 'pico_capture_nvram_live': {
      const baseAddr = (args?.baseAddr as number) ?? 0x807f0000;
      const wordCount = (args?.wordCount as number) ?? 16384;
      const evt = {
        source: 'USB_CDC',
        port: (args?.portName as string) || mockPicoState.getPort() || 'MOCK_RP2040',
        base_bus_address: `0x${baseAddr.toString(16).toUpperCase().padStart(8, '0')}`,
        captured_bytes: wordCount * 4,
        captured_at: new Date().toLocaleTimeString('pt-BR', { hour12: false }),
        report: mockNvramCleanReport,
      };
      emitLocalEvent('pico://nvram-live-capture', evt);
      return evt;
    }

    case 'analyze_live_nvram_bytes': {
      const bytes = Array.isArray(args?.bytes) ? (args?.bytes as number[]) : [];
      return {
        ...mockNvramCleanReport,
        input_file: (args?.sourceLabel as string) || 'Live Pico P7501 / OCD Capture',
        nvram_non_ff_bytes: bytes.filter(b => b !== 0xff).length || mockNvramCleanReport.nvram_non_ff_bytes,
      };
    }

    case 'set_pico_telemetry_stream':
      return Boolean(args?.active ?? true);

    default:
      return undefined;
  }
}
