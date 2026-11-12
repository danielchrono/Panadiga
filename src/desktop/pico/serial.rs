//! Panasonic DMR-EH55 DIGA Suite - Pico USB Serial Driver
//!
//! Single Responsibility: Manages low-level USB CDC communication, device enumeration,
//! telemetry status querying, raw CDC command dispatch, and Cold Break-In sequencing.
//! In-Circuit Flash/NVRAM erasure orchestration resides in `erase.rs`.

use serialport::SerialPortType;
use std::io::Write;
use std::time::{Duration, Instant};

use super::gdb_bridge::is_gdb_bridge_active;
use super::types::{PicoDeviceStatusDto, PicoStatusResponse, extract_port_name};

/// Verifies that the serial port is not currently owned by the active GDB RSP bridge thread.
pub fn ensure_serial_port_unlocked(clean_port: &str) -> Result<(), String> {
    if !clean_port.starts_with("MOCK") && is_gdb_bridge_active() {
        return Err(format!(
            "Porta serial {} ocupada pela Ponte GDB RSP ativa. Encerre a sessão GDB antes de enviar comandos CDC.",
            clean_port
        ));
    }
    Ok(())
}

/// Reads multi-chunk USB CDC serial output until a terminal prompt (`DWIRE32A>`),
/// completion keyword (`OK`, `ERROR`, `COMPLETED`, `SUCCESS`, `FAILED`, `ABORT`, `ACTIVE`, `Auto-ready`),
/// or until `max_wait` expires, preventing 64-byte USB packet truncation.
pub fn read_serial_response_complete(
    port: &mut dyn serialport::SerialPort,
    max_wait: Duration,
) -> Result<String, String> {
    let _ = port.set_timeout(Duration::from_millis(180));
    let start = Instant::now();
    let mut out = String::new();
    let mut chunk = [0u8; 1024];

    while start.elapsed() < max_wait {
        match port.read(&mut chunk) {
            Ok(n) if n > 0 => {
                out.push_str(&String::from_utf8_lossy(&chunk[..n]));
                if out.contains("DWIRE32A>")
                    || out.contains("DUMP: OK")
                    || out.contains("ERASE_NVRAM: OK")
                    || out.contains("ERASE_NVRAM: ERROR")
                    || out.contains("INJECT_LOADER: OK")
                    || out.contains("INJECT_LOADER: ERROR")
                    || out.contains("SAFETY INTERLOCK")
                    || out.contains("BREAK_IN: OK")
                    || out.contains("SCAN: COMPLETED")
                    || out.contains("SNIFF: OK")
                    || out.contains("ISOLATE: OK")
                    || out.contains("ENGAGE: OK")
                    || out.contains("AUTODETECT: OK")
                    || out.contains("FLASH_SIZE: OK")
                    || out.contains("FLASH_SIZE: ERROR")
                    || out.contains("[HANDSHAKE] SUCCESS")
                    || out.contains("[HANDSHAKE] FAILED")
                    || out.contains("[HANDSHAKE] ABORT")
                    || out.contains("[HANDSHAKE] ACTIVE")
                    || out.contains("Auto-ready")
                    || out.contains("PONG")
                    || out.contains("VDD:")
                    || out.contains("VDD=")
                {
                    break;
                }
            }
            Ok(_) => {}
            Err(ref e)
                if e.kind() == std::io::ErrorKind::TimedOut
                    || e.kind() == std::io::ErrorKind::WouldBlock =>
            {
                if !out.is_empty() && start.elapsed() > Duration::from_millis(350) {
                    break;
                }
            }
            Err(e) => return Err(format!("Erro de leitura USB CDC: {}", e)),
        }
    }
    Ok(out.trim().to_string())
}

