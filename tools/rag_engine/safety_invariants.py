"""Deterministic hardware safety shield for the Panasonic DIGA DMR-EH55.

Enforces physical electrical, bootloader, and protocol invariants to prevent
irreversible equipment damage during reverse engineering and diagnostics.
"""

from __future__ import annotations

import re
from typing import Final, List, Pattern, Set, Tuple

from .config import (
    FORBIDDEN_FLASH_WRITE_END,
    FORBIDDEN_FLASH_WRITE_START,
    FORBIDDEN_JTAG_PINS,
    MAX_SAFE_VOLTAGE_VOLTS,
    SAFE_NVRAM_BASE_OFFSET,
    SAFE_NVRAM_END_OFFSET,
)
from .models import SafetyReport

# Bus address constants for AM34 physical memory map
FORBIDDEN_FLASH_BUS_START: Final[int] = 0x80000000
FORBIDDEN_FLASH_BUS_END: Final[int] = 0x807EFFFF
SAFE_NVRAM_BUS_START: Final[int] = 0x807F0000
SAFE_NVRAM_BUS_END: Final[int] = 0x807FFFFF


def extract_voltages_volts(text: str) -> List[Tuple[float, int, int, str]]:
    """Extracts all voltage references from text and normalizes them to Volts (float).

    Supports:
    - Standard decimal/integer: 3.3V, 5V, 12V, 3.3VDC, 5.0 VAC, 3.3 volts
    - Comma decimals: 12,0V, 3,3V
    - EE RKM notation: 3V3 (3.3V), 5V0 (5.0V), 1V8 (1.8V)
    - Power rail labels: 5V_STBY, 12V_RAIL, +5V_AUX
    - Millivolts: 5000mV, 5000 mV, 3300mV (divided by 1000.0)

    Returns:
        List of (voltage_in_volts, start_char_idx, end_char_idx, matched_string).
    """
    results: List[Tuple[float, int, int, str]] = []

    # 1. Millivolts (e.g. 5000mV, 5000 mV, 3300mV)
    mv_pattern = re.compile(
        r"(?i)(?:^|[^\w])(\d+(?:[.,]\d+)?)\s*m(?:vdc|vac|v|volts?)(?:_[a-z0-9_]+|\b)"
    )
    for m in mv_pattern.finditer(text):
        raw_num = m.group(1).replace(",", ".")
        try:
            val = float(raw_num) / 1000.0
            results.append((val, m.start(), m.end(), m.group(0).strip()))
        except ValueError:
            pass

    # 2. EE RKM notation (e.g. 5V0, 3V3, 1V8)
    rkm_pattern = re.compile(r"(?i)\b(\d+)[vV](\d+)(?:_[a-zA-Z0-9_]+|\b)")
    for m in rkm_pattern.finditer(text):
        try:
            val = float(f"{m.group(1)}.{m.group(2)}")
            results.append((val, m.start(), m.end(), m.group(0).strip()))
        except ValueError:
            pass

    # 3. Standard Volts with comma/period and optional rail suffix (_STBY, _RAIL)
    std_pattern = re.compile(
        r"(?i)(?<!m)(?<![a-zA-Z0-9_])(\d+(?:[.,]\d+)?)\s*(?:vdc|vac|v|volts?)(?:_[a-zA-Z0-9_]+|\b)"
    )
    for m in std_pattern.finditer(text):
        raw_num = m.group(1).replace(",", ".")
        try:
            val = float(raw_num)
            results.append((val, m.start(), m.end(), m.group(0).strip()))
        except ValueError:
            pass

    return results


