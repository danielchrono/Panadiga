use flate2::Compression;
use flate2::write::GzEncoder;
use panadiga_core::constants::{PANASONIC_HDD_PARTITION_MAGIC, find_workspace_subdir};
use panadiga_core::headers::{hex_dump, inspect_firmware_headers};
use panadiga_core::manuals::search_manuals;
use panadiga_core::scan::scan_data_strings;
use std::io::Write;
use std::path::Path;

#[test]
fn test_headers_and_hex_dump() {
    let sample = b"Panasonic DIGA DMR-EH55 Engineering Suite Test Vector!";
    let dump = hex_dump(sample, 0x1000, 32);
    assert!(dump.contains("00001000"));
    assert!(dump.contains("Panasonic DIGA"));

    // 1. Tenta testar com o binário real caso ele esteja presente e íntegro no workspace
    let default_hdd = Path::new("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin.gz");
    if default_hdd.exists()
        && let Ok(metadata) = std::fs::metadata(default_hdd)
        && metadata.len() > 1024
        && inspect_firmware_headers(Some(default_hdd), None, 64).is_ok()
    {
        return;
    }

    // 2. Fallback rigoroso (sem simplificações): Constrói um container compactado em GZIP e .FRM sintéticos
    // com a volumetria exata (70 MB descompactados) para cobrir LBA_PARTITION_1_OFFSET (32MB) e LBA_PARTITION_2_OFFSET (64MB).
    let temp_dir = std::env::temp_dir();
    let mock_hdd_gz = temp_dir.join("panadiga_ci_mock_hdd.bin.gz");
    let mock_frm = temp_dir.join("panadiga_ci_mock_update.frm");

    {
        let file = std::fs::File::create(&mock_hdd_gz).expect("Failed to create mock gz file");
        let mut encoder = GzEncoder::new(file, Compression::fast());

        // 70 MB para garantir que 0x02000000 (32MB) e 0x04000000 (64MB) sejam alcançáveis pelo buffer descompactado
        let mut hdd_payload = vec![0u8; 70 * 1024 * 1024];

        let p1_offset = 0x0200_0000_usize;
        let p2_offset = 0x0400_0000_usize;

        let magic = PANASONIC_HDD_PARTITION_MAGIC;
        if p1_offset + magic.len() < hdd_payload.len() {
            hdd_payload[p1_offset..p1_offset + magic.len()].copy_from_slice(&magic);
        }
        if p2_offset + magic.len() < hdd_payload.len() {
            hdd_payload[p2_offset..p2_offset + magic.len()].copy_from_slice(&magic);
        }

        encoder
            .write_all(&hdd_payload)
            .expect("Failed to write mock compressed payload");
        encoder.finish().expect("Failed to finalize gzip stream");
    }

    // Simula o arquivo de atualização de firmware PANA_DVD.FRM estruturado
    {
        let mut frm_payload = vec![0u8; 4 * 1024 * 1024]; // 4 MB
        frm_payload[0..14].copy_from_slice(b"PANASONIC_FIRM");
        std::fs::write(&mock_frm, &frm_payload).expect("Failed to write mock FRM file");
    }

    // Executa a inspeção completa de headers
    let res = inspect_firmware_headers(Some(&mock_hdd_gz), Some(&mock_frm), 64);

    // Limpeza dos arquivos temporários de teste
    let _ = std::fs::remove_file(&mock_hdd_gz);
    let _ = std::fs::remove_file(&mock_frm);

    assert!(
        res.is_ok(),
        "inspect_firmware_headers failed on structured mock: {:?}",
        res.err()
    );
}

#[test]
fn test_scan_strings() {
    let mut data = vec![0u8; 128];
    data[10..24].copy_from_slice(b"panasonic_diga");
    data[50..65].copy_from_slice(b"error_u99_fatal");

    let keywords = vec!["u99".to_string(), "panasonic".to_string()];
    let (total, matches) = scan_data_strings(&data, 6, &keywords, 10);

    assert_eq!(total, 2);
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].text, "panasonic_diga");
    assert_eq!(matches[0].matched_keyword.as_deref(), Some("panasonic"));
    assert_eq!(matches[1].text, "error_u99_fatal");
    assert_eq!(matches[1].matched_keyword.as_deref(), Some("u99"));
}

#[test]
fn test_manuals_search_pdf() {
    let Some(manuals_dir) = find_workspace_subdir("manuals_and_schematics") else {
        return;
    };

    let terms = vec!["U99".to_string(), "FIRMWARE".to_string()];
    let res = search_manuals(&manuals_dir, Some(terms), Some("dmr-eh55p9"))
        .expect("Manual search failed");

    assert!(!res.is_empty());
    let r = &res[0];
    assert!(r.file_name.contains("eh55"));
}
