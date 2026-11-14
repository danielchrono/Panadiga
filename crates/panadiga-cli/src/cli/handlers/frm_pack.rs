//! CLI handlers for packing (`PANA_DVD.FRM`, `PANA__SD.FRM`, `DIGADIAG.FRM`) and unpacking `EXTRHEAD`.

use super::super::args::crypto::{FrmPackArgs, FrmUnpackArgs};
use anyhow::{Context, Result, bail};
use panadiga_core::constants::{PANA_DIGADIAG_FRM_SIZE, PANA_EXTRHEAD_DEFAULT_ADLER_STEP};
use panadiga_core::crypto::{
    ExtrheadCryptMode, FrmContainerMode, FrmPackModule, FrmPackReport, build_digadiag_frm,
    pack_frm, parse_u32_radix, unpack_extrhead,
};
use serde::Serialize;
use std::path::Path;

fn write_json_report<T: Serialize>(json_path: Option<&Path>, value: &T) -> Result<()> {
    if let Some(path) = json_path {
        let serialized = serde_json::to_string_pretty(value)?;
        std::fs::write(path, serialized)
            .with_context(|| format!("Writing JSON report to {}", path.display()))?;
    }
    Ok(())
}

fn split_name_and_rest(spec: &str) -> Option<(&str, &str)> {
    match (spec.find(':'), spec.find('=')) {
        (Some(c), Some(e)) => {
            if c < e {
                Some((&spec[..c], &spec[c + 1..]))
            } else {
                Some((&spec[..e], &spec[e + 1..]))
            }
        }
        (Some(c), None) => Some((&spec[..c], &spec[c + 1..])),
        (None, Some(e)) => Some((&spec[..e], &spec[e + 1..])),
        (None, None) => None,
    }
}

fn parse_module_spec(
    spec: &str,
    default_ver: u16,
    default_flag: u8,
) -> Result<(&str, &str, u16, u8)> {
    let Some((name, rest)) = split_name_and_rest(spec) else {
        bail!("Invalid --module specification '{spec}': expected NAME:PATH[:VER:FLAG]");
    };
    if let Some((path_ver, flag_tok)) = rest.rsplit_once(':')
        && let Some((path_part, ver_tok)) = path_ver.rsplit_once(':')
        && !path_part.is_empty()
        && !ver_tok.contains(['/', '\\'])
        && !flag_tok.contains(['/', '\\'])
        && let Ok(ver_u32) = parse_u32_radix(ver_tok)
        && let Ok(ver_u16) = u16::try_from(ver_u32)
        && let Ok(flag_u32) = parse_u32_radix(flag_tok)
        && let Ok(flag_u8) = u8::try_from(flag_u32)
    {
        return Ok((name, path_part, ver_u16, flag_u8));
    }
    Ok((name, rest, default_ver, default_flag))
}

fn parse_pack_modules(args: &FrmPackArgs) -> Result<Vec<FrmPackModule>> {
    if args.modules.is_empty() {
        bail!(
            "At least one --module NAME:PATH[:VER:FLAG] argument is required for --mode {}",
            args.mode
        );
    }
    let default_ver = u16::try_from(parse_u32_radix(&args.version_u16)?)
        .context("--version-u16 must fit within 16 bits")?;
    let mut modules = Vec::with_capacity(args.modules.len());
    for spec in &args.modules {
        let (name, file_part, ver_u16, target_flag) =
            parse_module_spec(spec, default_ver, args.target_flag)?;
        let payload = std::fs::read(file_part)
            .with_context(|| format!("Reading module '{name}' from {file_part}"))?;
        let mut m = FrmPackModule::new(name, payload);
        m.version_u16 = ver_u16;
        m.hardware_family = args.hw.clone();
        m.model_group = args.model.clone();
        m.target_flag = target_flag;
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
        "dvd" | "sd" | "sd48" => {
            let modules = parse_pack_modules(args)?;
            let container_mode = if args.mode.eq_ignore_ascii_case("dvd") {
                FrmContainerMode::Dvd
            } else if args.mode.eq_ignore_ascii_case("sd48") {
                FrmContainerMode::Sd48
            } else {
                FrmContainerMode::Sd
            };
            pack_frm(&modules, container_mode, None)?
        }
        other => bail!("Unsupported FRM pack mode '{other}': expected dvd, sd, sd48, or digadiag"),
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
    let crypt_mode: ExtrheadCryptMode = args.mode.parse()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_module_spec_variants() {
        let (n, p, v, f) = parse_module_spec("DRV S211:firmware/drv.bin", 0x0001, 0).unwrap();
        assert_eq!((n, p, v, f), ("DRV S211", "firmware/drv.bin", 0x0001, 0));

        let (n, p, v, f) =
            parse_module_spec("PROG061 =C:\\fw\\prog061_dec.bin:0x1202:1", 0x0001, 0).unwrap();
        assert_eq!(
            (n, p, v, f),
            ("PROG061 ", "C:\\fw\\prog061_dec.bin", 0x1202, 1)
        );

        let (n, p, v, f) =
            parse_module_spec("MAIN3060NP:C:\\fw\\main3060np.bin", 0x96AA, 0).unwrap();
        assert_eq!(
            (n, p, v, f),
            ("MAIN3060NP", "C:\\fw\\main3060np.bin", 0x96AA, 0)
        );

        assert!(parse_module_spec("NO_DELIMITER", 0x0001, 0).is_err());
    }
}