class HardwareSafetyValidator:
    """Enforces absolute hardware physical invariants with zero tolerance."""

    HEX_PATTERN: Pattern[str] = re.compile(r"\b0x[0-9A-Fa-f]+\b")

    # Pattern detecting direct intent to erase/write IPL sectors SA0..SA133
    IPL_ERASE_PATTERN: Pattern[str] = re.compile(
        r"\b(?:erase|erasing|wipe|wiping|clear|clearing|destroy|overwrite|program|write|writing)\s+(?:the\s+)?(?:flash\s+)?(?:sectors?\s+)?(?:sa0|sa133|ipl)\b",
        re.IGNORECASE,
    )

    # Strict word-boundary regex for negation and warning words (OPUS-02)
    # Prevents false positives on substrings like 'note', 'nothing', 'annotation'
    NEGATION_REGEX: Pattern[str] = re.compile(
        r"\b(?:never|not|no|do\s+not|must\s+not|cannot|prohibited|forbidden|avoid|warning|danger|destroy|damage|destruction|fatal|burn|bricks?|hazard|violation|exceeds|blocks|preserve|preserving|immutable)\b",
        re.IGNORECASE,
    )

    WRITE_ERASE_WORDS: Set[str] = {
        "erase",
        "erasing",
        "wipe",
        "wiping",
        "write",
        "writing",
        "overwrite",
        "program",
        "programming",
        "flash",
        "flashing",
        "clear",
        "clearing",
        "destroy",
    }

    PC_DISK_UTILS: Set[str] = {
        "diskpart",
        "mkfs",
        "fdisk",
        "format c:",
        "format d:",
        "format e:",
        "format /fs:ntfs",
        "format /fs:fat32",
    }

    def validate(self, text: str, is_query: bool = False) -> SafetyReport:
        """Inspects text for physical safety violations."""
        violations: List[str] = []
        alerts: List[str] = []
        warnings: List[str] = []

        lower_text = text.lower()
        upper_text = text.upper()

        # ----------------------------------------------------------------------
        # 1. Electrical Invariant: Absolute Max Voltage <= 3.35V (OPUS-10)
        # ----------------------------------------------------------------------
        for val, start_idx, end_idx, matched_str in extract_voltages_volts(text):
            if val > MAX_SAFE_VOLTAGE_VOLTS:
                start_ctx = max(0, start_idx - 60)
                for p in [".", ";", "!", "\n"]:
                    punct_idx = lower_text.rfind(p, start_ctx, start_idx)
                    if punct_idx != -1:
                        start_ctx = max(start_ctx, punct_idx + 1)

                end_ctx = min(len(text), end_idx + 60)
                for p in [".", ";", "!", "\n"]:
                    punct_idx = lower_text.find(p, end_idx, end_ctx)
                    if punct_idx != -1:
                        end_ctx = min(end_ctx, punct_idx)

                context_window = lower_text[start_ctx:end_ctx]

                has_warning = bool(self.NEGATION_REGEX.search(context_window))

                # If it's a query asking to apply > 3.3V, or text suggesting it without warning
                if is_query or not has_warning:
                    violations.append(
                        f"CRITICAL ELECTRICAL HAZARD: Voltage {val}V ({matched_str}) exceeds maximum safe tolerance of {MAX_SAFE_VOLTAGE_VOLTS}V on MN2DS0016 / VEP79138 digital board."
                    )
                    alerts.append(
                        f"Overvoltage detected ({val}V > {MAX_SAFE_VOLTAGE_VOLTS}V). Immediate risk of gate breakdown."
                    )
                else:
                    warnings.append(
                        f"High voltage {val}V referenced in warning or prohibition context."
                    )

        # ----------------------------------------------------------------------
        # 2. Bootloader Invariant: SA0..SA133 is strictly read-only (OPUS-02)
        # Check both file offsets (0x00000000..0x007EFFFF) and AM34 bus addresses (0x80000000..0x807EFFFF)
        # ----------------------------------------------------------------------
        found_hex_matches = self.HEX_PATTERN.finditer(text)
        global_has_write = any(w in lower_text for w in self.WRITE_ERASE_WORDS)

        for h_match in found_hex_matches:
            raw_hex = h_match.group(0)
            try:
                addr = int(raw_hex, 16)
            except ValueError:
                continue

            start_ctx = max(0, h_match.start() - 60)
            for p in [".", ";", "!", "\n"]:
                punct_idx = lower_text.rfind(p, start_ctx, h_match.start())
                if punct_idx != -1:
                    start_ctx = max(start_ctx, punct_idx + 1)

            end_ctx = min(len(text), h_match.end() + 60)
            for p in [".", ";", "!", "\n"]:
                punct_idx = lower_text.find(p, h_match.end(), end_ctx)
                if punct_idx != -1:
                    end_ctx = min(end_ctx, punct_idx)

            context_window = lower_text[start_ctx:end_ctx]

            # Ignore 1-byte data constants and bitmasks (e.g. 0xFF, 0xA0)
            if addr != 0 and addr < 0x1000 and len(raw_hex) <= 4:
                continue

            is_address = (
                len(raw_hex) >= 6
                or addr >= 0x1000
                or any(
                    k in context_window
                    for k in [
                        "address",
                        "offset",
                        "flash",
                        "starting at",
                        "from 0x",
                        "sector",
                        "sa0",
                        "sa133",
                        "range",
                    ]
                )
            )
            if not is_address:
                continue

            # Check if address falls in forbidden IPL range (file offset OR bus address)
            is_ipl_address = (
                FORBIDDEN_FLASH_WRITE_START <= addr <= FORBIDDEN_FLASH_WRITE_END
            ) or (FORBIDDEN_FLASH_BUS_START <= addr <= FORBIDDEN_FLASH_BUS_END)

            if is_ipl_address:
                has_local_write = any(w in context_window for w in self.WRITE_ERASE_WORDS) or global_has_write
                has_warning = bool(self.NEGATION_REGEX.search(context_window))

                # OPUS-02 FIX: If it is a query attempting write/erase, unconditionally block regardless
                # of negation words in the query (e.g. 'Erase flash starting at 0x00000000, do not touch anything else')
                if (is_query and has_local_write) or (not is_query and has_local_write and not has_warning):
                    violations.append(
                        f"CRITICAL BOOTLOADER HAZARD: Flash address {raw_hex.upper()} is within protected IPL range "
                        f"(SA0..SA133: {hex(FORBIDDEN_FLASH_WRITE_START)}..{hex(FORBIDDEN_FLASH_WRITE_END)} / "
                        f"{hex(FORBIDDEN_FLASH_BUS_START)}..{hex(FORBIDDEN_FLASH_BUS_END)}). "
                        "Erasing or writing this region permanently bricks the Root of Trust."
                    )
                    alerts.append(
                        f"Attempted write/erase at IPL address {raw_hex.upper()}. Operation vetoed by air-gap shield."
                    )
                elif has_warning:
                    warnings.append(
                        f"IPL address {raw_hex.upper()} referenced in protective warning context."
                    )
            elif (
                (SAFE_NVRAM_BASE_OFFSET <= addr <= SAFE_NVRAM_END_OFFSET)
                or (SAFE_NVRAM_BUS_START <= addr <= SAFE_NVRAM_BUS_END)
            ):
                # Safe region for U99 NVRAM parameter clearing
                pass

        # Also check for explicit direct intent to erase SA0..SA133 / IPL
        for ipl_match in self.IPL_ERASE_PATTERN.finditer(text):
            start_ctx = max(0, ipl_match.start() - 60)
            end_ctx = min(len(text), ipl_match.end() + 60)
            context_window = lower_text[start_ctx:end_ctx]
            has_warning = bool(self.NEGATION_REGEX.search(context_window))

            if is_query or not has_warning:
                violations.append(
                    f"CRITICAL BOOTLOADER HAZARD: Direct instruction to modify IPL detected ('{ipl_match.group(0)}'). "
                    "Erasing IPL sectors SA0..SA133 is strictly prohibited."
                )

        # ----------------------------------------------------------------------
        # 3. Protocol Invariant: CN55001 is DWire32A, NOT Standard JTAG
        # ----------------------------------------------------------------------
        if "CN55001" in upper_text or "CN5501" in upper_text:
            mentioned_jtag = [pin for pin in FORBIDDEN_JTAG_PINS if pin in upper_text]
            if mentioned_jtag:
                violations.append(
                    f"CRITICAL PROTOCOL HAZARD: CN55001 is a proprietary Matsushita DWire32A port (SDATA, SCLK, EXTRG, nRESET, GND). "
                    f"Standard JTAG pins ({', '.join(mentioned_jtag)}) do NOT exist on CN55001. Direct connection causes bus contention."
                )
                alerts.append(
                    f"Forbidden JTAG pin assignment on CN55001: {', '.join(mentioned_jtag)}"
                )

        # ----------------------------------------------------------------------
        # 4. Storage Invariant: MEIHDFS Hard Drive Structure Protection
        # ----------------------------------------------------------------------
        for util in self.PC_DISK_UTILS:
            if util in lower_text:
                if not bool(self.NEGATION_REGEX.search(lower_text)):
                    violations.append(
                        f"CRITICAL STORAGE HAZARD: PC utility '{util}' destroys MEIHDFS 'SH' signature and firmware partitions."
                    )
                    alerts.append(
                        f"Destructive disk formatting utility detected: {util}"
                    )

        is_safe = len(violations) == 0
        return SafetyReport(
            is_safe=is_safe,
            violations=violations,
            critical_alerts=alerts,
            warnings=warnings,
        )
