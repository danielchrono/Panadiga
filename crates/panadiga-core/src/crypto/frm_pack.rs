//! Panasonic `.FRM` Container Builders (`PANA_DVD.FRM`, `PANA__SD.FRM`, and `DIGADIAG.FRM`)
//!
//! Pure-Rust image packer matching `BOOT.BIN` (`FUN_440075aa`, `FUN_44007a12`,
//! `FUN_4400a733`, `FUN_4400a6d0`, and `FUN_4400a97c`).

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use super::codec::{pad_to_block, write_padded_ascii};
use super::extrhead::{ExtrheadCryptMode, pack_extrhead, parse_extrhead_header};
use super::feistel::feistel_encrypt_buffer;
use crate::constants::*;

/// Output container format for `.FRM` image generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrmContainerMode {
    Dvd,
    Sd,
}

/// Input module descriptor for packing `PANA_DVD.FRM` or `PANA__SD.FRM`.
#[derive(Debug, Clone)]
pub struct FrmPackModule {
    pub name: String,
    pub payload: Vec<u8>,
    pub version_u16: u16,
    pub hardware_family: String,
    pub model_prefix: u8,
    pub target_flag: u8,
    pub model_group: String,
    pub tail_meta: [u8; 6],
}

impl FrmPackModule {
    /// Creates a module specification populated with EH55 defaults (`M6930`, `M`, `NP-309`).
    pub fn new(name: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            payload,
            version_u16: 0x0001,
            hardware_family: "M6930".to_string(),
            model_prefix: b'M',
            target_flag: 0,
            model_group: "NP-309".to_string(),
            tail_meta: [0u8; 6],
        }
    }
}

/// Summary of a module slot written into a packed `.FRM` container.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackedFrmModuleSummary {
    pub index: usize,
    pub name: String,
    pub module_offset: u32,
    pub module_size: u32,
    pub fvu_checksum_u32: u32,
    pub list_sum_u16: u16,
}

/// Structured report emitted after packing a `PANA_DVD.FRM`, `PANA__SD.FRM`, or `DIGADIAG.FRM`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrmPackReport {
    pub mode: String,
    pub total_bytes: usize,
    pub toc_region_size: usize,
    pub modules: Vec<PackedFrmModuleSummary>,
}

/// Wraps an executable/diagnostic image into a plaintext `DIGADIAG.FRM` container padded to 128 KiB.
pub fn build_digadiag_frm(
    payload: &[u8],
    dst_addr: u32,
    tag: &str,
    build_name: &str,
    adler_step: u32,
    total_size: usize,
) -> Result<(Vec<u8>, FrmPackReport)> {
    let mut container = pack_extrhead(
        payload,
        tag,
        dst_addr,
        build_name,
        ExtrheadCryptMode::Plain,
        adler_step,
        1,
    )?;
    if container.len() > total_size {
        bail!(
            "DIGADIAG.FRM container ({} bytes) exceeds 0x{:X} byte limit",
            container.len(),
            total_size
        );
    }
    container.resize(total_size, 0);

    let hdr = parse_extrhead_header(&container[..0x40])?;
    let report = FrmPackReport {
        mode: "digadiag".to_string(),
        total_bytes: container.len(),
        toc_region_size: 0x40,
        modules: vec![PackedFrmModuleSummary {
            index: 0,
            name: hdr.tag,
            module_offset: 0x40,
            module_size: hdr.src_len,
            fvu_checksum_u32: hdr.header_checksum,
            list_sum_u16: 0,
        }],
    };
    Ok((container, report))
}

/// Packs modules into a `PANA_DVD.FRM` (Feistel + 48B TOC) or `PANA__SD.FRM` (plaintext + 64B TOC) image.
pub fn pack_frm(
    modules: &[FrmPackModule],
    mode: FrmContainerMode,
    toc_region_size: Option<usize>,
) -> Result<(Vec<u8>, FrmPackReport)> {
    if modules.is_empty() {
        bail!("At least one module is required to pack an .FRM container");
    }

    let is_dvd = mode == FrmContainerMode::Dvd;
    let stride = if is_dvd {
        PANA_FRM_TOC_ENTRY_SIZE
    } else {
        PANA_SD_TOC_ENTRY_SIZE
    };
    let region_size = toc_region_size.unwrap_or(if is_dvd {
        PANA_FRM_TOC_REGION_SIZE
    } else {
        PANA_SD_TOC_REGION_SIZE
    });

    if region_size < modules.len() * stride || (is_dvd && !region_size.is_multiple_of(8)) {
        bail!(
            "Invalid TOC region size 0x{:X} for {} modules",
            region_size,
            modules.len()
        );
    }
    if !is_dvd && modules.len() > PANA_SD_TOC_MAX_ENTRIES {
        bail!(
            "PANA__SD.FRM supports at most {} modules",
            PANA_SD_TOC_MAX_ENTRIES
        );
    }

    let mut image = vec![0u8; region_size];
    let mut summaries = Vec::with_capacity(modules.len());

    for (idx, module) in modules.iter().enumerate() {
        let mut mod_bytes = module.payload.clone();
        if is_dvd {
            pad_to_block(&mut mod_bytes);
        }
        let mod_offset = image.len() as u32;
        let mod_size = mod_bytes.len() as u32;
        let fvu_sum = mod_bytes
            .iter()
            .fold(0u32, |acc, &b| acc.wrapping_add(u32::from(b)));

        let entry = &mut image[idx * stride..(idx + 1) * stride];
        write_padded_ascii(&mut entry[0x00..0x0A], &module.name, 0);
        entry[0x0A..0x0C].copy_from_slice(&module.version_u16.to_le_bytes());
        entry[0x0C..0x10].copy_from_slice(&mod_offset.to_le_bytes());
        write_padded_ascii(&mut entry[0x10..0x18], &module.hardware_family, 0);
        entry[0x20..0x24].copy_from_slice(&mod_size.to_le_bytes());
        entry[0x24..0x28].copy_from_slice(&fvu_sum.to_le_bytes());

        if is_dvd {
            entry[0x18] = module.model_prefix;
            entry[0x19] = module.target_flag;
            write_padded_ascii(&mut entry[0x1A..0x20], &module.model_group, 0);
            entry[0x28..0x2E].copy_from_slice(&module.tail_meta);
        } else {
            entry[0x18] = module.target_flag;
            write_padded_ascii(&mut entry[0x30..0x3C], &module.model_group, 0);
        }

        let sum_off = stride - 2;
        let list_sum = entry[..sum_off]
            .iter()
            .fold(0u16, |acc, &b| acc.wrapping_add(u16::from(b)));
        entry[sum_off..stride].copy_from_slice(&list_sum.to_le_bytes());

        image.extend_from_slice(&mod_bytes);
        summaries.push(PackedFrmModuleSummary {
            index: idx,
            name: module.name.clone(),
            module_offset: mod_offset,
            module_size: mod_size,
            fvu_checksum_u32: fvu_sum,
            list_sum_u16: list_sum,
        });
    }

    if is_dvd {
        feistel_encrypt_buffer(
            &mut image,
            &PANA_FEISTEL_ROUND_KEY_32B,
            &PANA_FEISTEL_SBOX_256,
        );
    }

    let report = FrmPackReport {
        mode: if is_dvd { "dvd" } else { "sd" }.to_string(),
        total_bytes: image.len(),
        toc_region_size: region_size,
        modules: summaries,
    };
    Ok((image, report))
}
