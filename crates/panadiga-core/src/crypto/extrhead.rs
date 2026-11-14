//! Panasonic `EXTRHEAD`/`EXTRFOOT` zlib Codec & Partial Feistel Wrapper
//!
//! Pure-Rust implementation matching `BOOT.BIN` (`FUN_4400b5e1`, `FUN_4400b61a`,
//! `FUN_4400b6c6`, and `FUN_4401e5a3`).

use anyhow::{Context, Result, bail};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use super::codec::{extract_c_ascii, pad_to_block, read_u16_le, read_u32_le, write_padded_ascii};
use super::feistel::{adler32_panasonic_strided, feistel_decrypt_buffer, feistel_encrypt_buffer};
use crate::constants::*;

/// Partial encryption profile for an `EXTRHEAD`/`EXTRFOOT` container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtrheadCryptMode {
    Auto,
    Main,
    Drv,
    Plain,
}

/// Parsed 64-byte (`0x40`) Panasonic `EXTRHEAD` header block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtrheadHeader {
    pub tag: String,
    pub flag: u16,
    pub algo: u16,
    pub dst_len: u32,
    pub dst_addr: u32,
    pub src_len: u32,
    pub footer_offset: u32,
    pub header_checksum: u32,
    pub adler_step: u32,
    pub build_name: String,
}

/// Result of decrypting, decompressing, and validating an `EXTRHEAD`/`EXTRFOOT` payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpackedExtrhead {
    pub header: ExtrheadHeader,
    pub decompressed: Vec<u8>,
    pub calculated_checksum: u32,
    pub checksum_valid: bool,
    pub footer_present: bool,
}

fn apply_partial_crypt(buf: &mut [u8], mode: ExtrheadCryptMode, aligned_end: usize, encrypt: bool) {
    let transform = |slice: &mut [u8]| {
        if encrypt {
            feistel_encrypt_buffer(slice, &PANA_FEISTEL_ROUND_KEY_32B, &PANA_FEISTEL_SBOX_256);
        } else {
            feistel_decrypt_buffer(slice, &PANA_FEISTEL_ROUND_KEY_32B, &PANA_FEISTEL_SBOX_256);
        }
    };
    let end = aligned_end.min(buf.len()) & !7;
    if end == 0 {
        return;
    }
    match mode {
        ExtrheadCryptMode::Plain | ExtrheadCryptMode::Auto => {}
        ExtrheadCryptMode::Drv => {
            let span = PANA_EXTRHEAD_DRV_CRYPT_SPAN.min(end) & !7;
            transform(&mut buf[..span]);
        }
        ExtrheadCryptMode::Main => {
            let span = PANA_EXTRHEAD_MAIN_CRYPT_SPAN;
            if end <= 2 * span {
                transform(&mut buf[..end]);
            } else {
                transform(&mut buf[..span]);
                transform(&mut buf[(end - span)..end]);
            }
        }
    }
}

fn resolve_crypt_mode(
    mode: ExtrheadCryptMode,
    tag: &str,
    is_plain_header: bool,
) -> ExtrheadCryptMode {
    if mode != ExtrheadCryptMode::Auto {
        return mode;
    }
    if is_plain_header {
        return ExtrheadCryptMode::Plain;
    }
    match tag.trim() {
        "DRV" => ExtrheadCryptMode::Drv,
        "MAIN" => ExtrheadCryptMode::Main,
        _ => ExtrheadCryptMode::Plain,
    }
}

/// Parses a decrypted 64-byte (`0x40`) `EXTRHEAD` header block.
pub fn parse_extrhead_header(header_bytes: &[u8]) -> Result<ExtrheadHeader> {
    if header_bytes.len() < 0x40 || &header_bytes[..8] != b"EXTRHEAD" {
        bail!("Invalid EXTRHEAD container header");
    }
    Ok(ExtrheadHeader {
        tag: String::from_utf8_lossy(&header_bytes[0x08..0x0C]).into_owned(),
        flag: read_u16_le(header_bytes, 0x0C),
        algo: read_u16_le(header_bytes, 0x0E),
        dst_len: read_u32_le(header_bytes, 0x10),
        dst_addr: read_u32_le(header_bytes, 0x14),
        src_len: read_u32_le(header_bytes, 0x18),
        footer_offset: read_u32_le(header_bytes, 0x20),
        header_checksum: read_u32_le(header_bytes, 0x28),
        adler_step: read_u32_le(header_bytes, 0x2C),
        build_name: extract_c_ascii(&header_bytes[0x30..0x40]),
    })
}

