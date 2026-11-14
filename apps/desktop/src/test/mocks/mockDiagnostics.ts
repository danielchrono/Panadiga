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
  } else if (norm === 'BOOTCAP' || norm === 'UARTMON' || norm === 'JIGBOOT' || norm === 'UARTSTREAM' || norm === 'HEXCAP' || norm === 'WAITTX') {
    rawOutput = [
      `=== ${norm}: PANASONIC 115200 8N1 UART CAPTURE ON GP2 (42 bytes, ${cleanArg || '7500'} ms) ===`,
      'AM34 IPL BOOT v1.18 OK | DDR CHECK OK',
      `${norm}: OK`,
    ].join('\n');
  } else if (norm === 'BAUD' || norm === 'BAUDSCAN') {
    rawOutput = [
      `=== ${norm} PIO1 AUTO-BAUD ===`,
      'Detected: 115200 bps (min_low=8 us)',
      `${norm}: OK`,
    ].join('\n');
  } else if (norm === 'PWRTRACE') {
    rawOutput = [
      `=== PWRTRACE DUAL RAIL + UART (4000 ms) ===`,
      `VDD_start=${mockVdd}V VDD_end=${mockVdd}V | GP2=872 GP3=2 | bytes=42`,
      'PWRTRACE: OK',
    ].join('\n');
  } else if (norm === 'PIOLOGIC') {
    rawOutput = [
      `=== PIOLOGIC PIO0 (4000ms @ 1000kHz, boot=2, hold=350ms) ===`,
      'Initial: RXD=1 NSW5V=1 TXD=1 XRESET=0 | fifo_stall=NO | edges=28',
      '  RXD   (GP2) : Final=1, Transitions=24 (rise=12 fall=12 minLow=8us minHigh=17us)',
      '  NSW5V (GP3) : Final=1, Transitions=0 (rise=0 fall=0 minLow=0us minHigh=0us)',
      '  TXD   (GP4) : Final=1, Transitions=3 (rise=2 fall=1 minLow=200us minHigh=1000us)',
      '  XRESET(GP5) : Final=1, Transitions=1 (rise=1 fall=0 minLow=0us minHigh=0us)',
      '[PIOLOGIC] XRESET_rise -> first_RXD_fall: 482 us',
      '[PIOLOGIC] TXD_fall -> RXD_fall turnaround: min=120 us max=120 us',
      'PIOLOGIC: OK',
    ].join('\n');
  } else if (norm === 'PANARSP') {
    rawOutput = [
      '=== PANARSP PIO1 (baud=115200 ms=6000 boot=2 hold=350 action=0) ===',
      '[PANARSP] RX (7 bytes):',
      '  0000: 15 2B 24 4F 4B 23 39 41                           |.+$OK#9A|',
      '[PANARSP] Frame: ack=+ nak=no nak0x15=yes | payload="OK" | csum=0x9A/0x9A (VALID)',
      'PANARSP: OK (rx=8 bytes, rsp=VALID)',
    ].join('\n');
  } else if (norm === 'LED') {
    const req = (cleanArg || 'STATUS').toUpperCase();
    rawOutput = `LED: OK (${req}, ${req === 'AUTO' ? 'AUTO' : 'MANUAL_LOCK'})`;
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
