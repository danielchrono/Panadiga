//! Panasonic DMR-EH55 DIGA Suite - Hardware & Firmware Single Source of Truth (SSOT) DTO
//!
//! Exposes canonical constants from `crate::constants` and `crate::pico::platforms`,
//! P7501 6-pin Async UART mapping, discrete boot/reset specs, NOR Flash geometries,
//! AM34 memory regions, and the full granular P7501 UART/FMU Pico command catalog.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::constants::{
    AM34_NVRAM_BASE, AM34_SRAM_LOADER_BASE, AM34_SYSTEM_MEMORY_REGIONS, CANDIDATE_MANUALS_DIRS,
    DEFAULT_BAUD_RATE, DEFAULT_GDB_LISTEN_SOCKET,
};
use crate::manuals::count_pdf_files;
use crate::pico::platforms::{
    PICO_BOOTSEL_PID, PICO_RP2040_CDC_PID, PICO_RP2040_W_CDC_PID, PICO_RP2350_CDC_PID,
    PICO_RP2350_RISCV_CDC_PID, PICO_RP2350_W_CDC_PID, RASPBERRY_PI_VID,
};

/// Canonical electrical, timing, and bus address thresholds synchronized with C firmware (`p7501_types.h`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareThresholdsDto {
    pub bus_freq_hz: u32,
    pub nominal_vdd_volts: f32,
    pub min_vdd_threshold_volts: f32,
    pub safe_operating_min_volts: f32,
    pub max_vdd_ceiling_volts: f32,
    pub default_baud_rate: u32,
    pub default_gdb_socket: String,
    pub ram_loader_base_hex: String,
    pub default_nvram_base_hex: String,
}

/// Official Raspberry Pi RP2040/RP2350 USB VID/PID constants for host discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoUsbIdentityDto {
    pub raspberry_pi_vid: u16,
    pub rp2040_cdc_pid: u16,
    pub rp2040_w_cdc_pid: u16,
    pub rp2350_cdc_pid: u16,
    pub rp2350_w_cdc_pid: u16,
    pub rp2350_riscv_cdc_pid: u16,
    pub bootsel_pid: u16,
}

/// Physical and manual pin mapping entry for P7501 JST B6B-ZR-3R4.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P7501PinMappingDto {
    pub manual_pin: u8,
    pub physical_pin: u8,
    pub pin_label: String,
    pub pico_gpio: String,
    pub pico_pin: String,
    pub signal_name: String,
    pub direction: String,
    pub voltage: String,
    pub critical_note: Option<String>,
}

/// Cold Break-In mode descriptor matching `protocol/p7501_bus_protocol.c`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakInModeSpecDto {
    pub mode: u8,
    pub label: String,
    pub badge: String,
    pub badge_variant: String,
    pub description: String,
    pub signal_diagram: String,
}

/// Supported Panasonic DIGA Top-Boot Spansion NOR Flash geometry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashGeometrySpecDto {
    pub size_mb: u32,
    pub size_bytes: u32,
    pub chip_name: String,
    pub nvram_base_hex: String,
    pub nvram_size_bytes: u32,
    pub microsector_range: String,
}

/// Structured AM34 memory map region DTO mapped from `AM34_SYSTEM_MEMORY_REGIONS`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegionDto {
    pub name: String,
    pub start: String,
    pub end: String,
    pub size_kb: usize,
    pub perms: String,
}

/// Option entry for parameterized Pico commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoCommandArgOptionDto {
    pub label: String,
    pub value: String,
}

/// Granular Pico P7501 Async UART / FMU firmware command descriptor (`cli/p7501_cli.c`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoCommandSpecDto {
    pub id: String,
    pub category: String,
    pub label: String,
    pub description: String,
    pub syntax: String,
    pub arg_label: Option<String>,
    pub arg_placeholder: Option<String>,
    pub default_arg: Option<String>,
    pub arg_options: Vec<PicoCommandArgOptionDto>,
    pub dangerous: bool,
    pub timeout_ms: u64,
}

