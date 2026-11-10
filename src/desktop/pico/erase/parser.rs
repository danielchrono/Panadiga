//! Panasonic DMR-EH55 DIGA Suite - DWire32A Dump & Address Parser
//!
//! Single Responsibility: Pure parsing of DWire32A memory dump streams and
//! auto-detection of top-boot 64KB EEMUL NVRAM base addresses from Pico CDC output.

pub(crate) fn push_hex_word_le(token: &str, out: &mut Vec<u8>) {
    let clean_tok = token
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");
    if clean_tok.len() == 8 && clean_tok.chars().all(|c| c.is_ascii_hexdigit()) {
        if let Ok(word) = u32::from_str_radix(clean_tok, 16) {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
}

/// Pure parser extracting the auto-detected Top-Boot 64KB EEMUL NVRAM base address
/// (`0x801F0000`, `0x803F0000`, `0x807F0000`, or `0x80FF0000`) from the Pico `HANDSHAKE`
/// or `AUTODETECT` output string. Defaults to `"0x807F0000"` (8 MB).
pub fn parse_autodetected_nvram_base(handshake_output: &str) -> &'static str {
    if handshake_output.contains("0x801F0000") || handshake_output.contains("2 MB NOR Flash") {
        "0x801F0000"
    } else if handshake_output.contains("0x803F0000") || handshake_output.contains("4 MB NOR Flash")
    {
        "0x803F0000"
    } else if handshake_output.contains("0x80FE0000")
        || handshake_output.contains("0x80FF0000")
        || handshake_output.contains("16 MB NOR Flash")
    {
        if handshake_output.contains("0x80FE0000") {
            "0x80FE0000"
        } else {
            "0x80FF0000"
        }
    } else {
        "0x807F0000"
    }
}

/// Pure helper converting Pico `DUMP` ASCII output (`807F0000: AABBCCDD 11223344 ...`
/// or `[0x807F0000] = 0xAABBCCDD`) into true Little-Endian binary bytes (`u32::to_le_bytes`).
pub fn parse_dwire_dump_hex_to_bytes(ascii_dump: &str) -> Vec<u8> {
    let mut binary = Vec::new();
    for line in ascii_dump.lines() {
        let trimmed = line.trim();
        if let Some((addr_part, values_part)) = trimmed.split_once(':') {
            let addr_clean = addr_part.trim();
            if addr_clean.len() == 8 && addr_clean.chars().all(|c| c.is_ascii_hexdigit()) {
                for token in values_part.split_whitespace() {
                    push_hex_word_le(token, &mut binary);
                }
            }
        } else if let Some((_, rhs)) = trimmed.split_once('=') {
            push_hex_word_le(rhs, &mut binary);
        }
    }
    binary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dwire_dump_hex_to_bytes_little_endian() {
        let sample = "\
[DUMP] Reading 4 words (16 bytes) from 0x807F0000...

807F0000: 11223344 AABBCCDD FFFFFFFF 01020304 

DUMP: OK";
        let bytes = parse_dwire_dump_hex_to_bytes(sample);
        assert_eq!(bytes.len(), 16);
        assert_eq!(&bytes[0..4], &[0x44, 0x33, 0x22, 0x11]);
        assert_eq!(&bytes[4..8], &[0xDD, 0xCC, 0xBB, 0xAA]);
        assert_eq!(&bytes[8..12], &[0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(&bytes[12..16], &[0x04, 0x03, 0x02, 0x01]);
    }

    #[test]
    fn test_parse_autodetected_nvram_base() {
        assert_eq!(
            parse_autodetected_nvram_base("2 MB NOR Flash detected (EEMUL NVRAM @ 0x801F0000)"),
            "0x801F0000"
        );
        assert_eq!(
            parse_autodetected_nvram_base("4 MB NOR Flash detected (EEMUL NVRAM @ 0x803F0000)"),
            "0x803F0000"
        );
        assert_eq!(
            parse_autodetected_nvram_base("8 MB NOR Flash detected (EEMUL NVRAM @ 0x807F0000)"),
            "0x807F0000"
        );
        assert_eq!(
            parse_autodetected_nvram_base("16 MB NOR Flash detected (EEMUL NVRAM @ 0x80FE0000)"),
            "0x80FE0000"
        );
        assert_eq!(
            parse_autodetected_nvram_base("16 MB NOR Flash detected (EEMUL NVRAM @ 0x80FF0000)"),
            "0x80FF0000"
        );
    }
}
