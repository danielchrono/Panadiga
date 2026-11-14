import { describe, it, expect } from 'vitest';
import {
  CANONICAL_HARDWARE_SSOT_SPEC,
  PICO_MAX_VDD_CEILING_VOLTS,
  PICO_MIN_VDD_THRESHOLD_VOLTS,
  PICO_RP2350_CDC_PID,
} from '@/constants';
import { parseDiagnosticOutput, parseP7501DumpBytes } from './diagnosticParser';
import { simulateMockDiagnostic } from '@/test/mocks/mockDiagnostics';

describe('diagnosticParser & Hardware SSOT Consistency', () => {
  it('matches firmware SSOT constants and canonical P7501 UART pinout', () => {
    expect(PICO_MIN_VDD_THRESHOLD_VOLTS).toBe(2.5);
    expect(PICO_MAX_VDD_CEILING_VOLTS).toBe(3.45);
    expect(PICO_RP2350_CDC_PID).toBe(0x000f);
    expect(CANONICAL_HARDWARE_SSOT_SPEC.pinout).toHaveLength(6);
    expect(CANONICAL_HARDWARE_SSOT_SPEC.pinout[3].manual_pin).toBe(4);
    expect(CANONICAL_HARDWARE_SSOT_SPEC.pinout[4].signal_name).toContain('NSW5V');
    expect(CANONICAL_HARDWARE_SSOT_SPEC.pico_commands.length).toBeGreaterThanOrEqual(34);
  });

  it('parses PINCHECK matrix rows and XRESET rise time accurately', () => {
    const res = simulateMockDiagnostic('PINCHECK');
    expect(res.success).toBe(true);
    expect(res.pincheck_rows).toHaveLength(5);
    expect(res.xreset_rise_us).toBe('40 us');
    expect(res.vdd_volts).toBeCloseTo(3.28, 2);

    const liveHardwareOut = [
      '=== P7501 6-PIN ELECTRICAL MATRIX (PINCHECK) ===',
      'GP2 (Pin6 RXD   ) |  1   |     1     |    1    | STRONG HIGH (External Pull-Up / 3.3V Active)',
      'GP3 (Pin5 NSW5V ) |  1   |     1     |    1    | NSW5V HI-Z SENSE (Pulls Disabled Safe Rail)',
      'GP4 (Pin3 TXD   ) |  1   |     1     |    1    | STRONG HIGH (External Pull-Up / 3.3V Active)',
      'GP5 (Pin2 XRESET) |  1   |     1     |    1    | STRONG HIGH (Open-Drain Control)',
      'GP26(Pin1 SWUART) | 3.24V|   2.70V   |  3.26V  | P7501 SW_UART / Target Sense',
      'XRESET Release Rise Time (Hi-Z)     : 10 us',
      'PINCHECK: OK',
    ].join('\n');
    const liveParsed = parseDiagnosticOutput('PINCHECK', 'PINCHECK', liveHardwareOut, 393);
    expect(liveParsed.vdd_volts).toBeCloseTo(3.24, 2);
    expect(liveParsed.xreset_rise_us).toBe('10 us');

    const floatingOut = [
      'GP26(Pin1 SWUART) | 3.21V|   0.00V   |  3.28V  | P7501 SW_UART / Target Sense',
      'PINCHECK: OK',
    ].join('\n');
    const floatingParsed = parseDiagnosticOutput('PINCHECK', 'PINCHECK', floatingOut, 50);
    expect(floatingParsed.vdd_volts).toBe(0);
  });

  it('parses DUMP 32-bit words and little-endian ASCII representation', () => {
    const res = simulateMockDiagnostic('DUMP', '0x807F0000 16');
    expect(res.success).toBe(true);
    expect(res.dump_rows.length).toBeGreaterThanOrEqual(4);
    expect(res.dump_rows[0].address).toBe('0x807F0000');
    expect(res.dump_rows[0].words).toHaveLength(4);
  });

  it('marks FAILED or empty responses as unsuccessful', () => {
    const errRes = parseDiagnosticOutput('HANDSHAKE', 'HANDSHAKE', '[HANDSHAKE] FAILED: DCSR 0xFFFFFFFF', 120);
    expect(errRes.success).toBe(false);
    expect(errRes.summary).toContain('FAILED');
  });

  it('extracts little-endian raw byte payloads from DUMP output for live NVRAM ingestion', () => {
    const sampleDump = [
      '0x807F0000: 44434241 78563412 FFFFFFFF 00000000',
      '0x807F0010: CAFEBABE DEADBEEF 01020304 0A0B0C0D',
    ].join('\n');

    const extracted = parseP7501DumpBytes(sampleDump);
    expect(extracted).not.toBeNull();
    expect(extracted.baseAddress).toBe(0x807f0000);
    expect(extracted.bytes.length).toBe(32);
    // Word 0x44434241 in little-endian -> 0x41 ('A'), 0x42 ('B'), 0x43 ('C'), 0x44 ('D')
    expect(extracted.bytes.slice(0, 4)).toEqual([0x41, 0x42, 0x43, 0x44]);
    // Word 0x78563412 -> 0x12, 0x34, 0x56, 0x78
    expect(extracted.bytes.slice(4, 8)).toEqual([0x12, 0x34, 0x56, 0x78]);
  });

  it('keeps explicit OK command results successful even when target boot log prints failed/ERROR', () => {
    const raw = [
      '=== UARTSTREAM PIO1 (115200 bps, 120 bytes, 8000 ms) ===',
      'BGO driver init failed[1].',
      '[SDCD] Set SD Bus Width fail ERROR:-91006',
      'UARTSTREAM: OK',
    ].join('\n');
    const parsed = parseDiagnosticOutput('UARTSTREAM', 'UARTSTREAM 8000 JIG:350', raw, 8050);
    expect(parsed.success).toBe(true);

    const errParsed = parseDiagnosticOutput(
      'HANDSHAKE',
      'HANDSHAKE',
      '[HANDSHAKE] FAILED: DCSR=0xFFFFFFFF',
      300,
    );
    expect(errParsed.success).toBe(false);
  });
});