/// Root SSOT payload delivered to Frontend UI and ViewModels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSsotSpecDto {
    pub thresholds: HardwareThresholdsDto,
    pub usb_ids: PicoUsbIdentityDto,
    pub indexed_manuals_count: usize,
    pub pinout: Vec<P7501PinMappingDto>,
    pub breakin_modes: Vec<BreakInModeSpecDto>,
    pub flash_geometries: Vec<FlashGeometrySpecDto>,
    pub memory_regions: Vec<MemoryRegionDto>,
    pub pico_commands: Vec<PicoCommandSpecDto>,
}

fn opt(label: &str, value: &str) -> PicoCommandArgOptionDto {
    PicoCommandArgOptionDto {
        label: label.to_string(),
        value: value.to_string(),
    }
}

fn cmd(
    id: &str,
    category: &str,
    label: &str,
    description: &str,
    syntax: &str,
    arg_spec: Option<(&str, &str, &str, Vec<PicoCommandArgOptionDto>)>,
    dangerous: bool,
    timeout_ms: u64,
) -> PicoCommandSpecDto {
    let (arg_label, arg_placeholder, default_arg, arg_options) = match arg_spec {
        Some((al, ap, da, opts)) => (
            Some(al.to_string()),
            Some(ap.to_string()),
            Some(da.to_string()),
            opts,
        ),
        None => (None, None, None, Vec::new()),
    };
    PicoCommandSpecDto {
        id: id.to_string(),
        category: category.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        syntax: syntax.to_string(),
        arg_label,
        arg_placeholder,
        default_arg,
        arg_options,
        dangerous,
        timeout_ms,
    }
}

fn pin(
    manual_pin: u8,
    physical_pin: u8,
    pin_label: &str,
    pico_gpio: &str,
    pico_pin: &str,
    signal_name: &str,
    direction: &str,
    voltage: &str,
    critical_note: Option<&str>,
) -> P7501PinMappingDto {
    P7501PinMappingDto {
        manual_pin,
        physical_pin,
        pin_label: pin_label.to_string(),
        pico_gpio: pico_gpio.to_string(),
        pico_pin: pico_pin.to_string(),
        signal_name: signal_name.to_string(),
        direction: direction.to_string(),
        voltage: voltage.to_string(),
        critical_note: critical_note.map(|s| s.to_string()),
    }
}

