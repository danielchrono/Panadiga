//! Panasonic DMR-EH55 DIGA Suite - Pico USB Serial Driver
//!
//! Manages low-level USB CDC communication, device enumeration, Cold Break-In
//! sequencing, and in-circuit NVRAM erasure with the Raspberry Pi Pico (RP2040 / RP2350).

use std::io::{Read, Write};
use std::time::Duration;
use serialport::SerialPortType;

use super::types::{extract_port_name, PicoStatusResponse};

/// Lists all available serial COM ports, tagging recognized RP2040 and RP2350 hardware,
/// and includes high-fidelity bench simulator options.
pub fn list_pico_serial_ports() -> Result<Vec<String>, String> {
    let mut ports = Vec::new();
    if let Ok(available) = serialport::available_ports() {
        for p in available {
            let mut label = p.port_name.clone();
            if let SerialPortType::UsbPort(info) = &p.port_type {
                let (_plat, desc) = panadiga::pico::classify_usb_device(info.vid, info.pid, info.product.as_deref());
                label = format!("{} [{} - VID:{:04X} PID:{:04X}]", p.port_name, desc, info.vid, info.pid);
            }
            ports.push(label);
        }
    }
    // Multi-platform bench simulator options for development and headless verification
    ports.push("MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]".to_string());
    ports.push("MOCK_RP2350 [RP2350 Pico 2 DWire32A Bench Simulator - 520KB SRAM]".to_string());
    ports.push("MOCK_RP2350_RISCV [RP2350 Hazard3 RISC-V Bench Simulator - 520KB SRAM]".to_string());
    Ok(ports)
}

/// Connects to a serial port (or initializes mock simulator) and retrieves device telemetry.
pub fn pico_connect_and_test(port_name: String, baud_rate: u32) -> Result<PicoStatusResponse, String> {
    let clean = extract_port_name(&port_name);
    if clean.starts_with("MOCK") {
        let is_riscv = clean.contains("RISCV") || port_name.contains("RISCV");
        let is_rp2350 = clean.contains("2350") || port_name.contains("2350");
        let (soc_name, sram_desc) = if is_riscv {
            ("RP2350 (Pico 2 RISC-V Hazard3)", "520 KB SRAM / 64 KB Cache Setor")
        } else if is_rp2350 {
            ("RP2350 (Pico 2 ARM Cortex-M33)", "520 KB SRAM / 64 KB Cache Setor")
        } else {
            ("RP2040 (Pico 1 Dual Cortex-M0+)", "264 KB SRAM / 8 KB Cache Microsetor")
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

    let mut port = serialport::new(&clean, baud_rate)
        .timeout(Duration::from_millis(1500))
        .open()
        .map_err(|e| format!("Falha ao abrir porta serial {}: {}", clean, e))?;

    // Probe with PING
    let _ = port.write_all(b"\r\nPING\r\n");
    let mut buffer = [0u8; 1024];
    let bytes_read = port.read(&mut buffer).unwrap_or(0);
    let response_text = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

    // Query VDD voltage
    let _ = port.write_all(b"STATUS\r\n");
    let mut vdd_buf = [0u8; 512];
    let vdd_read = port.read(&mut vdd_buf).unwrap_or(0);
    let vdd_text = String::from_utf8_lossy(&vdd_buf[..vdd_read]).to_string();

    let mut vdd_volts = 3.30f32;
    for token in vdd_text.split_whitespace() {
        if let Ok(v) = token.trim_end_matches('V').trim_end_matches('v').parse::<f32>() {
            vdd_volts = v;
            break;
        }
    }

    let is_pico2 = response_text.contains("RP2350") || response_text.contains("Pico 2");
    let dev_label = if is_pico2 { "Raspberry Pi Pico 2 (RP2350)" } else { "Raspberry Pi Pico (RP2040)" };

    let banner = if response_text.is_empty() {
        format!("{} Conectado em {}", dev_label, clean)
    } else {
        response_text.trim().to_string()
    };

    Ok(PicoStatusResponse {
        connected: true,
        port_name: clean,
        baud_rate,
        vdd_voltage: vdd_volts,
        target_powered: vdd_volts >= 2.8,
        firmware_banner: banner,
        is_mock: false,
        message: format!("{} operacional na porta. VDD detectado: {:.2}V.", dev_label, vdd_volts),
    })
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
[Sincronismo] Transmitindo 32 ciclos de clock em SCLK (GP3) @ 2.5 MHz...
[Sincronismo] Amostrando resposta do registrador de status OCD em SDATA (GP4)...
[OCD Hit] Resposta capturada: 0xA0000000 (Vetor de Inicialização Matsushita MN2DS0016).
[Status] CPU Interceptada com Sucesso! Núcleo AM34 congelado em estado de depuração.
[Pronto] Barramento DWire32A aberto para execução de varredura ou injeção de código na RAM."#
        );
        return Ok(log);
    }

    let mut port = serialport::new(&clean, 115200)
        .timeout(Duration::from_millis(3000))
        .open()
        .map_err(|e| format!("Falha ao conectar na porta serial {}: {}", clean, e))?;

    let cmd = format!("BREAK_IN {}\r\n", mode);
    port.write_all(cmd.as_bytes())
        .map_err(|e| format!("Falha ao enviar comando BREAK_IN: {}", e))?;

    let mut buffer = [0u8; 4096];
    let bytes_read = port.read(&mut buffer)
        .map_err(|e| format!("Falha aguardando resposta do Pico: {}", e))?;

    Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
}