/// Lists all available serial COM ports, tagging recognized RP2040 and RP2350 hardware,
/// and includes high-fidelity bench simulator options.
/// NOTE: Pico in BOOTSEL mode (VID:2E8A PID:0003) appears as USB mass storage, NOT as a COM port.
/// If the OS does register a COM entry for it, we exclude it here to prevent "Acesso Negado"
/// when the connection dialog iterates all ports.
pub fn list_pico_serial_ports() -> Result<Vec<String>, String> {
    let mut ports = Vec::new();
    if let Ok(available) = serialport::available_ports() {
        for p in available {
            let mut label = p.port_name.clone();
            if let SerialPortType::UsbPort(info) = &p.port_type {
                // Skip BOOTSEL-mode entries: PID:0003 is the RP2040/RP2350 UF2 mass-storage bootloader.
                // Opening a serial port on BOOTSEL drives always fails with "Access Denied"
                // because the OS treats them as block devices, not CDC ACM.
                if info.pid == crate::pico::platforms::PICO_BOOTSEL_PID {
                    continue;
                }
                let (_plat, desc) =
                    crate::pico::classify_usb_device(info.vid, info.pid, info.product.as_deref());
                label = format!(
                    "{} [{} - VID:{:04X} PID:{:04X}]",
                    p.port_name, desc, info.vid, info.pid
                );
            }
            ports.push(label);
        }
    }
    // Removed mock ports injection to ensure only real hardware ports are listed.
    // ports.push("MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]".to_string());
    // ports.push("MOCK_RP2350 [RP2350 Pico 2 DWire32A Bench Simulator - 520KB SRAM]".to_string());
    // ports.push("MOCK_RP2350_RISCV [RP2350 Hazard3 RISC-V Bench Simulator - 520KB SRAM]".to_string());
    Ok(ports)
}

/// Connects to a serial port (or initializes mock simulator) and retrieves device telemetry.
pub fn pico_connect_and_test(
    port_name: String,
    baud_rate: u32,
) -> Result<PicoStatusResponse, String> {
    let clean = extract_port_name(&port_name);
    if clean.starts_with("MOCK") {
        let is_riscv = clean.contains("RISCV") || port_name.contains("RISCV");
        let is_rp2350 = clean.contains("2350") || port_name.contains("2350");
        let (soc_name, sram_desc) = if is_riscv {
            (
                "RP2350 (Pico 2 RISC-V Hazard3)",
                "520 KB SRAM / 64 KB Cache Setor",
            )
        } else if is_rp2350 {
            (
                "RP2350 (Pico 2 ARM Cortex-M33)",
                "520 KB SRAM / 64 KB Cache Setor",
            )
        } else {
            (
                "RP2040 (Pico 1 Dual Cortex-M0+)",
                "264 KB SRAM / 8 KB Cache Microsetor",
            )
        };

        return Ok(PicoStatusResponse {
            connected: true,
            port_name: format!("{} Simulator", soc_name),
            baud_rate,
            vdd_voltage: 3.32,
            target_powered: true,
            firmware_banner: format!(
                "PANASONIC DWIRE32A / OCD ENGINE v2.0 ({}, {})",
                soc_name, sram_desc
            ),
            is_mock: true,
            message: format!(
                "Conectado ao Simulador de Bancada {}. Barramento DWire32A e alvo DMR-EH55 MN2DS0016 prontos para comandos e ponte GDB.",
                soc_name
            ),
        });
    }

    ensure_serial_port_unlocked(&clean)?;

    let mut port = serialport::new(&clean, baud_rate)
        .timeout(Duration::from_millis(1500))
        .open()
        .map_err(|e| format!("Falha ao abrir porta serial {}: {}", clean, e))?;

    let _ = port.write_all(b"\r\nPING\r\n");
    let _ = port.flush();
    let response_text =
        read_serial_response_complete(&mut *port, Duration::from_millis(800)).unwrap_or_default();

    let _ = port.write_all(b"STATUS\r\n");
    let _ = port.flush();
    let vdd_text =
        read_serial_response_complete(&mut *port, Duration::from_millis(800)).unwrap_or_default();

    let mut vdd_volts = 0.0f32; // 0.0 = unknown / no reading yet
    for token in vdd_text.split_whitespace() {
        if let Ok(v) = token
            .trim_end_matches('V')
            .trim_end_matches('v')
            .parse::<f32>()
        {
            vdd_volts = v;
            break;
        }
    }

    // Floating ADC ceiling: GP26 unloaded biases toward AVDD (3.3 V).
    // Treat readings >= 3.08 V with no probe as "disconnected", NOT "powered".
    const VDD_FLOATING_CEIL: f32 = 3.08;
    const VDD_MIN_THRESHOLD: f32 = 2.8;
    let probe_floating = vdd_volts >= VDD_FLOATING_CEIL;
    let target_powered = !probe_floating && vdd_volts >= VDD_MIN_THRESHOLD;

    let is_pico2 = response_text.contains("RP2350") || response_text.contains("Pico 2");
    let dev_label = if is_pico2 {
        "Raspberry Pi Pico 2 (RP2350)"
    } else {
        "Raspberry Pi Pico (RP2040)"
    };

    let banner = if response_text.is_empty() {
        format!("{} Conectado em {}", dev_label, clean)
    } else {
        response_text.trim().to_string()
    };

    let vdd_msg = if probe_floating {
        format!(
            "ALERTA: VDD={:.2}V (ADC flutuante — cabo CN55001 NÃO conectado ou resistor de 1kΩ ausente em GP26).",
            vdd_volts
        )
    } else if !target_powered {
        format!(
            "ALERTA: VDD={:.2}V abaixo do limiar. Gravadora desligada ou cabo ruim.",
            vdd_volts
        )
    } else {
        format!(
            "{} operacional na porta. VDD={:.2}V OK.",
            dev_label, vdd_volts
        )
    };

    Ok(PicoStatusResponse {
        connected: true,
        port_name: clean,
        baud_rate,
        vdd_voltage: vdd_volts,
        target_powered,
        firmware_banner: banner,
        is_mock: false,
        message: vdd_msg,
    })
}

