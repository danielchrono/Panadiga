//! Tauri IPC Commands for Panasonic `.FRM` Packing and `EXTRHEAD`/`EXTRFOOT` Unpacking.

use panadiga_core::constants::{
    PANA_DIGADIAG_FRM_SIZE, PANA_EXTRHEAD_DEFAULT_ADLER_STEP, find_workspace_subdir,
    resolve_workspace_root,
};
use panadiga_core::crypto::{
    ExtrheadCryptMode, ExtrheadHeader, FrmContainerMode, FrmPackModule, FrmPackReport,
    build_digadiag_frm, pack_frm, parse_u32_radix, unpack_extrhead,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Module input item passed from the desktop UI for `dvd` / `sd` container packing.
#[derive(Debug, Clone, Deserialize)]
pub struct FrmPackModuleInputDto {
    pub name: String,
    pub path: String,
}

/// Parameters accepted by `pack_frm_command`.
#[derive(Debug, Clone, Deserialize)]
pub struct PackFrmRequestDto {
    pub mode: String,
    pub out_path: String,
    #[serde(default)]
    pub modules: Vec<FrmPackModuleInputDto>,
    pub input_path: Option<String>,
    pub dst_addr: Option<String>,
    pub tag: Option<String>,
    pub build_name: Option<String>,
    pub hw_family: Option<String>,
    pub model_group: Option<String>,
    pub version_u16: Option<String>,
    pub target_flag: Option<u8>,
}

/// Result returned by `unpack_extrhead_command` with verified header and checksum details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrheadUnpackReportDto {
    pub file_path: String,
    pub header: ExtrheadHeader,
    pub decompressed_bytes: usize,
    pub calculated_checksum_hex: String,
    pub header_checksum_hex: String,
    pub checksum_valid: bool,
    pub footer_present: bool,
    pub written_output_path: Option<String>,
}

fn resolve_read_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    let direct = PathBuf::from(trimmed);
    if direct.exists() || direct.is_absolute() {
        return direct;
    }
    let ws_root = resolve_workspace_root();
    let ws_rel = ws_root.join(trimmed);
    if ws_rel.exists() {
        return ws_rel;
    }
    let fw_rel = ws_root.join("firmware_and_images").join(trimmed);
    if fw_rel.exists() {
        return fw_rel;
    }
    ws_rel
}

fn resolve_write_path(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Caminho de saída obrigatório.".to_string());
    }
    let path = PathBuf::from(trimmed);
    let resolved = if path.is_absolute() {
        path
    } else {
        resolve_workspace_root().join(path)
    };
    if let Some(parent) = resolved.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Falha criando diretório '{}': {}", parent.display(), e))?;
    }
    Ok(resolved)
}

fn collect_frm_files(dir: &Path, root: &Path, out: &mut Vec<String>, depth: usize) {
    if depth > 2 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if name != "bkp" {
                collect_frm_files(&p, root, out, depth + 1);
            }
            continue;
        }
        let is_frm = p
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("frm"))
            .unwrap_or(false);
        if is_frm {
            let rel = p.strip_prefix(root).unwrap_or(&p);
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

/// Lists `.FRM` containers available under workspace `firmware_and_images`.
#[tauri::command]
pub fn list_frm_containers_command() -> Result<Vec<String>, String> {
    let ws_root = resolve_workspace_root();
    let Some(fw_dir) = find_workspace_subdir("firmware_and_images") else {
        return Ok(Vec::new());
    };
    let mut found = Vec::new();
    collect_frm_files(&fw_dir, &ws_root, &mut found, 0);
    found.sort();
    found.dedup();
    Ok(found)
}

/// Packs modules or raw diagnostic binaries into `PANA_DVD.FRM`, `PANA__SD.FRM`, or `DIGADIAG.FRM`.
#[tauri::command]
pub fn pack_frm_command(request: PackFrmRequestDto) -> Result<FrmPackReport, String> {
    let mode_norm = request.mode.trim().to_ascii_lowercase();
    let out_path = resolve_write_path(&request.out_path)?;

    let (bytes, report) = match mode_norm.as_str() {
        "digadiag" => {
            let in_raw = request
                .input_path
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "Modo 'digadiag' exige arquivo binário de entrada.".to_string())?;
            let in_path = resolve_read_path(in_raw);
            let payload = std::fs::read(&in_path)
                .map_err(|e| format!("Falha lendo '{}': {}", in_path.display(), e))?;
            let dst_addr = parse_u32_radix(request.dst_addr.as_deref().unwrap_or("0x50400000"))
                .map_err(|e| format!("Endereço destino inválido: {e}"))?;
            let tag = request.tag.as_deref().unwrap_or("FVU ");
            let build = request.build_name.as_deref().unwrap_or("DIAG01");
            build_digadiag_frm(
                &payload,
                dst_addr,
                tag,
                build,
                PANA_EXTRHEAD_DEFAULT_ADLER_STEP as u32,
                PANA_DIGADIAG_FRM_SIZE,
            )
            .map_err(|e| format!("Falha empacotando DIGADIAG.FRM: {e}"))?
        }
        "sd" | "dvd" => {
            if request.modules.is_empty() {
                return Err(format!(
                    "Modo '{mode_norm}' exige ao menos um módulo (nome + caminho)."
                ));
            }
            let ver_raw = request.version_u16.as_deref().unwrap_or("0x0001");
            let ver_u32 = parse_u32_radix(ver_raw).map_err(|e| e.to_string())?;
            let ver_u16 = u16::try_from(ver_u32)
                .map_err(|_| "version_u16 deve caber em 16 bits.".to_string())?;
            let hw = request.hw_family.as_deref().unwrap_or("M6930");
            let model = request.model_group.as_deref().unwrap_or("NP-309");
            let flag = request.target_flag.unwrap_or(0);

            let mut parsed_modules = Vec::with_capacity(request.modules.len());
            for item in &request.modules {
                let name = item.name.trim();
                let path_str = item.path.trim();
                if name.is_empty() || path_str.is_empty() {
                    return Err("Cada módulo requer nome e caminho não vazios.".to_string());
                }
                let mod_path = resolve_read_path(path_str);
                let payload = std::fs::read(&mod_path)
                    .map_err(|e| format!("Falha lendo módulo '{}': {}", mod_path.display(), e))?;
                let mut m = FrmPackModule::new(name, payload);
                m.version_u16 = ver_u16;
                m.hardware_family = hw.to_string();
                m.model_group = model.to_string();
                m.target_flag = flag;
                parsed_modules.push(m);
            }

            let container_mode = if mode_norm == "dvd" {
                FrmContainerMode::Dvd
            } else {
                FrmContainerMode::Sd
            };
            pack_frm(&parsed_modules, container_mode, None)
                .map_err(|e| format!("Falha empacotando .{}: {e}", mode_norm.to_uppercase()))?
        }
        other => {
            return Err(format!(
                "Modo inválido '{other}': esperado 'sd', 'dvd' ou 'digadiag'."
            ));
        }
    };

    std::fs::write(&out_path, &bytes)
        .map_err(|e| format!("Falha gravando '{}': {}", out_path.display(), e))?;
    Ok(report)
}

