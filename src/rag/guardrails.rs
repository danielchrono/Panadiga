//! Query guardrails, prompt injection detection, and domain entity extraction.

use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

use super::models::{Entity, EntityType, Query};
use super::safety::extract_voltages_volts;

static INJECTION_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)ignore\s+(all\s+)?(previous\s+)?instructions").unwrap(),
        Regex::new(r"(?i)disregard\s+(all\s+)?(safety\s+)?rules").unwrap(),
        Regex::new(r"(?i)bypass\s+(hardware\s+)?safety").unwrap(),
        Regex::new(r"(?i)override\s+(system\s+)?(policy|limits?)").unwrap(),
        Regex::new(r"(?i)jailbreak").unwrap(),
        Regex::new(r"(?i)you\s+are\s+now\s+in\s+developer\s+mode").unwrap(),
        Regex::new(r"(?i)do\s+anything\s+now").unwrap(),
        Regex::new(r"(?i)act\s+as\s+an\s+unrestricted").unwrap(),
    ]
});

static HEX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b0x[0-9A-Fa-f]{1,16}\b").unwrap()
});

static ERROR_CODE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bU\d{2}\b").unwrap()
});

static LBA_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bLBA\s*\d+\b").unwrap()
});

static WORD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[A-Za-z0-9_#/]+").unwrap()
});

pub const KNOWN_COMPONENTS: &[&str] = &[
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
];

pub const KNOWN_PINS_AND_SIGNALS: &[&str] = &[
    "SDATA", "SCLK", "EXTRG", "NRESET", "RESET", "GND", "WP#", "ACC", "RY/BY#",
    "BYTE#", "TDI", "TDO", "TMS", "TCK", "TRST",
];

const HIGH_RISK_KEYWORDS: &[&str] = &[
    "erase", "write", "flash", "eeprom", "nvram", "u99", "voltage", "vcc",
    "overvoltage", "short", "jtag", "dwire", "dwire32a", "cn55001", "format",
    "diskpart", "partition", "meihdfs",
];

pub struct QueryGuard;

impl QueryGuard {
    pub fn new() -> Self {
        Self
    }

    pub fn process(&self, raw_text: &str) -> Query {
        let sanitized = raw_text.trim().to_string();

        let mut is_injection = false;
        let mut val_error = None;
        for pat in INJECTION_PATTERNS.iter() {
            if pat.is_match(&sanitized) {
                is_injection = true;
                val_error = Some("Prompt injection or safety override attempt detected.".to_string());
                break;
            }
        }

        let mut entities = Vec::new();

        // 1. Hex addresses
        for m in HEX_REGEX.find_iter(&sanitized) {
            entities.push(Entity {
                name: m.as_str().to_uppercase(),
                entity_type: EntityType::HexAddress,
                raw_value: m.as_str().to_string(),
            });
        }

        // 2. Voltages
        for (val, _, _, matched_str) in extract_voltages_volts(&sanitized) {
            entities.push(Entity {
                name: format!("{}V", val),
                entity_type: EntityType::Voltage,
                raw_value: matched_str,
            });
        }

        // 3. Error codes
        for m in ERROR_CODE_REGEX.find_iter(&sanitized) {
            entities.push(Entity {
                name: m.as_str().to_uppercase(),
                entity_type: EntityType::ErrorCode,
                raw_value: m.as_str().to_string(),
            });
        }

        // 4. LBA Sectors
        for m in LBA_REGEX.find_iter(&sanitized) {
            entities.push(Entity {
                name: m.as_str().to_uppercase(),
                entity_type: EntityType::LbaSector,
                raw_value: m.as_str().to_string(),
            });
        }

        // 5. Components and Pins
        let upper_text = sanitized.to_uppercase();
        let words: HashSet<&str> = WORD_REGEX
            .find_iter(&upper_text)
            .map(|m| m.as_str())
            .collect();

        for &comp in KNOWN_COMPONENTS {
            if words.contains(comp) || upper_text.contains(comp) {
                entities.push(Entity {
                    name: comp.to_string(),
                    entity_type: EntityType::Component,
                    raw_value: comp.to_string(),
                });
            }
        }

        for &pin in KNOWN_PINS_AND_SIGNALS {
            if words.contains(pin) {
                entities.push(Entity {
                    name: pin.to_string(),
                    entity_type: EntityType::Pin,
                    raw_value: pin.to_string(),
                });
            }
        }

        let lower_words: HashSet<String> = WORD_REGEX
            .find_iter(&sanitized.to_lowercase())
            .map(|m| m.as_str().to_string())
            .collect();

        let is_high_risk = HIGH_RISK_KEYWORDS.iter().any(|k| lower_words.contains(*k))
            || entities.iter().any(|e| {
                e.entity_type == EntityType::HexAddress || e.entity_type == EntityType::Voltage
            });

        let intent = if lower_words.contains("u99") || lower_words.contains("error") {
            "diagnostic".to_string()
        } else {
            "general".to_string()
        };

        Query {
            raw_text: raw_text.to_string(),
            sanitized_text: sanitized,
            intent,
            extracted_entities: entities,
            is_high_risk,
            is_injection,
            validation_error: val_error,
        }
    }
}

impl Default for QueryGuard {
    fn default() -> Self {
        Self::new()
    }
}
