//! Panasonic DMR-EH55 DIGA Suite - Pico Dump & Address Parser
//!
//! Single Responsibility: Re-exports canonical Pico memory stream parsers from `panadiga-core`.

pub use panadiga_core::pico::cdc::{parse_autodetected_nvram_base, parse_pico_dump_hex_to_bytes};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pico_dump_hex_to_bytes_little_endian() {
        let sample = "\
[DUMP] Reading 4 words (16 bytes) from 0x807F0000...

807F0000: 11223344 AABBCCDD FFFFFFFF 01020304 

DUMP: OK";
        let bytes = parse_pico_dump_hex_to_bytes(sample);
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