/// Constructs the canonical SSOT hardware/firmware specification for UI consumers.
pub fn build_hardware_ssot_spec() -> HardwareSsotSpecDto {
    let indexed_manuals_count = crate::constants::find_workspace_subdir("manuals_and_schematics")
        .or_else(|| {
            CANDIDATE_MANUALS_DIRS
                .iter()
                .map(Path::new)
                .find(|p| p.is_dir())
                .map(Path::to_path_buf)
        })
        .as_deref()
        .map(count_pdf_files)
        .unwrap_or(0);

    let memory_regions = AM34_SYSTEM_MEMORY_REGIONS
        .iter()
        .map(|r| MemoryRegionDto {
            name: r.name.to_string(),
            start: format!("0x{:08X}", r.start),
            end: format!("0x{:08X}", r.end),
            size_kb: r.size_kb,
            perms: r.perms.to_string(),
        })
        .collect();

    HardwareSsotSpecDto {
        thresholds: HardwareThresholdsDto {
            bus_freq_hz: 1_000_000,
            nominal_vdd_volts: 3.30,
            min_vdd_threshold_volts: 2.50,
            safe_operating_min_volts: 3.00,
            max_vdd_ceiling_volts: 3.45,
            default_baud_rate: DEFAULT_BAUD_RATE,
            default_gdb_socket: DEFAULT_GDB_LISTEN_SOCKET.to_string(),
            ram_loader_base_hex: format!("0x{:08X}", AM34_SRAM_LOADER_BASE),
            default_nvram_base_hex: format!("0x{:08X}", AM34_NVRAM_BASE),
        },
        usb_ids: PicoUsbIdentityDto {
            raspberry_pi_vid: RASPBERRY_PI_VID,
            rp2040_cdc_pid: PICO_RP2040_CDC_PID,
            rp2040_w_cdc_pid: PICO_RP2040_W_CDC_PID,
            rp2350_cdc_pid: PICO_RP2350_CDC_PID,
            rp2350_w_cdc_pid: PICO_RP2350_W_CDC_PID,
            rp2350_riscv_cdc_pid: PICO_RP2350_RISCV_CDC_PID,
            bootsel_pid: PICO_BOOTSEL_PID,
        },
        indexed_manuals_count,
        pinout: vec![
            pin(
                1,
                1,
                "Pino 1 [P7501 SW_UART / XFWE]",
                "GP26 (ADC0 / Pin 31)",
                "Pino 31",
                "SW_UART & ADC0 Sense (~3.25V Hi-Z / ~2.69V PD)",
                "Target <-> Pico (Q7510/QR7503)",
                "3.3V LVTTL",
                Some("LOW força serial boot via XFWE (corta Q7514 TX). Mantenha Hi-Z para TX normal e NUNCA alimente o Pico por este pino."),
            ),
            pin(
                2,
                2,
                "Pino 2 [P7501 XRESET]",
                "GP5 (Pin 7)",
                "Pino 7",
                "XRESET Open-Drain (Nunca Push-Pull)",
                "Pico Open-Drain",
                "3.3V Hi-Z / 0V",
                None,
            ),
            pin(
                3,
                3,
                "Pino 3 [P7501 UARTM2P_T / Q7514]",
                "GP4 (Pin 6)",
                "Pino 6",
                "UART TX -> AM34 RX via Q7514",
                "Pico -> Target (115200 8N1)",
                "3.3V LVTTL",
                Some("Pull-up R7570 10kΩ para NSW5V. Requer SW_UART=Hi-Z para Q7514 conduzir até o SoC AM34."),
            ),
            pin(
                4,
                4,
                "Pino 4 [P7501 GND]",
                "GND (Pin 38)",
                "Pino 38",
                "Terra 0.0Ω Chassi",
                "Terra Comum",
                "0.0V (VSS)",
                Some("Manual P7501 B6B-ZR-3R4 = PINO 4 (referência comum 0.0V). Confira continuidade 0.0Ω antes de ligar."),
            ),
            pin(
                5,
                5,
                "Pino 5 [P7501 NSW5V / IC1506]",
                "GP3 (Pin 5)",
                "Pino 5",
                "NSW5V Sense Hi-Z ONLY (~3.29V em repouso)",
                "Target -> Pico (Hi-Z Input Sem Pulls)",
                "~3.29V (IC1506 ~5.15V upstream)",
                Some("CRÍTICO: No DMR-EH55 P7501 Pino 5 é barramento NSW5V (originado em IC1506 ~5.15V via D7510). SEMPRE Hi-Z Input sem pulls — NUNCA dirigir push-pull!"),
            ),
            pin(
                6,
                6,
                "Pino 6 [P7501 UARTP2M_T / MAX TXD]",
                "GP2 (Pin 4)",
                "Pino 4",
                "UART RX <- AM34 TX 115200 8N1",
                "Target -> Pico (115200 8N1)",
                "3.3V LVTTL",
                Some("Única linha UART TX ativa observada no DMR-EH55 P7501 (R7572 10kΩ para NSW5V)."),
            ),
        ],
        breakin_modes: vec![
            BreakInModeSpecDto {
                mode: 0,
                label: "Modo 0: Normal Hi-Z (SW_UART Aberto)".to_string(),
                badge: "Passivo".to_string(),
                badge_variant: "secondary".to_string(),
                description:
                    "Mantém SW_UART (GP26) em Hi-Z durante pulso Open-Drain de XRESET; Q7514 permanece conduzindo UARTM2P_T -> AM34.".to_string(),
                signal_diagram: "SW_UART: Hi-Z | XRESET: 10ms Open-Drain Low".to_string(),
            },
            BreakInModeSpecDto {
                mode: 1,
                label: "Modo 1: Serial Pulse (XFWE Pré-Reset)".to_string(),
                badge: "FMU".to_string(),
                badge_variant: "secondary".to_string(),
                description:
                    "Pulsa SW_UART LOW apenas durante a liberação de XRESET e libera GP26 em Hi-Z para habilitar TX bidirecional após handshake.".to_string(),
                signal_diagram: "SW_UART: Pulse Low -> Hi-Z | Q7514 Re-enabled".to_string(),
            },
            BreakInModeSpecDto {
                mode: 2,
                label: "Modo 2: JIG Hold LOW (SW_UART Retido)".to_string(),
                badge: "RECOMENDADO".to_string(),
                badge_variant: "success".to_string(),
                description:
                    "Padrão JIGBOOT para captura UART do AM34: mantém SW_UART em LOW via Q7510 durante toda a janela de inicialização e retorna a Hi-Z.".to_string(),
                signal_diagram: "SW_UART: Hold Low -> Isolate Hi-Z | 115200 8N1 RX".to_string(),
            },
            BreakInModeSpecDto {
                mode: 3,
                label: "Modo 3: Hi-Z Estendido (Captura Longa)".to_string(),
                badge: "Seguro".to_string(),
                badge_variant: "warning".to_string(),
                description:
                    "Executa reset Open-Drain em XRESET mantendo os pinos UART em escuta Hi-Z para inspeção completa de CDDIAG e DDR CHECK.".to_string(),
                signal_diagram: "XRESET: Open-Drain | GP2 RX + GP3 NSW5V Hi-Z".to_string(),
            },
        ],
        flash_geometries: vec![
            FlashGeometrySpecDto {
                size_mb: 2,
                size_bytes: 0x0020_0000,
                chip_name: "Spansion S29AL016J NOR Flash (2 MB / 16 Mbit)".to_string(),
                nvram_base_hex: "0x801F0000".to_string(),
                nvram_size_bytes: 0x0001_0000,
                microsector_range: "SA38..SA45".to_string(),
            },
            FlashGeometrySpecDto {
                size_mb: 4,
                size_bytes: 0x0040_0000,
                chip_name: "Spansion S29JL032H NOR Flash (4 MB / 32 Mbit)".to_string(),
                nvram_base_hex: "0x803F0000".to_string(),
                nvram_size_bytes: 0x0001_0000,
                microsector_range: "SA70..SA77".to_string(),
            },
            FlashGeometrySpecDto {
                size_mb: 8,
                size_bytes: 0x0080_0000,
                chip_name: "Spansion S29JL064H NOR Flash (8 MB / 64 Mbit)".to_string(),
                nvram_base_hex: "0x807F0000".to_string(),
                nvram_size_bytes: 0x0001_0000,
                microsector_range: "SA134..SA141".to_string(),
            },
            FlashGeometrySpecDto {
                size_mb: 16,
                size_bytes: 0x0100_0000,
                chip_name: "Spansion S29GL128P NOR Flash (16 MB / 128 Mbit)".to_string(),
                nvram_base_hex: "0x80FE0000".to_string(),
                nvram_size_bytes: 0x0002_0000,
                microsector_range: "SA254..SA269".to_string(),
            },
        ],
        memory_regions,
        pico_commands: vec![
            // --- Electrical & Bus Control ---
            cmd("PINCHECK", "electrical", "Matriz Elétrica (PINCHECK)", "Diagnostica os 6 pinos do conector P7501 em Hi-Z, Pull-Down e Pull-Up + rise time do XRESET.", "PINCHECK", None, false, 4000),
            cmd("BUSDIAG", "electrical", "Contenção & Boot (BUSDIAG)", "Varre drive strengths (2..12 mA) em GP4/UARTM2P_T e perfil de boot em GP2 (UART RX) com SW_UART Hi-Z vs LOW.", "BUSDIAG", None, false, 12000),
            cmd("SNIFF", "electrical", "Monitor Passivo (SNIFF)", "Conta transições de borda passivas em GP2..GP5 durante janela configurável (ms).", "SNIFF [ms]", Some(("Janela (ms)", "250", "250", vec![opt("250 ms", "250"), opt("1000 ms", "1000"), opt("3000 ms", "3000")])), false, 5000),
            cmd("ISOLATE", "electrical", "Isolar Barramento (Hi-Z)", "Coloca GP2..GP5 em alta impedância para não interferir no boot normal do DMR-EH55.", "ISOLATE", None, false, 1500),
            cmd("ENGAGE", "electrical", "Engajar UART / PIO", "Ativa drive P7501 PIO / UART em 3.3V mantendo NSW5V (GP3) em Hi-Z.", "ENGAGE", None, false, 1500),
            cmd("RESET", "electrical", "Pulso Open-Drain XRESET", "Puxa XRESET para 0V em dreno aberto por 10 ms e libera em Hi-Z.", "RESET", None, false, 2000),
            // --- P7501 Boot Sequences & NOR Flash ---
            cmd("HANDSHAKE", "ocd_flash", "Executar Boot JIG", "Executa sequência JIGBOOT / P7501 (SW_UART+XRESET) e lê resposta inicial da ROM.", "HANDSHAKE", None, false, 4000),
            cmd("BREAK_IN", "ocd_flash", "Perfil Boot/Reset", "Executa perfil de boot/reset discreto no P7501 (modos 0..3: Hi-Z, Serial Pulse, JIG Hold LOW, Extended).", "BREAK_IN [0..3]", Some(("Modo P7501", "2", "2", vec![opt("Modo 0 (SW_UART Hi-Z)", "0"), opt("Modo 1 (Serial Pulse)", "1"), opt("Modo 2 (JIG Hold LOW)", "2"), opt("Modo 3 (Hi-Z Extended)", "3")])), false, 3000),
            cmd("SCAN", "ocd_flash", "Sondar Barramento UART", "Audita estabilidade elétrica e detecção automática de atividade UART no P7501.", "SCAN", None, false, 18000),
            cmd("AUTODETECT", "ocd_flash", "Autodetectar Flash (Leitura)", "Mede espelhamento passivo de endereço (2/4/8/16 MB) sem gravação na NOR Flash.", "AUTODETECT", None, false, 3000),
            cmd("CFI", "ocd_flash", "Consulta JEDEC CFI", "Envia sequência JEDEC 0x98 para leitura CFI (assinatura QRY / expoente 2^N da Spansion).", "CFI", None, false, 3000),
            cmd("FLASH_SIZE", "ocd_flash", "Definir Geometria Flash", "Ajusta manualmente o tamanho esperado da NOR Flash para leituras/GDB RSP.", "FLASH_SIZE [2|4|8|16]", Some(("Tamanho Flash", "8", "8", vec![opt("2 MB (S29AL016J)", "2"), opt("4 MB (S29JL032H)", "4"), opt("8 MB (S29JL064H)", "8"), opt("16 MB (S29GL128P)", "16")])), false, 2000),
            cmd("DUMP", "ocd_flash", "Dump Memória", "Lê palavras de 32 bits em endereço arbitrário da SRAM/Flash/MMIO do AM34.", "DUMP <hex_addr> <words>", Some(("Endereço e Contagem", "0x807F0000 16", "0x807F0000 16", vec![opt("NVRAM Header (0x807F0000 16)", "0x807F0000 16"), opt("Bootloader Reset (0x80000000 16)", "0x80000000 16"), opt("SRAM Loader (0x00004000 16)", "0x00004000 16")])), false, 8000),
            // --- UART & Bootrom FMU ---
            cmd("BOOTCAP", "uart_fmu", "Captura Boot UART (BOOTCAP)", "Reinicia o gravador e captura telemetria 115200 8N1 em GP2 durante a janela configurada.", "BOOTCAP [ms]", Some(("Duração (ms)", "7500", "7500", vec![opt("3.0 s", "3000"), opt("7.5 s", "7500"), opt("15.0 s", "15000")])), false, 32000),
            cmd("UARTMON", "uart_fmu", "Monitor UART Passivo", "Escuta canal GP2 115200 8N1 sem acionar reset no gravador.", "UARTMON [ms]", Some(("Duração (ms)", "5000", "5000", vec![opt("2.0 s", "2000"), opt("5.0 s", "5000"), opt("10.0 s", "10000")])), false, 32000),
            cmd("JIGBOOT", "uart_fmu", "Boot Modo Jig (JIGBOOT)", "Inicializa o chip mantendo pino SW_UART em LOW durante a janela de captura.", "JIGBOOT [ms]", Some(("Duração (ms)", "7500", "7500", vec![opt("5.0 s", "5000"), opt("7.5 s", "7500")])), false, 32000),
            cmd("FMUSCAN", "uart_fmu", "Sweep Bootrom FMU (FMUSCAN)", "Varre sequências de handshake FMU da Bootrom Panasonic na janela Wait/Serial.", "FMUSCAN [modo]", Some(("Modo FMU", "all", "all", vec![opt("Todos (all)", "all"), opt("Modo 0", "0"), opt("Modo 1", "1"), opt("Modo 2 (seri)", "2"), opt("Modo 3 (DDR)", "3")])), false, 20000),
            cmd("FMUTEST", "uart_fmu", "Injetar Payload Hex FMU", "Transmite payload hexadecimal arbitrário durante handshake SW_UART FMU e captura eco.", "FMUTEST <hex>", Some(("Payload Hex", "0D", "0D", vec![opt("CR (0D)", "0D"), opt("CR+LF (0D0A)", "0D0A"), opt("Help (3F0D)", "3F0D")])), false, 10000),
            cmd("FMUBATCH", "uart_fmu", "Lote Multi-Comando FMU", "Executa lote automatizado de probes Bootrom FMU em sequência controlada.", "FMUBATCH [step]", Some(("Etapa / Batch", "", "", Vec::new())), false, 22000),
            cmd("UARTSEND", "uart_fmu", "Enviar String TX P7501", "Transmite string ASCII + CRLF com SW_UART=Hi-Z padrão (Q7514 em condução) ou prefixo LOW: e escuta 2.5s de resposta RX.", "UARTSEND <texto>", Some(("Texto ASCII TX", "help", "help", vec![opt("help", "help"), opt("ver", "ver"), opt("info", "info")])), false, 5000),
            cmd("UARTPROBE", "uart_fmu", "Probe Interativo UART", "Testa modos 0..8 de estímulo UART/BREAK durante boot para descobrir console oculto.", "UARTPROBE [0..8]", Some(("Modo Probe", "5", "5", vec![opt("Modo 5 (SW_UART Low)", "5"), opt("Modo 6 (Auto Probe)", "6"), opt("Modo 7 (Break 550ms)", "7")])), false, 8000),
            // --- Probe Controls ---
            cmd("PING", "probe", "Ping USB CDC / BLE", "Verifica responsividade imediata da camada CLI do firmware Pico.", "PING", None, false, 1500),
            cmd("STATUS", "probe", "Ler Telemetria VDD/ADC", "Lê tensão alvo no canal ADC0 (GP26) com detector ativo de pino flutuante.", "STATUS", None, false, 1500),
            cmd("BANNER", "probe", "Versão & Banner Firmware", "Exibe identificação completa da compilação RP2040/RP2350 em execução na sonda.", "BANNER", None, false, 1500),
            cmd("LED", "probe", "Modo LED Onboard GP25", "Alterna o padrão de sinalização visual do LED onboard do Pico.", "LED <modo>", Some(("Padrão LED", "IDLE", "IDLE", vec![opt("IDLE (1Hz Heartbeat)", "IDLE"), opt("ACTIVE (Sólido)", "ACTIVE"), opt("GDB (5Hz)", "GDB"), opt("FAULT (Triplo)", "FAULT")])), false, 1500),
            cmd("BOOTSEL", "probe", "Reiniciar para BOOTSEL USB", "Reinicia o Pico para bootloader USB Mass Storage (`RPI-RP2` / `RP2350`).", "BOOTSEL", None, true, 1500),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssot_thresholds_match_firmware_and_platform_constants() {
        let spec = build_hardware_ssot_spec();
        assert!((spec.thresholds.min_vdd_threshold_volts - 2.50).abs() < f32::EPSILON);
        assert!((spec.thresholds.max_vdd_ceiling_volts - 3.45).abs() < f32::EPSILON);
        assert_eq!(spec.usb_ids.raspberry_pi_vid, 0x2E8A);
        assert_eq!(spec.usb_ids.rp2350_cdc_pid, 0x000F);
        assert_eq!(spec.pinout.len(), 6);
        assert_eq!(
            spec.pinout[1].signal_name,
            "XRESET Open-Drain (Nunca Push-Pull)"
        );
        assert_eq!(spec.pinout[2].signal_name, "UART TX -> AM34 RX via Q7514");
        assert_eq!(spec.pinout[3].manual_pin, 4);
        assert_eq!(spec.pinout[3].signal_name, "Terra 0.0Ω Chassi");
        assert_eq!(spec.breakin_modes.len(), 4);
        assert_eq!(spec.flash_geometries.len(), 4);
        assert!(spec.pico_commands.len() >= 25);
        assert!(spec.indexed_manuals_count >= 15);
    }
}
