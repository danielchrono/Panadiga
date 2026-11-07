"""Deterministic hardware safety shield for the Panasonic DIGA DMR-EH55.

Enforces physical electrical, bootloader, and protocol invariants to prevent
irreversible equipment damage during reverse engineering and diagnostics.
"""

from __future__ import annotations

import re
from typing import List, Pattern, Set

from .config import (
    FORBIDDEN_FLASH_WRITE_END,
    FORBIDDEN_FLASH_WRITE_START,
    FORBIDDEN_JTAG_PINS,
    MAX_SAFE_VOLTAGE_VOLTS,
    SAFE_NVRAM_BASE_OFFSET,
    SAFE_NVRAM_END_OFFSET,
)
from .models import SafetyReport


class HardwareSafetyValidator:
    """Enforces absolute hardware physical invariants with zero tolerance."""

    VOLTAGE_PATTERN: Pattern[str] = re.compile(
        r"\b(\d+(?:\.\d+)?)\s*(?:VDC|VAC|V|volts?)\b", re.IGNORECASE
    )
    HEX_PATTERN: Pattern[str] = re.compile(r"\b0x[0-9A-Fa-f]+\b")

    NEGATION_WARNING_WORDS: Set[str] = {
        "never",
        "not",
        "prohibited",
        "forbidden",
        "destroy",
        "destruction",
        "damage",
        "fatal",
        "burn",
        "cannot",
        "must not",
        "do not",
        "avoid",
        "exceeds",
        "violation",
        "hazard",
        "blocks",
        "danger",
        "warning",
    }

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
        # 1. Electrical Invariant: Absolute Max Voltage <= 3.35V
        # ----------------------------------------------------------------------
        for match in self.VOLTAGE_PATTERN.finditer(text):
            try:
                val = float(match.group(1))
            except ValueError:
                continue

            if val > MAX_SAFE_VOLTAGE_VOLTS:
                start_ctx = max(0, match.start() - 60)
                for p in [".", ";", "!", "\n"]:
                    punct_idx = lower_text.rfind(p, start_ctx, match.start())
                    if punct_idx != -1:
                        start_ctx = max(start_ctx, punct_idx + 1)

                end_ctx = min(len(text), match.end() + 60)
                for p in [".", ";", "!", "\n"]:
                    punct_idx = lower_text.find(p, match.end(), end_ctx)
                    if punct_idx != -1:
                        end_ctx = min(end_ctx, punct_idx)

                context_window = lower_text[start_ctx:end_ctx]

                has_warning = any(w in context_window for w in self.NEGATION_WARNING_WORDS)

                # If it's a query asking to apply > 3.3V, or text suggesting it without warning
                if is_query or not has_warning:
                    violations.append(
                        f"CRITICAL ELECTRICAL HAZARD: Voltage {val}V exceeds maximum safe tolerance of {MAX_SAFE_VOLTAGE_VOLTS}V on MN2DS0016 / VEP79138 digital board."
                    )
                    alerts.append(
                        f"Overvoltage detected ({val}V > {MAX_SAFE_VOLTAGE_VOLTS}V). Immediate risk of gate breakdown."
                    )
                else:
                    warnings.append(
                        f"High voltage {val}V referenced in warning or prohibition context."
                    )

        # ----------------------------------------------------------------------
        # 2. Bootloader Invariant: SA0..SA133 (0x00000000..0x007EFFFF) is read-only
        # ----------------------------------------------------------------------
        found_hex_matches = self.HEX_PATTERN.finditer(text)

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

            # Ignore 1-byte data constants and bitmasks (e.g. 0xFF, 0xA0).
            # If length is >= 6 (e.g. 0x00000800 in SA0), it is a real memory address.
            if addr != 0 and addr < 0x1000 and len(raw_hex) <= 4:
                continue

            # Distinguish Flash addresses from small constants
            is_address = (
                len(raw_hex) >= 6
                or addr >= 0x1000
                or any(k in context_window for k in ["address", "offset", "flash", "starting at", "from 0x", "sector", "sa0", "sa133", "range"])
            )
            if not is_address:
                continue

            # Check if address falls in forbidden IPL range
            if FORBIDDEN_FLASH_WRITE_START <= addr <= FORBIDDEN_FLASH_WRITE_END:
                has_local_write = any(w in context_window for w in self.WRITE_ERASE_WORDS)
                has_warning = any(w in context_window for w in self.NEGATION_WARNING_WORDS)

                if (has_local_write or is_query) and not has_warning:
                    violations.append(
                        f"CRITICAL BOOTLOADER HAZARD: Flash address {raw_hex.upper()} is within protected IPL range "
                        f"(SA0..SA133: {hex(FORBIDDEN_FLASH_WRITE_START)}..{hex(FORBIDDEN_FLASH_WRITE_END)}). "
                        "Erasing or writing this region permanently bricks the Root of Trust."
                    )
                    alerts.append(
                        f"Attempted write/erase at IPL address {raw_hex.upper()}. Operation vetoed by air-gap shield."
                    )
                elif has_warning:
                    warnings.append(
                        f"IPL address {raw_hex.upper()} referenced in protective warning context."
                    )
            elif SAFE_NVRAM_BASE_OFFSET <= addr <= SAFE_NVRAM_END_OFFSET:
                # Safe region for U99
                pass

        # Also check for explicit mentions of SA0..SA133 erase
        if any(w in lower_text for w in self.WRITE_ERASE_WORDS) and ("sa0" in lower_text or "sa133" in lower_text or "ipl" in lower_text):
            if not any(w in lower_text for w in self.NEGATION_WARNING_WORDS):
                violations.append(
                    "CRITICAL BOOTLOADER HAZARD: Erasing IPL sectors SA0..SA133 is strictly prohibited."
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
                if not any(w in lower_text for w in self.NEGATION_WARNING_WORDS):
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
