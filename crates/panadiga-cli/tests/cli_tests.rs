use clap::Parser;
use panadiga_cli::cli::{Cli, Commands};

#[test]
fn test_cli_args_parsing_matrix() {
    let cli = Cli::try_parse_from(["panadiga", "memory-map", "--format", "json"]).unwrap();
    match cli.command {
        Some(Commands::MemoryMap(args)) => {
            assert_eq!(args.format, "json");
            assert!(args.output.is_none());
        }
        _ => panic!("Expected MemoryMap command"),
    }

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

    assert!(matches!(
        Cli::try_parse_from(["panadiga", "pico", "list"])
            .unwrap()
            .command,
        Some(Commands::Pico(_))
    ));
    assert!(matches!(
        Cli::try_parse_from(["panadiga", "pico", "flash", "--port", "COM3"])
            .unwrap()
            .command,
        Some(Commands::Pico(_))
    ));
    assert!(matches!(
        Cli::try_parse_from(["panadiga", "pico", "status", "--port", "COM10"])
            .unwrap()
            .command,
        Some(Commands::Pico(_))
    ));
    assert!(matches!(
        Cli::try_parse_from(["panadiga", "pico", "breakin", "--mode", "2"])
            .unwrap()
            .command,
        Some(Commands::Pico(_))
    ));
    assert!(matches!(
        Cli::try_parse_from([
            "panadiga",
            "pico",
            "dump",
            "--port",
            "COM10",
            "--addr",
            "0x807F0000",
            "--words",
            "16"
        ])
        .unwrap()
        .command,
        Some(Commands::Pico(_))
    ));
    assert!(matches!(
        Cli::try_parse_from([
            "panadiga",
            "gdb-bridge",
            "--port",
            "COM10",
            "--listen",
            "127.0.0.1:3333"
        ])
        .unwrap()
        .command,
        Some(Commands::GdbBridge { .. })
    ));

    assert!(matches!(
        Cli::try_parse_from(["panadiga", "hdd", "list-drives"])
            .unwrap()
            .command,
        Some(Commands::Hdd(_))
    ));
    assert!(matches!(
        Cli::try_parse_from([
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
        .unwrap()
        .command,
        Some(Commands::Hdd(_))
    ));
    assert!(matches!(
        Cli::try_parse_from([
            "panadiga",
            "hdd",
            "restore-mirrors",
            "--target",
            r"\\.\PhysicalDrive1",
            "--confirm",
            "CONFIRM-PhysicalDrive1",
        ])
        .unwrap()
        .command,
        Some(Commands::Hdd(_))
    ));

    assert!(matches!(
        Cli::try_parse_from([
            "panadiga",
            "frm",
            "pack",
            "--mode",
            "digadiag",
            "--input",
            "diag.bin",
            "--out",
            "DIGADIAG.FRM",
        ])
        .unwrap()
        .command,
        Some(Commands::Frm(_))
    ));
    assert!(matches!(
        Cli::try_parse_from([
            "panadiga",
            "frm",
            "unpack",
            "DIGADIAG.FRM",
            "--out",
            "diag_out.bin",
            "--mode",
            "plain",
        ])
        .unwrap()
        .command,
        Some(Commands::Frm(_))
    ));

    let cli = Cli::try_parse_from(["panadiga", "-i"]).unwrap();
    assert!(cli.interactive);

    assert!(Cli::try_parse_from(["panadiga", "non-existent-subcommand"]).is_err());
}
