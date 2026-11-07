//! Deterministic hardware safety shield for the Panasonic DIGA DMR-EH55.
//!
//! Enforces physical electrical, bootloader, and protocol invariants to prevent
//! irreversible equipment damage during reverse engineering and diagnostics.

use regex::Regex;
use std::sync::LazyLock;

use super::config::*;
use super::models::SafetyReport;

static MV_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:^|[^\w])(\d+(?:[.,]\d+)?)\s*m(?:vdc|vac|v|volts?)(?:_[a-z0-9_]+|\b)")
        .expect("Invalid regex: MV_PATTERN")
});

static RKM_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(\d+)[vV](\d+)(?:_[a-zA-Z0-9_]+|\b)")
        .expect("Invalid regex: RKM_PATTERN")
});

static STD_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*(?:vdc|vac|v|volts?)(?:_[a-zA-Z0-9_]+|\b)")
        .expect("Invalid regex: STD_PATTERN")
});

static HEX_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b0x[0-9A-Fa-f]+\b").expect("Invalid regex: HEX_PATTERN")
});

static IPL_ERASE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:erase|erasing|wipe|wiping|clear|clearing|destroy|overwrite|program|write|writing)\s+(?:the\s+)?(?:flash\s+)?(?:sectors?\s+)?(?:sa0|sa133|ipl)\b")
        .expect("Invalid regex: IPL_ERASE_PATTERN")
});

static NEGATION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:never|not|no|do\s+not|must\s+not|cannot|prohibited|forbidden|avoid|warning|danger|destroy|damage|destruction|fatal|burn|bricks?|hazard|violation|exceeds|blocks|preserve|preserving|immutable)\b")
        .expect("Invalid regex: NEGATION_REGEX")
});

/// Extracts all voltage references from text and normalizes them to Volts (f64).
///
/// Supports:
/// - Millivolts: 5000mV, 5000 mV, 3300mV (divided by 1000.0)
/// - EE RKM notation: 5V0 (5.0V), 3V3 (3.3V), 1V8 (1.8V)
/// - Standard: 3.3V, 5V, 12V, 12,0V, 5V_STBY, 12V_RAIL
///
/// Returns a list of (voltage_volts, start_idx, end_idx, matched_string).
pub fn extract_voltages_volts(text: &str) -> Vec<(f64, usize, usize, String)> {
    let mut results = Vec::new();

    // 1. Millivolts
    for cap in MV_PATTERN.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            if let Some(num_match) = cap.get(1) {
                let num_str = num_match.as_str().replace(',', ".");
                if let Ok(num) = num_str.parse::<f64>() {
                    results.push((num / 1000.0, m.start(), m.end(), m.as_str().trim().to_string()));
                }
            }
        }
    }

    // 2. EE RKM notation
    for cap in RKM_PATTERN.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            if let (Some(major), Some(minor)) = (cap.get(1), cap.get(2)) {
                let formatted = format!("{}.{}", major.as_str(), minor.as_str());
                if let Ok(val) = formatted.parse::<f64>() {
                    results.push((val, m.start(), m.end(), m.as_str().trim().to_string()));
                }
            }
        }
    }

    // 3. Standard Volts
    for cap in STD_PATTERN.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            let start = m.start();
            let end = m.end();
            if results.iter().any(|(_, s, e, _)| *s < end && *e > start) {
                continue;
            }
            if let Some(num_match) = cap.get(1) {
                let num_str = num_match.as_str().replace(',', ".");
                if let Ok(val) = num_str.parse::<f64>() {
                    results.push((val, start, end, m.as_str().trim().to_string()));
                }
            }
        }
    }

    results
}

pub struct HardwareSafetyValidator;

impl HardwareSafetyValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, text: &str, is_query: bool) -> SafetyReport {
        let mut violations = Vec::new();
        let mut critical_alerts = Vec::new();
        let mut warnings = Vec::new();