/// Executes in-circuit surgical erase of Spansion S29JL064H NVRAM (EEMUL micro-sectors).
pub fn pico_execute_in_circuit_erase(port_name: String) -> Result<String, String> {
    let clean = extract_port_name(&port_name);
    if clean.starts_with("MOCK") {
        let log = r#"[In-Circuit Erase] INICIANDO APAGAMENTO CIRÚRGICO DA NVRAM SPANSION S29JL064H...
[Segurança] Ilha de teste de aceleração (WP#/ACC Pino 14): Nível lógico alto (3.3V) verificado.
[Segurança] Proteção de fábrica validada: Setores SA0 a SA133 (Bootloader e SO) permanecem bloqueados.
[Alvo] Limpeza focada exclusivamente nos 8 micro-setores EEMUL corrompidos (0x7F0000 - 0x7FFFFF).
--------------------------------------------------------------------------------
[+] Injetando rotina de controle JEDEC na memória RAM interna do MN2DS0016...
[+] Executando sequência JEDEC 6-Cycle Sector Erase nos 8 micro-setores:
    [1/8] SA134 (0x7F0000 - 0x7F1FFF | 8 KB): Polling DQ7=1, DQ5=0 -> 0xFFFF [OK]
    [2/8] SA135 (0x7F2000 - 0x7F3FFF | 8 KB): Polling DQ7=1, DQ5=0 -> 0xFFFF [OK]
    [3/8] SA136 (0x7F4000 - 0x7F5FFF | 8 KB): Polling DQ7=1, DQ5=0 -> 0xFFFF [OK]
    [4/8] SA137 (0x7F6000 - 0x7F7FFF | 8 KB): Polling DQ7=1, DQ5=0 -> 0xFFFF [OK]
    [5/8] SA138 (0x7F8000 - 0x7F9FFF | 8 KB): Polling DQ7=1, DQ5=0 -> 0xFFFF [OK]
    [6/8] SA139 (0x7FA000 - 0x7FBFFF | 8 KB): Polling DQ7=1, DQ5=0 -> 0xFFFF [OK]
    [7/8] SA140 (0x7FC000 - 0x7FDFFF | 8 KB): Polling DQ7=1, DQ5=0 -> 0xFFFF [OK]
    [8/8] SA141 (0x7FE000 - 0x7FFFFF | 8 KB): Polling DQ7=1, DQ5=0 -> 0xFFFF [OK]
--------------------------------------------------------------------------------
[Verificação Pós-Erase] Lendo partição EEMUL (65.536 bytes): 100% dos bytes confirmados em 0xFF.
[Criptografia / Checksum] Hash da partição limpa coincide com padrão de fábrica Spansion.
[SUCESSO] NVRAM restaurada com total integridade!
[Recomendação] O DMR-EH55 agora reinicializará e recriará a estrutura EEMUL limpa sem erro U99."#.to_string();
        return Ok(log);
    }

    let mut port = serialport::new(&clean, 115200)
        .timeout(Duration::from_millis(10000))
        .open()
        .map_err(|e| format!("Falha ao conectar na porta serial {}: {}", clean, e))?;

    port.write_all(b"ERASE_NVRAM\r\n")
        .map_err(|e| format!("Falha ao enviar comando ERASE_NVRAM: {}", e))?;

    let mut buffer = [0u8; 8192];
    let bytes_read = port.read(&mut buffer)
        .map_err(|e| format!("Falha aguardando conclusão do erase no Pico: {}", e))?;

    Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
}

/// Switches the Pico firmware into GDB Remote Serial Protocol (RSP) bridge mode.
pub fn pico_switch_to_gdb_mode(port_name: String) -> Result<String, String> {
    let clean = extract_port_name(&port_name);
    if clean.starts_with("MOCK") {
        return Ok("[GDB Bridge] Modo GDB RSP ativado no Simulador de Bancada. Pronto para conexão do GNU Debugger (mn10300-elf-gdb).".to_string());
    }

    let mut port = serialport::new(&clean, 115200)
        .timeout(Duration::from_millis(2000))
        .open()
        .map_err(|e| format!("Falha ao conectar na porta serial {}: {}", clean, e))?;

    port.write_all(b"GDB\r\n")
        .map_err(|e| format!("Falha ao enviar comando GDB: {}", e))?;

    let mut buffer = [0u8; 1024];
    let bytes_read = port.read(&mut buffer)
        .map_err(|e| format!("Falha aguardando confirmação do modo GDB: {}", e))?;

    Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
}
