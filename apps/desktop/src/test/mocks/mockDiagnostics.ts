import { PICO_MIN_VDD_THRESHOLD_VOLTS } from '@/constants';
import { PicoDiagnosticResult } from '@/models';
import { parseDiagnosticOutput } from '@/services/diagnosticParser';

/**
 * Test-only fixture simulator for Pico diagnostic outputs (used exclusively in Vitest & Playwright E2E).
 */
export function simulateMockDiagnostic(commandId: string, arg?: string): PicoDiagnosticResult {
  const norm = commandId.trim().toUpperCase();
  const cleanArg = arg?.trim();
  const rawCommand = cleanArg ? `${norm} ${cleanArg}` : norm;
  const mockVdd = (PICO_MIN_VDD_THRESHOLD_VOLTS + 0.78).toFixed(2);

  let rawOutput = `${norm}: OK VDD=${mockVdd}V`;
  if (norm === 'PINCHECK') {
    rawOutput = [
      '=== P7501 6-PIN ELECTRICAL MATRIX (PINCHECK) ===',
      'Pin               | Hi-Z | Pull-Down | Pull-Up | Electrical State',
      '------------------+------+-----------+---------+----------------------------------------',
      'GP2 (Pin6 RXD   ) |  1   |     0     |    1    | HI-Z / UART RX 115200 8N1 (R7572 Pull-Up)',
      'GP3 (Pin5 NSW5V ) |  1   |     1     |    1    | HI-Z SENSE ONLY (Static rail ~3.29V; IC1506 ~5.15V)',
      'GP4 (Pin3 TXD   ) |  1   |     1     |    1    | IDLE HIGH (R7570 10k / Q7514 TX)',
      'GP5 (Pin2 XRESET) |  1   |     1     |    1    | STRONG HIGH (Open-Drain Control)',
      `GP26(Pin1 SWUART) | ${mockVdd}V|   2.69V   |  3.29V  | SW_UART / ADC0 Rail Sense`,
      '------------------------------------------------------------------------',
      'While GP5 (XRESET) asserted LOW (0V): GP2=1, GP3=1, GP4=1, GP5=0',
      'XRESET Release Rise Time (Hi-Z)     : 40 us',
      'Post-Reset 250ms Transitions        : GP2=872, GP3=0, GP4=0, GP5=2',
      'PINCHECK: OK',
    ].join('\n');
  } else if (norm === 'SNIFF') {
    rawOutput = [
      `SNIFF RESULT (${cleanArg || '250'} ms):`,
      '  RXD/EXT(GP2) : Final=1, Transitions=0',
      '  NSW5V  (GP3) : Final=1, Transitions=24',
      '  TXD/DAT(GP4) : Final=1, Transitions=68',
      '  XRESET (GP5) : Final=1, Transitions=0',
      '  [STATUS] ACTIVE BUS ACTIVITY DETECTED! (GP2=0, GP3=24, GP4=68, GP5=0)',
      'SNIFF: OK',
    ].join('\n');
  } else if (norm === 'BUSDIAG') {
    rawOutput = [
      '=== P7501 ELECTRICAL CONTENTION & DUAL-CHANNEL DIAGNOSTIC (BUSDIAG) ===',
      'State 0 (SW_UART=HiZ, XRESET=HiZ): GP4 @ 2mA -> pad_low=0, rise_ticks=12',
      'Boot Transitions (NORMAL SW_UART=HiZ): GP2=284 GP3=12 GP4=420 GP5=2',
      'BUSDIAG: OK',
    ].join('\n');
  } else if (norm === 'CFI') {
    rawOutput = 'CFI: OK (Spansion S29JL064H QRY Validated, Size: 8 MB / 64 Mbit)';
  } else if (norm === 'FLASH_SIZE') {
    rawOutput = `FLASH_SIZE: OK (Target NOR Flash configurada para ${cleanArg || '8'} MB)`;
  } else if (norm === 'DUMP') {
    rawOutput = [
      `[DUMP] Reading words from ${cleanArg || '0x807F0000 16'}...`,
      '807F0000: FFFFFFFF FFFFFFFF 55AA0100 00000001',
      '807F0010: 44494741 35354845 00000000 FFFFFFFF',
      '807F0020: FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF',
      '807F0030: FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF',
      'DUMP: OK',
    ].join('\n');
  } else if (norm === 'BOOTCAP' || norm === 'UARTMON' || norm === 'JIGBOOT') {
    rawOutput = [
      `=== ${norm}: PANASONIC 115200 8N1 UART CAPTURE ON GP2 (42 bytes, ${cleanArg || '7500'} ms) ===`,
      'AM34 IPL BOOT v1.18 OK | DDR CHECK OK',
      `${norm}: OK`,
    ].join('\n');
  } else if (norm.startsWith('FMU') || norm.startsWith('UART')) {
    rawOutput = `UART/FMU Probe Capture (18 bytes): SERI_READY OK\n${norm}: OK`;
  } else if (norm === 'SCAN') {
    rawOutput = '[SCAN] Probing 4 Modes x 256 Opcodes...\nMode 2 Opcode 0x01: HIT! DCSR=0x00000001 (HALTED)\nSCAN: COMPLETED';
  } else if (norm === 'AUTODETECT') {
    rawOutput = 'AUTODETECT: OK (8 MB NOR Flash, EEMUL NVRAM @ 0x807F0000)';
  } else if (norm === 'STATUS' || norm === 'VDD') {
    rawOutput = `STATUS: OK VDD=${mockVdd}V | OCD=HALTED | PC=0x80000000 | MODE=2`;
  } else if (norm === 'HANDSHAKE' || norm === 'BREAK_IN') {
    rawOutput = `HANDSHAKE: OK Mode=${cleanArg || '2'} DCSR=0x00000001 (AM34 HALTED) VDD=${mockVdd}V`;
  } else if (norm === 'BOOTSEL') {
    rawOutput = 'BOOTSEL: OK (Pico reiniciando em modo USB Mass Storage RPI-RP2)';
  }

  return parseDiagnosticOutput(norm, rawCommand, rawOutput, 45);
}
