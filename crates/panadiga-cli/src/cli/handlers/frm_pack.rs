//! CLI handlers for packing (`PANA_DVD.FRM`, `PANA__SD.FRM`, `DIGADIAG.FRM`) and unpacking `EXTRHEAD`.

use super::super::args::crypto::{FrmPackArgs, FrmUnpackArgs};
use anyhow::{Context, Result, bail};
use panadiga_core::constants::{PANA_DIGADIAG_FRM_SIZE, PANA_EXTRHEAD_DEFAULT_ADLER_STEP};
use panadiga_core::crypto::{
    ExtrheadCryptMode, FrmContainerMode, FrmPackModule, FrmPackReport, build_digadiag_frm,
    pack_frm, unpack_extrhead,
};
use serde::Serialize;
use std::path::Path;

fn parse_u32_radix(raw: &str) -> Result<u32> {
    let trimmed = raw.trim();
    if let Some(hex_body) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        return u32::from_str_radix(hex_body, 16)
            .with_context(|| format!("Invalid hex u32 literal: {raw}"));
    }
    trimmed
        .parse::<u32>()
        .with_context(|| format!("Invalid decimal u32 literal: {raw}"))
}

fn write_json_report<T: Serialize>(json_path: Option<&Path>, value: &T) -> Result<()> {
    if let Some(path) = json_path {
        let serialized = serde_json::to_string_pretty(value)?;
        std::fs::write(path, serialized)
            .with_context(|| format!("Writing JSON report to {}", path.display()))?;
    }
    Ok(())
}

fn parse_pack_modules(args: &FrmPackArgs) -> Result<Vec<FrmPackModule>> {
    if args.modules.is_empty() {
        bail!(
            "At least one --module NAME:PATH argument is required for --mode {}",
            args.mode
        );
    }
    let ver_u16 = u16::try_from(parse_u32_radix(&args.version_u16)?)
        .context("--version-u16 must fit within 16 bits")?;
    let mut modules = Vec::with_capacity(args.modules.len());
    for spec in &args.modules {
        let Some((name, file_part)) = spec.split_once(':') else {
            bail!("Invalid --module specification '{spec}': expected NAME:PATH");
        };
        let payload = std::fs::read(file_part)
            .with_context(|| format!("Reading module '{name}' from {file_part}"))?;
        let mut m = FrmPackModule::new(name, payload);
        m.version_u16 = ver_u16;
        m.hardware_family = args.hw.clone();
        m.model_group = args.model.clone();
        m.target_flag = args.target_flag;
        modules.push(m);
    }
    Ok(modules)
}

pub fn handle_frm_pack(args: &FrmPackArgs) -> Result<()> {
    let (bytes, report): (Vec<u8>, FrmPackReport) = match args.mode.to_ascii_lowercase().as_str() {
        "digadiag" => {
            let bin_path = args
                .input
                .as_deref()
                .context("--input <BIN_FILE> is required for --mode digadiag")?;
            let payload = std::fs::read(bin_path)
                .with_context(|| format!("Reading {}", bin_path.display()))?;
            let dst_addr = parse_u32_radix(&args.dst_addr)?;
            build_digadiag_frm(
                &payload,
                dst_addr,
                &args.tag,
                &args.build,
                PANA_EXTRHEAD_DEFAULT_ADLER_STEP as u32,
                PANA_DIGADIAG_FRM_SIZE,
            )?
        }
        "dvd" | "sd" => {
            let modules = parse_pack_modules(args)?;
            let container_mode = if args.mode.eq_ignore_ascii_case("dvd") {
                FrmContainerMode::Dvd
            } else {
                FrmContainerMode::Sd
            };
            pack_frm(&modules, container_mode, None)?
        }
        other => bail!("Unsupported FRM pack mode '{other}': expected dvd, sd, or digadiag"),
    };

    std::fs::write(&args.out, &bytes).with_context(|| format!("Writing {}", args.out.display()))?;
    write_json_report(args.json.as_deref(), &report)?;
    println!(
        "Packed {} ({}) -> {} bytes ({} module(s))",
        args.out.display(),
        report.mode,
        report.total_bytes,
        report.modules.len()
    );
    Ok(())
}

pub fn handle_frm_unpack(args: &FrmUnpackArgs) -> Result<()> {
    let crypt_mode = match args.mode.to_ascii_lowercase().as_str() {
        "auto" => ExtrheadCryptMode::Auto,
        "main" => ExtrheadCryptMode::Main,
        "drv" => ExtrheadCryptMode::Drv,
        "plain" => ExtrheadCryptMode::Plain,
        other => bail!("Unsupported unpack mode '{other}': expected auto, main, drv, or plain"),
    };
    let raw =
        std::fs::read(&args.file).with_context(|| format!("Reading {}", args.file.display()))?;
    let unpacked = unpack_extrhead(&raw, crypt_mode)?;
    std::fs::write(&args.out, &unpacked.decompressed)
        .with_context(|| format!("Writing {}", args.out.display()))?;
    write_json_report(args.json.as_deref(), &unpacked.header)?;
    println!(
        "Unpacked {} -> {} ({} bytes, tag='{}', dst=0x{:08X}, checksum_valid={}, footer={})",
        args.file.display(),
        args.out.display(),
        unpacked.decompressed.len(),
        unpacked.header.tag,
        unpacked.header.dst_addr,
        unpacked.checksum_valid,
        unpacked.footer_present
    );
    Ok(())
}