static RAM_LOADER_INJECTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static SCAN_TOTAL_PROBES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static SCAN_HITS_DETECTED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Manages state tracking for RAM loader injection in the current process.
pub fn set_ram_loader_injected(injected: bool) {
    RAM_LOADER_INJECTED.store(injected, std::sync::atomic::Ordering::SeqCst);
}

/// Checks whether the in-circuit RAM loader has been injected during this session.
pub fn is_ram_loader_injected() -> bool {
    RAM_LOADER_INJECTED.load(std::sync::atomic::Ordering::SeqCst)
}

/// Retrieves full `PicoDeviceStatusDto` matching the React Frontend `PicoDeviceStatus` interface.
pub fn pico_get_device_status_dto(
    port_name: String,
    mode_override: Option<u8>,
) -> Result<PicoDeviceStatusDto, String> {
    // If no port name provided, return a disconnected status DTO.
    if port_name.trim().is_empty() {
        return Ok(PicoDeviceStatusDto {
            connected: false,
            port: "".to_string(),
            baud_rate: 0,
            vdd_volts: 0.0,
            target_powered: false,
            breakin_mode: 2,
            core_state: "DISCONNECTED".to_string(),
            bus_freq_hz: 0,
            firmware_version: "".to_string(),
            chip_id: "".to_string(),
            total_probes: 0,
            hits_detected: 0,
            open_bus_high: 0,
            open_bus_low: 0,
            ram_loader_injected: false,
            last_keepalive: "".to_string(),
        });
    }
    let status = pico_connect_and_test(port_name.clone(), 115_200)?;
    let is_rp2350 = status.port_name.contains("2350") || status.firmware_banner.contains("2350");

    let probes = if status.is_mock {
        1024
    } else {
        SCAN_TOTAL_PROBES.load(std::sync::atomic::Ordering::SeqCst)
    };
    let hits = if status.is_mock {
        12
    } else {
        SCAN_HITS_DETECTED.load(std::sync::atomic::Ordering::SeqCst)
    };
    let open_high = if status.is_mock { 1012 } else { 0 };

    let firmware_ver = if !status.firmware_banner.is_empty() {
        status.firmware_banner.clone()
    } else {
        "v2.0-PIO-DWire32A".to_string()
    };

    Ok(PicoDeviceStatusDto {
        connected: status.connected,
        port: status.port_name,
        baud_rate: status.baud_rate,
        vdd_volts: status.vdd_voltage,
        target_powered: status.target_powered,
        breakin_mode: mode_override.unwrap_or(2),
        core_state: if !status.connected {
            "DISCONNECTED".to_string()
        } else if !status.target_powered {
            "DISCONNECTED".to_string()
        } else {
            "HALTED".to_string()
        },
        bus_freq_hz: 1_000_000,
        firmware_version: firmware_ver,
        chip_id: if is_rp2350 {
            "RP2350-A2".to_string()
        } else {
            "RP2040-B2".to_string()
        },
        total_probes: probes,
        hits_detected: hits,
        open_bus_high: open_high,
        open_bus_low: 0,
        ram_loader_injected: is_ram_loader_injected(),
        last_keepalive: "OK".to_string(),
    })
}

