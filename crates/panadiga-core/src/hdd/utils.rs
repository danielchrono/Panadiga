use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::ops::Deref;
use std::path::Path;

use anyhow::{Context, Result};
use memmap2::Mmap;

use crate::nvram::resolve_input_path;

/// Smart abstraction representing an image buffer in memory.
/// Transparently encapsulates either a zero-copy memory mapped file or a dynamically
/// decompressed in-memory GZIP payload, exposing a unified byte slice `&[u8]`.
pub enum ImageBuffer {
    Mmap((File, Mmap)),
    Ram(Vec<u8>),
}

impl Deref for ImageBuffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match self {
            ImageBuffer::Mmap((_, mmap)) => mmap,
            ImageBuffer::Ram(vec) => vec,
        }
    }
}

const ENTROPY_PARALLEL_CHUNK_BYTES: usize = 256 * 1024;

/// Shannon entropy calculator (0.0000 to 8.0000 bits/byte), sharded in parallel for large slices.
pub fn calculate_entropy(data: &[u8]) -> f64 {
    use rayon::prelude::*;

    if data.is_empty() {
        return 0.0;
    }
    let counts = if data.len() >= ENTROPY_PARALLEL_CHUNK_BYTES * 2 {
        data.par_chunks(ENTROPY_PARALLEL_CHUNK_BYTES)
            .fold(
                || [0usize; 256],
                |mut local, chunk| {
                    for &b in chunk {
                        local[b as usize] += 1;
                    }
                    local
                },
            )
            .reduce(
                || [0usize; 256],
                |mut acc, local| {
                    for (dst, src) in acc.iter_mut().zip(local) {
                        *dst += src;
                    }
                    acc
                },
            )
    } else {
        let mut local = [0usize; 256];
        for &b in data {
            local[b as usize] += 1;
        }
        local
    };
    let total = data.len() as f64;
    let mut entropy = 0.0;
    for c in counts {
        if c > 0 {
            let p = c as f64 / total;
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// Opens a disk image buffer, transparently supporting raw images via zero-copy mmap
/// or compressed images (.gz or magic 0x1F 0x8B) via streaming in-memory GZIP decompression.
pub fn open_image_buffer(path: &Path) -> Result<ImageBuffer> {
    let resolved = resolve_input_path(path)?;
    let mut file = File::open(&resolved)
        .with_context(|| format!("Failed opening image file: {}", resolved.display()))?;
    let file_len = file.metadata()?.len();
    if file_len == 0 {
        anyhow::bail!("Cannot load empty file (0 bytes): {}", resolved.display());
    }

    // Check for GZIP magic bytes (0x1F 0x8B)
    let mut magic = [0u8; 2];
    if file.read_exact(&mut magic).is_ok() && magic == [0x1f, 0x8b] {
        file.seek(SeekFrom::Start(0))?;
        let mut decoder = flate2::read::GzDecoder::new(file);
        let mut buffer = Vec::new();
        decoder
            .read_to_end(&mut buffer)
            .with_context(|| format!("Failed decompressing GZIP image: {}", resolved.display()))?;
        return Ok(ImageBuffer::Ram(buffer));
    }

    // Default to high-performance zero-copy mmap for raw binaries
    file.seek(SeekFrom::Start(0))?;
    let mmap = unsafe { Mmap::map(&file) }
        .with_context(|| format!("Failed creating mmap on: {}", resolved.display()))?;
    Ok(ImageBuffer::Mmap((file, mmap)))
}

/// Opens a disk image with zero-copy memory mapping.
pub fn open_mmap(path: &Path) -> Result<(File, Mmap)> {
    let resolved = resolve_input_path(path)?;
    let file = File::open(&resolved)
        .with_context(|| format!("Failed opening HDD image: {}", resolved.display()))?;
    let file_len = file.metadata()?.len();
    if file_len == 0 {
        anyhow::bail!(
            "Cannot memory map empty file (0 bytes): {}",
            resolved.display()
        );
    }
    let mmap = unsafe { Mmap::map(&file) }
        .with_context(|| format!("Failed creating mmap on: {}", resolved.display()))?;
    Ok((file, mmap))
}

/// Helper function to serialize any report struct to formatted JSON and write to disk.
pub fn export_report_to_json<T: serde::Serialize>(
    report: &T,
    json_path: &Path,
    report_desc: &str,
) -> Result<()> {
    let json_data = serde_json::to_string_pretty(report)
        .with_context(|| format!("Failed serializing {report_desc} to JSON"))?;
    fs::write(json_path, json_data)
        .with_context(|| format!("Failed writing JSON report to {}", json_path.display()))?;
    log_println!(
        "\n[+] Structured JSON report exported: {}",
        json_path.display()
    );
    Ok(())
}
