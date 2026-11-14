//! Shared low-level binary & fixed-width ASCII codec helpers for Panasonic firmware containers.

use crate::constants::CIPHER_BLOCK_SIZE;

pub(crate) fn read_u16_le(slice: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([slice[off], slice[off + 1]])
}

pub(crate) fn read_u32_le(slice: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([slice[off], slice[off + 1], slice[off + 2], slice[off + 3]])
}

pub(crate) fn extract_c_ascii(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

pub(crate) fn write_padded_ascii(dst: &mut [u8], text: &str, pad: u8) {
    dst.fill(pad);
    let src = text.as_bytes();
    let count = src.len().min(dst.len());
    dst[..count].copy_from_slice(&src[..count]);
}

pub(crate) fn pad_to_block(buf: &mut Vec<u8>) {
    let rem = buf.len() % CIPHER_BLOCK_SIZE;
    if rem != 0 {
        buf.resize(buf.len() + (CIPHER_BLOCK_SIZE - rem), 0);
    }
}