        let lower_text = text.to_lowercase();
        let upper_text = text.to_uppercase();

        // ----------------------------------------------------------------------
        // 1. Electrical Invariant: Absolute Max Voltage <= 3.35V
        // ----------------------------------------------------------------------
        for (val, start_idx, end_idx, matched_str) in extract_voltages_volts(text) {
            if val > MAX_SAFE_VOLTAGE_VOLTS {
                let mut start_ctx = start_idx.saturating_sub(60);
                for p in ['.', ';', '!', '\n'] {
                    if let Some(idx) = lower_text[start_ctx..start_idx].rfind(p) {
                        start_ctx = start_ctx.max(start_ctx + idx + 1);
                    }
                }

                let mut end_ctx = (end_idx + 60).min(text.len());
                for p in ['.', ';', '!', '\n'] {
                    if let Some(idx) = lower_text[end_idx..end_ctx].find(p) {
                        end_ctx = end_ctx.min(end_idx + idx);
                    }
                }

                let context_window = &lower_text[start_ctx..end_ctx];
                let has_warning = NEGATION_REGEX.is_match(context_window);

                if is_query || !has_warning {
                    violations.push(format!(
                        "CRITICAL ELECTRICAL HAZARD: Voltage {}V ({}) exceeds maximum safe tolerance of {}V on MN2DS0016 / VEP79138 digital board.",
                        val, matched_str, MAX_SAFE_VOLTAGE_VOLTS
                    ));
                    critical_alerts.push(format!(
                        "Overvoltage detected ({}V > {}V). Immediate risk of gate breakdown.",
                        val, MAX_SAFE_VOLTAGE_VOLTS
                    ));
                } else {
                    warnings.push(format!(
                        "High voltage {}V referenced in warning or prohibition context.",
                        val
                    ));
                }
            }
        }

        // ----------------------------------------------------------------------
        // 2. Bootloader Invariant: SA0..SA133 is strictly read-only
        // ----------------------------------------------------------------------
        let write_words = [
            "erase", "erasing", "wipe", "wiping", "write", "writing", "overwrite",
            "program", "programming", "flash", "flashing", "clear", "clearing", "destroy",
        ];
        let global_has_write = write_words.iter().any(|w| lower_text.contains(w));

        for m in HEX_PATTERN.find_iter(text) {
            let raw_hex = m.as_str();
            let addr = match u64::from_str_radix(raw_hex.trim_start_matches("0x").trim_start_matches("0X"), 16) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let mut start_ctx = m.start().saturating_sub(60);
            for p in ['.', ';', '!', '\n'] {
                if let Some(idx) = lower_text[start_ctx..m.start()].rfind(p) {
                    start_ctx = start_ctx.max(start_ctx + idx + 1);
                }
            }

            let mut end_ctx = (m.end() + 60).min(text.len());
            for p in ['.', ';', '!', '\n'] {
                if let Some(idx) = lower_text[m.end()..end_ctx].find(p) {
                    end_ctx = end_ctx.min(m.end() + idx);
                }
            }

            let context_window = &lower_text[start_ctx..end_ctx];

            // Ignore 1-byte data constants and bitmasks (e.g. 0xFF, 0xA0)
            if addr != 0 && addr < 0x1000 && raw_hex.len() <= 4 {
                continue;
            }

            let address_keywords = [
                "address", "offset", "flash", "starting at", "from 0x", "sector", "sa0", "sa133", "range",
            ];
            let is_address = raw_hex.len() >= 6
                || addr >= 0x1000
                || address_keywords.iter().any(|k| context_window.contains(k));

            if !is_address {
                continue;
            }

            let is_ipl_address = (addr <= (FORBIDDEN_FLASH_WRITE_END as u64))
                || (addr >= (FORBIDDEN_FLASH_BUS_START as u64) && addr <= (FORBIDDEN_FLASH_BUS_END as u64));

            if is_ipl_address {
                let has_local_write = write_words.iter().any(|w| context_window.contains(w)) || global_has_write;
                let has_warning = NEGATION_REGEX.is_match(context_window);

                if (is_query && has_local_write) || (!is_query && has_local_write && !has_warning) {
                    violations.push(format!(
                        "CRITICAL BOOTLOADER HAZARD: Flash address {} is within protected IPL range (SA0..SA133: 0x{:X}..0x{:X} / 0x{:X}..0x{:X}). Erasing or writing this region permanently bricks the Root of Trust.",
                        raw_hex.to_uppercase(),
                        FORBIDDEN_FLASH_WRITE_START,
                        FORBIDDEN_FLASH_WRITE_END,
                        FORBIDDEN_FLASH_BUS_START,
                        FORBIDDEN_FLASH_BUS_END
                    ));
                    critical_alerts.push(format!(
                        "Attempted write/erase at IPL address {}. Operation vetoed by air-gap shield.",
                        raw_hex.to_uppercase()
                    ));
                } else if has_warning {
                    warnings.push(format!(
                        "IPL address {} referenced in protective warning context.",
                        raw_hex.to_uppercase()
                    ));
                }
            }
        }