/// Compresses and wraps `payload` into a Panasonic `EXTRHEAD`/`EXTRFOOT` container.
pub fn pack_extrhead(
    payload: &[u8],
    tag: &str,
    dst_addr: u32,
    build_name: &str,
    mode: ExtrheadCryptMode,
    adler_step: u32,
    flag: u16,
) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(payload)
        .context("Compressing EXTRHEAD payload")?;
    let compressed = encoder
        .finish()
        .context("Finalizing zlib EXTRHEAD stream")?;

    let footer_offset = (0x40 + compressed.len()) as u32;
    let checksum = adler32_panasonic_strided(payload, adler_step as usize);

    let mut raw = Vec::with_capacity(0x40 + compressed.len() + 16);
    raw.extend_from_slice(b"EXTRHEAD");
    let mut tag_buf = [b' '; 4];
    write_padded_ascii(&mut tag_buf, tag, b' ');
    raw.extend_from_slice(&tag_buf);
    raw.extend_from_slice(&flag.to_le_bytes());
    raw.extend_from_slice(&0u16.to_le_bytes());
    raw.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    raw.extend_from_slice(&dst_addr.to_le_bytes());
    raw.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    raw.extend_from_slice(&0u32.to_le_bytes());
    raw.extend_from_slice(&footer_offset.to_le_bytes());
    raw.extend_from_slice(&dst_addr.to_le_bytes());
    raw.extend_from_slice(&checksum.to_le_bytes());
    raw.extend_from_slice(&adler_step.to_le_bytes());
    let mut build_buf = [0u8; 16];
    write_padded_ascii(&mut build_buf, build_name, 0);
    raw.extend_from_slice(&build_buf);
    raw.extend_from_slice(&compressed);
    raw.extend_from_slice(b"EXTRFOOT");
    pad_to_block(&mut raw);

    let eff_mode = resolve_crypt_mode(mode, tag, mode == ExtrheadCryptMode::Plain);
    let len = raw.len();
    apply_partial_crypt(&mut raw, eff_mode, len, true);
    Ok(raw)
}

/// Unpacks an `EXTRHEAD`/`EXTRFOOT` container using Panasonic partial Feistel encryption.
pub fn unpack_extrhead(raw_container: &[u8], mode: ExtrheadCryptMode) -> Result<UnpackedExtrhead> {
    if raw_container.len() < 0x40 {
        bail!("Container shorter than 64-byte EXTRHEAD header");
    }

    let is_plain_header = mode == ExtrheadCryptMode::Plain || &raw_container[..8] == b"EXTRHEAD";
    let mut hdr_blk = [0u8; 0x40];
    hdr_blk.copy_from_slice(&raw_container[..0x40]);
    if !is_plain_header {
        feistel_decrypt_buffer(
            &mut hdr_blk,
            &PANA_FEISTEL_ROUND_KEY_32B,
            &PANA_FEISTEL_SBOX_256,
        );
    }

    let header = parse_extrhead_header(&hdr_blk)?;
    if header.algo != 0 {
        bail!("Unsupported EXTRHEAD compression algo: {}", header.algo);
    }

    let eff_mode = resolve_crypt_mode(mode, &header.tag, is_plain_header);
    let footer_span = if header.footer_offset >= 0x40 {
        header.footer_offset as usize + 8
    } else {
        0x40 + header.src_len as usize + 8
    };
    let aligned_end = (raw_container.len() & !7).min((footer_span + 7) & !7);

    let mut decrypted = raw_container.to_vec();
    apply_partial_crypt(&mut decrypted, eff_mode, aligned_end, false);

    let payload_end = 0x40usize + header.src_len as usize;
    if payload_end > decrypted.len() {
        bail!(
            "EXTRHEAD container truncated: expected {} bytes, got {}",
            payload_end,
            decrypted.len()
        );
    }

    let mut decoder = ZlibDecoder::new(&decrypted[0x40..payload_end]);
    let mut decompressed = Vec::with_capacity(header.dst_len as usize);
    decoder
        .read_to_end(&mut decompressed)
        .context("Decompressing EXTRHEAD zlib payload")?;

    let stride = header.adler_step.max(1) as usize;
    let calculated_checksum = adler32_panasonic_strided(&decompressed, stride);
    let foff = header.footer_offset as usize;
    let footer_present =
        foff >= 0x40 && foff + 8 <= decrypted.len() && &decrypted[foff..foff + 8] == b"EXTRFOOT";

    Ok(UnpackedExtrhead {
        checksum_valid: calculated_checksum == header.header_checksum,
        header,
        decompressed,
        calculated_checksum,
        footer_present,
    })
}
