import type {
  BusTransitionRow,
  DumpWordRow,
  PicoDiagnosticResult,
  PincheckRow,
} from '@/models/pico';

function parsePincheckRows(rawOutput: string): { rows: PincheckRow[]; riseTime: string | null } {
  const rows: PincheckRow[] = [];
  let riseTime: string | null = null;

  for (const line of rawOutput.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (
      trimmed.includes('|') &&
      (trimmed.startsWith('GP') || trimmed.includes('Pin')) &&
      !trimmed.startsWith('Pin ') &&
      !trimmed.includes('---+')
    ) {
      const parts = trimmed.split('|').map(s => s.trim());
      if (parts.length >= 5) {
        rows.push({
          pin_label: parts[0],
          hiz: parts[1],
          pull_down: parts[2],
          pull_up: parts[3],
          state: parts[4],
        });
      }
    } else if (trimmed.includes('XRESET Release Rise Time')) {
      const idx = trimmed.indexOf(':');
      if (idx >= 0) {
        riseTime = trimmed.slice(idx + 1).trim();
      }
    }
  }

  return { rows, riseTime };
}

function parseTransitionRows(rawOutput: string): BusTransitionRow[] {
  const rows: BusTransitionRow[] = [];
  for (const line of rawOutput.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed.includes('Final=') && trimmed.includes('Transitions=') && trimmed.includes(':')) {
      const colonIdx = trimmed.indexOf(':');
      const left = trimmed.slice(0, colonIdx).trim();
      const right = trimmed.slice(colonIdx + 1);
      const finalMatch = right.match(/Final=(\d+)/);
      const transMatch = right.match(/Transitions=(\d+)/);
      const leftParts = left.split(/\s+/);
      rows.push({
        signal: leftParts[0] || 'SIG',
        gpio: (leftParts[1] || '').replace(/[()]/g, ''),
        final_level: finalMatch ? Number.parseInt(finalMatch[1], 10) : null,
        transitions: transMatch ? Number.parseInt(transMatch[1], 10) : 0,
      });
    }
  }
  return rows;
}

function pushHexWordLe(wordHex: string, out: number[]): void {
  const value = Number.parseInt(wordHex, 16);
  if (Number.isNaN(value)) return;
  out.push(value & 0xff, (value >>> 8) & 0xff, (value >>> 16) & 0xff, (value >>> 24) & 0xff);
}

function parseDumpRows(rawOutput: string): DumpWordRow[] {
  const rows: DumpWordRow[] = [];
  for (const line of rawOutput.split(/\r?\n/)) {
    const trimmed = line.trim();
    const colonIdx = trimmed.indexOf(':');
    if (colonIdx <= 0) continue;
    const addrRaw = trimmed.slice(0, colonIdx).trim().replace(/^0x/i, '');
    if (!/^[0-9a-fA-F]{8}$/.test(addrRaw)) continue;

    const valsPart = trimmed.slice(colonIdx + 1).trim();
    const words: string[] = [];
    const bytes: number[] = [];
    for (const tok of valsPart.split(/\s+/)) {
      const cleanTok = tok.replace(/^0x/i, '');
      if (/^[0-9a-fA-F]{8}$/.test(cleanTok)) {
        words.push(`0x${cleanTok.toUpperCase()}`);
        pushHexWordLe(cleanTok, bytes);
      }
    }
    if (words.length > 0) {
      const ascii = bytes.map(b => (b >= 0x20 && b <= 0x7e ? String.fromCharCode(b) : '.')).join('');
      rows.push({
        address: `0x${addrRaw.toUpperCase()}`,
        words,
        ascii,
      });
    }
  }
  return rows;
}

function parseVddVolts(rawOutput: string): number | null {
  const match = rawOutput.match(/VDD[=:\s]+(\d+(?:\.\d+)?)\s*V/i);
  if (!match) return null;
  const v = Number.parseFloat(match[1]);
  return Number.isFinite(v) && v > 0.01 ? v : null;
}

function parseCapturedBytes(rawOutput: string): number | null {
  const match = rawOutput.match(/(\d+)\s+bytes/i);
  return match ? Number.parseInt(match[1], 10) : null;
}

function parseFlashMb(rawOutput: string): number | null {
  for (const mb of [16, 8, 4, 2]) {
    if (rawOutput.includes(`${mb} MB`)) return mb;
  }
  return null;
}

/**
 * Shared structured diagnostic parser used by Android BLE / USB-OTG and USB CDC transports.
 */