/// Sends an arbitrary interactive command to the USB CDC serial port and returns the response.
pub fn pico_send_raw_serial_command(port_name: String, command: String) -> Result<String, String> {
    let clean = extract_port_name(&port_name);
    let trimmed = command.trim().to_uppercase();
    if clean.starts_with("MOCK") {
        let reply = match trimmed.as_str() {
            "PING" => "PONG (RP2040/RP2350 DWire32A Ready)".to_string(),
            "STATUS" | "VDD" => "VDD=3.28V | OCD=HALTED | PC=0x80000000 | MODE=2".to_string(),
            "HANDSHAKE" => {
                "DCSR: 0x00000001\n[AUTODETECT] Passive Address Aliasing Probe: 8 MB NOR Flash detected (EEMUL NVRAM @ 0x807F0000).\n[HANDSHAKE] SUCCESS! Matsushita AM34 core is HALTED in On-Chip Debug mode (Mode 2).".to_string()
            }
            "AUTODETECT" => {
                "AUTODETECT: OK (8 MB NOR Flash | EEMUL NVRAM: 0x807F0000..0x807FFFFF)".to_string()
            }
            "RESET" => "D_RESET pulsed low for 10ms. Target reset asserted.".to_string(),
            "ADC" => "ADC0 Raw: 4072 (3.281 V). Rail within tolerance (+-5%).".to_string(),
            "GDB" => "GDB Remote Serial Protocol mode active.".to_string(),
            "ISOLATE" | "HIZ" => {
                set_ram_loader_injected(false);
                "ISOLATE: OK (All 6 pins in High-Impedance Hi-Z mode; unified JST ZR-6 plug is electrically invisible)".to_string()
            }
            "ENGAGE" => {
                "ENGAGE: OK (Active PIO OCD bus drive restored on 6-pin connector)".to_string()
            }
            _ => format!("ACK: {} (Instrução executada no PIO0 @ 0x00004000)", trimmed),
        };
        return Ok(reply);
    }

    ensure_serial_port_unlocked(&clean)?;

    if trimmed == "ISOLATE" || trimmed == "HIZ" {
        set_ram_loader_injected(false);
    }

    let mut port = serialport::new(&clean, 115_200)
        .timeout(Duration::from_millis(2000))
        .open()
        .map_err(|e| format!("Falha ao abrir porta serial {}: {}", clean, e))?;

    let payload = format!("{}\r\n", trimmed);
    port.write_all(payload.as_bytes())
        .map_err(|e| format!("Falha ao transmitir comando CDC: {}", e))?;
    let _ = port.flush();

    let response = read_serial_response_complete(&mut *port, Duration::from_millis(8000))?;

    if trimmed.starts_with("SCAN") {
        if let Some(pos) = response.find("Hits: ") {
            let after = &response[pos + 6..];
            if let Some(comma) = after.find(',') {
                if let Ok(h) = after[..comma].trim().parse::<u32>() {
                    SCAN_HITS_DETECTED.store(h, std::sync::atomic::Ordering::SeqCst);
                }
            }
        }
        if let Some(pos) = response.find("Probes: ") {
            let after = &response[pos + 8..];
            let end_pos = after.find(')').unwrap_or(after.len());
            if let Ok(p) = after[..end_pos].trim().parse::<u32>() {
                SCAN_TOTAL_PROBES.store(p, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    Ok(response)
}

/// Executes Cold Break-In sequence on the DMR-EH55 via RP2040 PIO state machine.
pub fn pico_execute_cold_break_in(port_name: String, mode: u8) -> Result<String, String> {
    let clean = extract_port_name(&port_name);
    if clean.starts_with("MOCK") {
        let (mode_name, extrg_state) = match mode {
            0 => ("Mode 0: EXTRG High, Pre-Reset Only", "HIGH"),
            1 => ("Mode 1: EXTRG Low, Pre-Reset Only", "LOW"),
            2 => ("Mode 2: EXTRG High, Pre + Post Burst", "HIGH"),
            3 => ("Mode 3: EXTRG Low, Pre + Post Burst", "LOW"),
            _ => ("Mode Custom", "HIGH"),
        };
        let log = format!(
            r#"[Cold Break-In] Executando rotina de captura de inicialização ({mode_name})...
[Hardware] nRESET colocado em nível BAIXO (D_RESET assertado no barramento AM34).
[Hardware] Amostragem de VDD via ADC0 (GP26): 3.32 V (Tensão da Digital Board nominal).
[Hardware] Linha EXTRG (GP2) configurada em {extrg_state}. Habilitando PIO0 SM0.
[Hardware] nRESET liberado para Hi-Z com pull-up externo na placa VEP79138.
[Sincronismo] Transmitindo 32 ciclos de clock em SCLK (GP3) @ 1.0 MHz...
[Sincronismo] Amostrando resposta do registrador de status OCD em SDATA (GP4)...
[OCD Hit] Resposta capturada: 0x80000000 (Vetor de Inicialização Matsushita MN2DS0016).
[Status] CPU Interceptada com Sucesso! Núcleo AM34 congelado em estado de depuração.
[Pronto] Barramento DWire32A aberto para injeção do RAM Loader em 0x00004000."#
        );
        return Ok(log);
    }

    ensure_serial_port_unlocked(&clean)?;

    let mut port = serialport::new(&clean, 115200)
        .timeout(Duration::from_millis(3000))
        .open()
        .map_err(|e| format!("Falha ao conectar na porta serial {}: {}", clean, e))?;

    let cmd = format!("BREAK_IN {}\r\n", mode);
    port.write_all(cmd.as_bytes())
        .map_err(|e| format!("Falha ao enviar comando BREAK_IN: {}", e))?;
    let _ = port.flush();

    read_serial_response_complete(&mut *port, Duration::from_millis(3000))
}

/// Switches the Pico firmware into GDB Remote Serial Protocol (RSP) bridge mode.
pub fn pico_switch_to_gdb_mode(port_name: String) -> Result<String, String> {
    let clean = extract_port_name(&port_name);
    if clean.starts_with("MOCK") {
        return Ok("[GDB Bridge] Modo GDB RSP ativado no Simulador de Bancada. Pronto para conexão do GNU Debugger (mn10300-elf-gdb).".to_string());
    }

    ensure_serial_port_unlocked(&clean)?;

    let mut port = serialport::new(&clean, 115200)
        .timeout(Duration::from_millis(2000))
        .open()
        .map_err(|e| format!("Falha ao conectar na porta serial {}: {}", clean, e))?;

    port.write_all(b"GDB\r\n")
        .map_err(|e| format!("Falha ao enviar comando GDB: {}", e))?;
    let _ = port.flush();

    read_serial_response_complete(&mut *port, Duration::from_millis(2000))
}