/// Unpacks and verifies an `EXTRHEAD`/`EXTRFOOT` binary container using partial Feistel decryption.
#[tauri::command]
pub fn unpack_extrhead_command(
    input_path: String,
    mode: Option<String>,
    out_path: Option<String>,
) -> Result<ExtrheadUnpackReportDto, String> {
    let trimmed = input_path.trim();
    if trimmed.is_empty() {
        return Err("Caminho do contêiner EXTRHEAD é obrigatório.".to_string());
    }
    let resolved_in = resolve_read_path(trimmed);
    let raw = std::fs::read(&resolved_in)
        .map_err(|e| format!("Falha lendo '{}': {}", resolved_in.display(), e))?;

    let crypt_mode: ExtrheadCryptMode = mode
        .as_deref()
        .unwrap_or("auto")
        .parse()
        .map_err(|e: anyhow::Error| e.to_string())?;
    let unpacked = unpack_extrhead(&raw, crypt_mode)
        .map_err(|e| format!("Falha ao desempacotar EXTRHEAD: {e}"))?;

    let written_output_path = if let Some(out_raw) = out_path.filter(|s| !s.trim().is_empty()) {
        let out_file = resolve_write_path(&out_raw)?;
        std::fs::write(&out_file, &unpacked.decompressed)
            .map_err(|e| format!("Falha gravando '{}': {}", out_file.display(), e))?;
        Some(out_file.to_string_lossy().to_string())
    } else {
        None
    };

    Ok(ExtrheadUnpackReportDto {
        file_path: resolved_in.to_string_lossy().to_string(),
        header_checksum_hex: format!("0x{:08X}", unpacked.header.header_checksum),
        calculated_checksum_hex: format!("0x{:08X}", unpacked.calculated_checksum),
        header: unpacked.header,
        decompressed_bytes: unpacked.decompressed.len(),
        checksum_valid: unpacked.checksum_valid,
        footer_present: unpacked.footer_present,
        written_output_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_pack_and_unpack_commands_roundtrip() {
        let tmp = std::env::temp_dir().join("panadiga_tauri_frm_pack_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let payload_path = tmp.join("diag_payload.bin");
        let digadiag_out = tmp.join("DIGADIAG.FRM");
        let unpacked_out = tmp.join("diag_out.bin");
        let sd_out = tmp.join("PANA__SD.FRM");

        let payload = vec![0xA5u8; 512];
        std::fs::write(&payload_path, &payload).unwrap();

        let pack_diag = pack_frm_command(PackFrmRequestDto {
            mode: "digadiag".to_string(),
            out_path: digadiag_out.to_string_lossy().to_string(),
            modules: Vec::new(),
            input_path: Some(payload_path.to_string_lossy().to_string()),
            dst_addr: Some("0x50400000".to_string()),
            tag: Some("FVU ".to_string()),
            build_name: Some("TEST01".to_string()),
            hw_family: None,
            model_group: None,
            version_u16: None,
            target_flag: None,
        })
        .unwrap();
        assert_eq!(pack_diag.mode, "digadiag");
        assert_eq!(pack_diag.total_bytes, PANA_DIGADIAG_FRM_SIZE);

        let unpacked = unpack_extrhead_command(
            digadiag_out.to_string_lossy().to_string(),
            Some("auto".to_string()),
            Some(unpacked_out.to_string_lossy().to_string()),
        )
        .unwrap();
        assert!(unpacked.checksum_valid);
        assert!(unpacked.footer_present);
        assert_eq!(unpacked.decompressed_bytes, 512);
        assert_eq!(std::fs::read(&unpacked_out).unwrap(), payload);

        let pack_sd = pack_frm_command(PackFrmRequestDto {
            mode: "sd".to_string(),
            out_path: sd_out.to_string_lossy().to_string(),
            modules: vec![FrmPackModuleInputDto {
                name: "DIAG01".to_string(),
                path: payload_path.to_string_lossy().to_string(),
            }],
            input_path: None,
            dst_addr: None,
            tag: None,
            build_name: None,
            hw_family: Some("M6930".to_string()),
            model_group: Some("NP-309".to_string()),
            version_u16: Some("0x0061".to_string()),
            target_flag: Some(0),
        })
        .unwrap();
        assert_eq!(pack_sd.mode, "sd");
        assert_eq!(pack_sd.modules.len(), 1);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
