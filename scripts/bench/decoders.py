"""Decoders for Pico P7501 telemetry (`STATUS`/`PINCHECK`/`BUSDIAG`) and AM34 UART logs."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
from typing import Final

_MECHA_ERROR_GLOSSARY: Final[dict[int, str]] = {
    0x83A7: "0.5mW発光ができない(HFｵﾝ密調整) — 0.5mW laser emission failure during HF ON fine tuning",
}

_SDCD_ERROR_GLOSSARY: Final[dict[int, str]] = {
    -91006: "ACMD13 SD Status card-type rejection in GetSD_Status",
    -90002: "SD card identification / bus width initialization failed",
    -90001: "No SD card detected by sdcd_insrmv_check",
}


@dataclass(frozen=True)
class StatusReport:
    """Decoded output of Pico `STATUS` or `VDD` command."""

    vdd_volts: float | None
    vdd_status: str
    gp26_sticky_low_suspected: bool
    raw_text: str


@dataclass(frozen=True)
class PinCheckRow:
    """One pin row parsed from `PINCHECK` output."""

    pin_label: str
    hiz_volts: float | None
    pull_down_volts: float | None
    pull_up_volts: float | None
    electrical_state: str


@dataclass(frozen=True)
class BusDiagReport:
    """Decoded transition counts and reset rise time from `BUSDIAG`."""

    gp2_transitions: int | None
    gp3_transitions: int | None
    gp4_transitions: int | None
    xreset_rise_us: int | None
    gp2_sole_active_tx: bool


@dataclass(frozen=True)
class SensePacket:
    """Decoded 18-byte ATAPI Sense packet captured from AM34 serial output."""

    raw_bytes: tuple[int, ...]
    sense_key: int
    asc: int
    ascq: int
    vendor_mecha_code: int
    mecha_description: str | None


@dataclass(frozen=True)
class UartBootReport:
    """Structured facts decoded from an AM34 115200 8N1 P7501 UART capture."""

    ddr_check_ok: bool
    loader_version: str | None
    boot_mode: str | None
    bgo_failed_code: int | None
    startup_reason: str | None
    sdcd_errors: tuple[int, ...]
    drive_error_codes: tuple[str, ...]
    sense_packets: tuple[SensePacket, ...]
    notes: tuple[str, ...]


def read_captured_log(path: Path) -> str:
    """Read a captured COM10 log file handling UTF-16LE/BE BOM or UTF-8 transparently."""
    raw = path.read_bytes()
    if raw.startswith(b"\xff\xfe") or raw.startswith(b"\xfe\xff"):
        return raw.decode("utf-16", errors="replace")
    # Also handle BOM-less UTF-16LE if null bytes appear in even/odd lanes early
    if len(raw) >= 4 and raw[1] == 0 and raw[3] == 0:
        return raw.decode("utf-16-le", errors="replace")
    return raw.decode("utf-8", errors="replace")


def _parse_float_token(token: str) -> float | None:
    match = re.search(r"([0-9]+(?:\.[0-9]+)?)\s*V", token, re.IGNORECASE)
    return float(match.group(1)) if match else None


def parse_status_output(text: str) -> StatusReport:
    """Parse Pico `STATUS` / `VDD` telemetry and detect sticky-LOW GP26 bug."""
    match = re.search(r"VDD:\s*([0-9]+(?:\.[0-9]+)?)\s*V\s*(?:\(([^)]+)\))?", text, re.IGNORECASE)
    vdd = float(match.group(1)) if match else None
    status = match.group(2).strip().upper() if match and match.group(2) else "UNKNOWN"
    sticky_low = vdd is not None and vdd <= 0.10
    return StatusReport(vdd, status, sticky_low, text)


def parse_pincheck_output(text: str) -> list[PinCheckRow]:
    """Parse tabular rows produced by the Pico `PINCHECK` electrical matrix check."""
    rows: list[PinCheckRow] = []
    for line in text.splitlines():
        if "|" not in line or "Pull-Down" in line:
            continue
        cols = [c.strip() for c in line.split("|")]
        if len(cols) < 5 or cols[0].lower().startswith("pin"):
            continue
        rows.append(
            PinCheckRow(
                pin_label=cols[0],
                hiz_volts=_parse_float_token(cols[1]),
                pull_down_volts=_parse_float_token(cols[2]),
                pull_up_volts=_parse_float_token(cols[3]),
                electrical_state=cols[4],
            )
        )
    return rows


def parse_busdiag_output(text: str) -> BusDiagReport:
    """Extract per-pin transition counters and `XRESET` rise time from `BUSDIAG`."""
    def _extract_trans(gpio: str) -> int | None:
        m = re.search(rf"{gpio}\b[^\r\n]*?:\s*(\d+)\s+trans", text, re.IGNORECASE)
        return int(m.group(1)) if m else None

    gp2 = _extract_trans("GP2")
    gp3 = _extract_trans("GP3")
    gp4 = _extract_trans("GP4")
    rise_m = re.search(r"Rise Time\s*\(Hi-Z\)\s*:\s*(\d+)\s*us", text, re.IGNORECASE)
    rise_us = int(rise_m.group(1)) if rise_m else None
    sole_tx = bool(gp2 is not None and gp2 > 0 and (gp3 in (None, 0)) and (gp4 in (None, 0)))
    return BusDiagReport(gp2, gp3, gp4, rise_us, sole_tx)


def parse_am34_uart_log(text: str) -> UartBootReport:
    """Decode bootloader milestones, SD errors, ATAPI errors, and Sense packets from a UART log."""
    ddr_ok = "DDR CHECK: OK" in text
    ver_m = re.search(r"Loader\s+(ver[0-9.]+\s*\[[^\]]+\])", text)
    loader_version = ver_m.group(1) if ver_m else None

    mode_m = re.search(r"\*{5}\s*(normal mode|serial boot mode|FEonly|support card)\s*\*{5}", text, re.IGNORECASE)
    boot_mode = mode_m.group(1) if mode_m else None

    bgo_m = re.search(r"BGO driver init failed\[(-?\d+)\]", text)
    bgo_failed = int(bgo_m.group(1)) if bgo_m else None

    reason_m = re.search(r"startup reason\s*=\s*\[([^\]]+)\]", text)
    startup_reason = reason_m.group(1) if reason_m else None

    sdcd_errs: list[int] = []
    for pat in (r"SD Status ERROR\s+(-?\d+)", r"SD init failed\[(-?\d+)\]"):
        for m in re.finditer(pat, text):
            code = int(m.group(1))
            if code not in sdcd_errs:
                sdcd_errs.append(code)

    drive_errs: list[str] = []
    for m in re.finditer(r"X\(([0-9a-fA-F]{6,8})\)", text):
        code_hex = m.group(1).upper()
        if code_hex not in drive_errs:
            drive_errs.append(code_hex)

    senses: list[SensePacket] = []
    seen_sense: set[tuple[int, ...]] = set()
    for m in re.finditer(r"SENSE\[([0-9a-fA-F\s]+)\]", text):
        byte_vals = tuple(int(tok, 16) for tok in m.group(1).split() if len(tok) == 2)
        if len(byte_vals) < 14 or byte_vals in seen_sense:
            continue
        seen_sense.add(byte_vals)
        sk = byte_vals[2] & 0x0F
        asc = byte_vals[12]
        ascq = byte_vals[13]
        mecha = (byte_vals[10] << 8) | byte_vals[11]
        senses.append(SensePacket(byte_vals, sk, asc, ascq, mecha, _MECHA_ERROR_GLOSSARY.get(mecha)))

    notes: list[str] = []
    for code in sdcd_errs:
        if code in _SDCD_ERROR_GLOSSARY:
            notes.append(f"SDCD {code}: {_SDCD_ERROR_GLOSSARY[code]}")
    for s in senses:
        if s.mecha_description:
            notes.append(f"Mecha 0x{s.vendor_mecha_code:04X}: {s.mecha_description}")

    return UartBootReport(
        ddr_check_ok=ddr_ok,
        loader_version=loader_version,
        boot_mode=boot_mode,
        bgo_failed_code=bgo_failed,
        startup_reason=startup_reason,
        sdcd_errors=tuple(sdcd_errs),
        drive_error_codes=tuple(drive_errs),
        sense_packets=tuple(senses),
        notes=tuple(notes),
    )
