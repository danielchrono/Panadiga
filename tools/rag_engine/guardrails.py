"""Query guardrails, prompt injection detection, and domain entity extraction.
"""

from __future__ import annotations

import re
from typing import List, Pattern, Set

from .models import Entity, EntityType, Query


class QueryGuard:
    """Pre-retrieval defensive firewall and domain entity extractor."""

    # Patterns indicating prompt injection or safety bypass attempts
    INJECTION_PATTERNS: List[Pattern[str]] = [
        re.compile(r"ignore\s+(all\s+)?(previous\s+)?instructions", re.IGNORECASE),
        re.compile(r"disregard\s+(all\s+)?(safety\s+)?rules", re.IGNORECASE),
        re.compile(r"bypass\s+(hardware\s+)?safety", re.IGNORECASE),
        re.compile(r"override\s+(system\s+)?(policy|limits?)", re.IGNORECASE),
        re.compile(r"jailbreak", re.IGNORECASE),
        re.compile(r"you\s+are\s+now\s+in\s+developer\s+mode", re.IGNORECASE),
        re.compile(r"do\s+anything\s+now", re.IGNORECASE),
        re.compile(r"act\s+as\s+an\s+unrestricted", re.IGNORECASE),
    ]

    # Domain Entity Regular Expressions
    HEX_REGEX: Pattern[str] = re.compile(r"\b0x[0-9A-Fa-f]{1,8}\b")
    VOLTAGE_REGEX: Pattern[str] = re.compile(r"\b\d+(?:\.\d+)?\s*(?:V|volts?)\b", re.IGNORECASE)
    ERROR_CODE_REGEX: Pattern[str] = re.compile(r"\bU\d{2}\b", re.IGNORECASE)
    LBA_REGEX: Pattern[str] = re.compile(r"\bLBA\s*\d+\b", re.IGNORECASE)

    KNOWN_COMPONENTS: Set[str] = {
        "MN2DS0016",
        "S29JL064H",
        "S29JL064H70TFI00",
        "VEP79138",
        "CN55001",
        "AM34",
        "UNIPHIER",
        "MN103S",
        "AN22111A",
        "C1AB00002870",
    }

    KNOWN_PINS_AND_SIGNALS: Set[str] = {
        "SDATA",
        "SCLK",
        "EXTRG",
        "NRESET",
        "RESET",
        "GND",
        "WP#",
        "ACC",
        "RY/BY#",
        "BYTE#",
        "TDI",
        "TDO",
        "TMS",
        "TCK",
        "TRST",
    }

    HIGH_RISK_KEYWORDS: Set[str] = {
        "erase",
        "write",
        "flash",
        "eeprom",
        "nvram",
        "u99",
        "voltage",
        "vcc",
        "overvoltage",
        "short",
        "jtag",
        "dwire",
        "dwire32a",
        "cn55001",
        "format",
        "diskpart",
        "partition",
        "meihdfs",
    }

    def process(self, raw_text: str) -> Query:
        """Sanitizes query, extracts entities, and classifies risk."""
        sanitized = raw_text.strip()

        # Check prompt injection
        is_injection = False
        val_error = None
        for pat in self.INJECTION_PATTERNS:
            if pat.search(sanitized):
                is_injection = True
                val_error = "Prompt injection or safety override attempt detected."
                break

        # Extract entities
        entities: List[Entity] = []

        # Hex addresses
        for match in self.HEX_REGEX.finditer(sanitized):
            entities.append(
                Entity(
                    name=match.group(0).upper(),
                    entity_type=EntityType.HEX_ADDRESS,
                    raw_value=match.group(0),
                )
            )

        # Voltages
        for match in self.VOLTAGE_REGEX.finditer(sanitized):
            entities.append(
                Entity(
                    name=match.group(0).upper(),
                    entity_type=EntityType.VOLTAGE,
                    raw_value=match.group(0),
                )
            )

        # Error codes
        for match in self.ERROR_CODE_REGEX.finditer(sanitized):
            entities.append(
                Entity(
                    name=match.group(0).upper(),
                    entity_type=EntityType.ERROR_CODE,
                    raw_value=match.group(0),
                )
            )

        # LBAs
        for match in self.LBA_REGEX.finditer(sanitized):
            entities.append(
                Entity(
                    name=match.group(0).upper(),
                    entity_type=EntityType.LBA_SECTOR,
                    raw_value=match.group(0),
                )
            )

        # Components & Pins
        upper_text = sanitized.upper()
        words = set(re.findall(r"\b[A-Z0-9_#/]+(?<![.,:;?!])", upper_text))

        for comp in self.KNOWN_COMPONENTS:
            if comp in words or comp in upper_text:
                entities.append(
                    Entity(
                        name=comp,
                        entity_type=EntityType.COMPONENT,
                        raw_value=comp,
                    )
                )

        for pin in self.KNOWN_PINS_AND_SIGNALS:
            if pin in words:
                entities.append(
                    Entity(
                        name=pin,
                        entity_type=EntityType.PIN,
                        raw_value=pin,
                    )
                )

        # High-risk classification
        query_words = set(re.findall(r"\w+", sanitized.lower()))
        is_high_risk = bool(query_words & self.HIGH_RISK_KEYWORDS) or any(
            e.entity_type in (EntityType.HEX_ADDRESS, EntityType.VOLTAGE) for e in entities
        )

        intent = "diagnostic" if "u99" in query_words or "error" in query_words else "general"

        return Query(
            raw_text=raw_text,
            sanitized_text=sanitized,
            intent=intent,
            extracted_entities=entities,
            is_high_risk=is_high_risk,
            is_injection=is_injection,
            validation_error=val_error,
        )