export function parseDiagnosticOutput(
  commandId: string,
  rawCommand: string,
  rawOutput: string,
  elapsedMs: number
): PicoDiagnosticResult {
  const norm = commandId.trim().toUpperCase();
  const normLower = norm.toLowerCase();
  const lower = rawOutput.toLowerCase();
  const linesTrimmed = lower.split(/\r?\n/).map(l => l.trim());
  const cmdOkPrefix = `${normLower}: ok`;
  const cmdErrPrefix = `${normLower}: error`;
  const bracketErrPrefix = `[${normLower}] error:`;

  const hasExplicitOk = linesTrimmed.some(
    line => line.startsWith(cmdOkPrefix) || (normLower === 'scan' && line === 'scan: completed'),
  );
  const hasPicoFatal = linesTrimmed.some(
    line =>
      line.startsWith(cmdErrPrefix) ||
      line.startsWith(bracketErrPrefix) ||
      line.startsWith('[safety interlock]') ||
      line.startsWith('[handshake] failed') ||
      line.startsWith('unknown command:'),
  );

  const failed =
    rawOutput.trim().length === 0 ||
    hasPicoFatal ||
    (!hasExplicitOk &&
      (lower.includes('failed') ||
        lower.includes('error') ||
        lower.includes('abort') ||
        lower.includes('safety interlock')));

  const pinData = norm === 'PINCHECK' ? parsePincheckRows(rawOutput) : { rows: [], riseTime: null };
  const transitionRows = parseTransitionRows(rawOutput);
  const dumpRows = norm === 'DUMP' ? parseDumpRows(rawOutput) : [];
  const meaningfulLines = rawOutput
    .split(/\r?\n/)
    .map(l => l.trim())
    .filter(l => l.length > 0 && !l.startsWith('PANADIGA>') && !l.startsWith('P7501>'));

  const directVdd = parseVddVolts(rawOutput);
  const vddRow = pinData.rows.find(
    r => r.pin_label.includes('SWUART') || r.pin_label.includes('GP26') || r.pin_label.includes('VDD'),
  );
  const pinPdParsed = vddRow ? Number.parseFloat(vddRow.pull_down) : NaN;
  const pinHizParsed = vddRow ? Number.parseFloat(vddRow.hiz) : NaN;
  let pinVddParsed: number | null = null;
  if (Number.isFinite(pinPdParsed) && pinPdParsed < 1.5) {
    pinVddParsed = pinPdParsed;
  } else if (Number.isFinite(pinHizParsed)) {
    pinVddParsed = pinHizParsed;
  } else if (Number.isFinite(pinPdParsed)) {
    pinVddParsed = pinPdParsed;
  }
  const vddVolts = directVdd ?? pinVddParsed;

  return {
    command_id: norm,
    raw_command: rawCommand,
    success: !failed,
    elapsed_ms: elapsedMs,
    summary:
      meaningfulLines[meaningfulLines.length - 1] ??
      `Sem resposta no barramento para '${rawCommand}'`,
    raw_output: rawOutput,
    vdd_volts: vddVolts,
    detected_flash_mb: parseFlashMb(rawOutput),
    xreset_rise_us: pinData.riseTime,
    pincheck_rows: pinData.rows,
    transition_rows: transitionRows,
    dump_rows: dumpRows,
    captured_bytes: parseCapturedBytes(rawOutput),
  };
}

/**
 * Decodes `ADDR: w0 w1 w2 w3` lines returned by the Pico `DUMP` command into a Little-Endian byte array.
 */
export function parseP7501DumpBytes(rawOutput: string): { baseAddress: number | null; bytes: number[] } {
  const bytes: number[] = [];
  let baseAddress: number | null = null;

  for (const rawLine of rawOutput.split(/\r?\n/)) {
    const trimmed = rawLine.trim();
    const colonIdx = trimmed.indexOf(':');
    if (colonIdx <= 0) continue;

    const addrPart = trimmed.slice(0, colonIdx).trim().replace(/^0x/i, '');
    if (!/^[0-9a-fA-F]{4,8}$/.test(addrPart)) continue;

    const parsedAddr = Number.parseInt(addrPart, 16);
    if (Number.isNaN(parsedAddr)) continue;
    if (baseAddress === null) {
      baseAddress = parsedAddr;
    }

    const wordsPart = trimmed.slice(colonIdx + 1).trim();
    for (const token of wordsPart.split(/\s+/)) {
      const cleanToken = token.replace(/^0x/i, '');
      if (!/^[0-9a-fA-F]{8}$/.test(cleanToken)) continue;
      const word = Number.parseInt(cleanToken, 16) >>> 0;
      bytes.push(word & 0xff, (word >>> 8) & 0xff, (word >>> 16) & 0xff, (word >>> 24) & 0xff);
    }
  }

  return { baseAddress, bytes };
}
