use clap::Parser;
use flate2::write::GzEncoder;
use flate2::Compression;
use panadiga::cli::{Cli, Commands};
use panadiga::headers::{hex_dump, inspect_firmware_headers};
use panadiga::manuals::search_manuals;
use panadiga::scan::scan_data_strings;
use std::io::Write;
use std::path::Path;

#[test]
fn test_headers_and_hex_dump() {
    let sample = b"Panasonic DIGA DMR-EH55 Engineering Suite Test Vector!";
    let dump = hex_dump(sample, 0x1000, 32);
    assert!(dump.contains("00001000"));
    assert!(dump.contains("Panasonic DIGA"));

    // Verifica se a imagem de firmware real existe no ambiente de trabalho
    let default_hdd = Path::new("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin.gz");
    if !default_hdd.exists() {
        eprintln!(
            "Skipping test_headers_and_hex_dump: firmware fixture not present in CI workspace"
        );
        return;
    }

    // Se o arquivo existir, executa a inspeção real
    assert!(inspect_firmware_headers(None, None, 64).is_ok());
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
    let manuals_dir = Path::new("manuals_and_schematics");
    if !manuals_dir.exists() {
        return;
    }

    let terms = vec!["U99".to_string(), "FIRMWARE".to_string()];
    let res =
        search_manuals(manuals_dir, Some(terms), Some("dmr-eh55p9")).expect("Manual search failed");

    assert!(!res.is_empty());
    let r = &res[0];
    assert!(r.file_name.contains("eh55"));
}

#[test]
fn test_cli_args_parsing_matrix() {
    // 1. memory-map
    let cli = Cli::try_parse_from(["panadiga", "memory-map", "--format", "json"]).unwrap();
    match cli.command {
        Some(Commands::MemoryMap(args)) => {
            assert_eq!(args.format, "json");
            assert!(args.output.is_none());
        }
        _ => panic!("Expected MemoryMap command"),
    }

    // 2. gdb-config
    let cli = Cli::try_parse_from([
        "panadiga",
        "gdb-config",
        "--port",
        "COM7",
        "--tcp-port",
        "4444",
    ])
    .unwrap();
    match cli.command {
        Some(Commands::GdbConfig(args)) => {
            assert_eq!(args.port, "COM7");
            assert_eq!(args.tcp_port, 4444);
        }
        _ => panic!("Expected GdbConfig command"),
    }

    // 3. pico list & pico flash
    let cli = Cli::try_parse_from(["panadiga", "pico", "list"]).unwrap();
    assert!(matches!(cli.command, Some(Commands::Pico(_))));

    let cli_flash = Cli::try_parse_from(["panadiga", "pico", "flash", "--port", "COM3"]).unwrap();
    assert!(matches!(cli_flash.command, Some(Commands::Pico(_))));

    // 4. hdd list-drives & format-meihdfs
    let cli_drives = Cli::try_parse_from(["panadiga", "hdd", "list-drives"]).unwrap();
    assert!(matches!(cli_drives.command, Some(Commands::Hdd(_))));

    let cli_format = Cli::try_parse_from([
        "panadiga",
        "hdd",
        "format-meihdfs",
        "--target",
        r"\\.\PhysicalDrive1",
        "--size-gb",
        "250",
        "--confirm",
        "CONFIRM-PhysicalDrive1",
    ])
    .unwrap();
    assert!(matches!(cli_format.command, Some(Commands::Hdd(_))));

    let cli_restore = Cli::try_parse_from([
        "panadiga",
        "hdd",
        "restore-mirrors",
        "--target",
        r"\\.\PhysicalDrive1",
        "--confirm",
        "CONFIRM-PhysicalDrive1",
    ])
    .unwrap();
    assert!(matches!(cli_restore.command, Some(Commands::Hdd(_))));

    // 5. interactive flag
    let cli = Cli::try_parse_from(["panadiga", "-i"]).unwrap();
    assert!(cli.interactive);

    // 6. invalid command rejection
    let err = Cli::try_parse_from(["panadiga", "non-existent-subcommand"]);
    assert!(err.is_err());
}