        // Direct explicit intent to erase SA0..SA133 / IPL
        for m in IPL_ERASE_PATTERN.find_iter(text) {
            let start_ctx = m.start().saturating_sub(60);
            let end_ctx = (m.end() + 60).min(text.len());
            let context_window = &lower_text[start_ctx..end_ctx];
            let has_warning = NEGATION_REGEX.is_match(context_window);

            if is_query || !has_warning {
                violations.push(format!(
                    "CRITICAL BOOTLOADER HAZARD: Direct instruction to modify IPL detected ('{}'). Erasing IPL sectors SA0..SA133 is strictly prohibited.",
                    m.as_str()
                ));
            }
        }

        // ----------------------------------------------------------------------
        // 3. Protocol Invariant: CN55001 is DWire32A, NOT Standard JTAG
        // ----------------------------------------------------------------------
        if upper_text.contains("CN55001") || upper_text.contains("CN5501") {
            let forbidden_pins = ["TDI", "TDO", "TMS", "TRST", "TCK"];
            let mentioned_jtag: Vec<&str> = forbidden_pins
                .iter()
                .filter(|&&pin| upper_text.contains(pin))
                .copied()
                .collect();

            if !mentioned_jtag.is_empty() {
                violations.push(format!(
                    "CRITICAL PROTOCOL HAZARD: CN55001 is a proprietary Matsushita DWire32A port (SDATA, SCLK, EXTRG, nRESET, GND). Standard JTAG pins ({}) do NOT exist on CN55001. Direct connection causes bus contention.",
                    mentioned_jtag.join(", ")
                ));
                critical_alerts.push(format!(
                    "Forbidden JTAG pin assignment on CN55001: {}",
                    mentioned_jtag.join(", ")
                ));
            }
        }

        // ----------------------------------------------------------------------
        // 4. Storage Invariant: MEIHDFS Hard Drive Structure Protection
        // ----------------------------------------------------------------------
        let pc_disk_utils = [
            "diskpart", "mkfs", "fdisk", "format c:", "format d:", "format e:",
            "format /fs:ntfs", "format /fs:fat32",
        ];
        for util in pc_disk_utils {
            if lower_text.contains(util) && !NEGATION_REGEX.is_match(&lower_text) {
                violations.push(format!(
                    "CRITICAL STORAGE HAZARD: PC utility '{}' destroys MEIHDFS 'SH' signature and firmware partitions.",
                    util
                ));
                critical_alerts.push(format!(
                    "Destructive disk formatting utility detected: {}",
                    util
                ));
            }
        }

        SafetyReport {
            is_safe: violations.is_empty(),
            violations,
            critical_alerts,
            warnings,
        }
    }
}

impl Default for HardwareSafetyValidator {
    fn default() -> Self {
        Self::new()
    }
}
