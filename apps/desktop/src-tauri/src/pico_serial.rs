//! Panasonic DMR-EH55 DIGA Suite - RP2040 & RP2350 Serial Controller
//!
//! Provides bidirectional USB CDC communication with the Raspberry Pi Pico (RP2040)
//! and Raspberry Pi Pico 2 (RP2350) host controllers managing the Panasonic DWire32A
//! On-Chip Debug (OCD) interface on connector CN55001.
//!
//! Includes bench simulation / mock mode for testing without physical hardware.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serialport::SerialPortType;

/// Raspberry Pi USB Vendor ID
pub const RASPBERRY_PI_VID: u16 = 0x2E8A;
/// Standard Raspberry Pi Pico (RP2040) USB CDC Serial Product ID
pub const PICO_RP2040_CDC_PID: u16 = 0x000A;
/// Standard Raspberry Pi Pico 2 (RP2350) USB CDC Serial Product ID
pub const PICO_RP2350_CDC_PID: u16 = 0x000F;
/// Raspberry Pi BOOTSEL Mode Product ID
pub const PICO_BOOTSEL_PID: u16 = 0x0003;

/// Diagnostic connection and telemetry response from the Pico controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoStatusResponse {
    pub connected: bool,
    pub port_name: String,
    pub baud_rate: u32,
    pub vdd_voltage: f32,
    pub target_powered: bool,
    pub firmware_banner: String,
    pub is_mock: bool,
    pub message: String,
}

/// Normalizes port name string, extracting "COMx" or detecting mock mode.
pub fn extract_port_name(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.to_uppercase().contains("MOCK") {
        return "MOCK".to_string();
    }
    trimmed.split_whitespace().next().unwrap_or(trimmed).to_string()
}

/// Lists all available serial COM ports, tagging recognized RP2040 and RP2350 hardware,
/// and includes high-fidelity bench simulator options.
pub fn list_pico_serial_ports() -> Result<Vec<String>, String> {
    let mut ports = Vec::new();
    if let Ok(available) = serialport::available_ports() {
        for p in available {
            let mut label = p.port_name.clone();
            if let SerialPortType::UsbPort(info) = &p.port_type {
                if info.vid == RASPBERRY_PI_VID {
                    match info.pid {
                        PICO_RP2040_CDC_PID => {
                            label = format!("{} [Raspberry Pi Pico (RP2040) DWire32A - VID:2E8A PID:000A]", p.port_name);
                        }
                        PICO_RP2350_CDC_PID => {
                            label = format!("{} [Raspberry Pi Pico 2 (RP2350) DWire32A - VID:2E8A PID:000F]", p.port_name);
                        }
                        PICO_BOOTSEL_PID => {
                            label = format!("{} [Raspberry Pi BOOTSEL Mode - VID:2E8A PID:0003]", p.port_name);
                        }
                        other => {
                            let desc = info.product.as_deref().unwrap_or("Raspberry Pi Device");
                            label = format!("{} [{} - VID:2E8A PID:{:04X}]", p.port_name, desc, other);
                        }
                    }
                } else {
                    let desc = info.product.as_deref().unwrap_or("USB Serial Device");
                    label = format!("{} [{} - {:04X}:{:04X}]", p.port_name, desc, info.vid, info.pid);
                }
            }
            ports.push(label);
        }
    }
    // Always provide Bench Simulator options for both architectures
    ports.push("MOCK_RP2040 [RP2040 DWire32A Bench Simulator - 264KB SRAM]".to_string());
    ports.push("MOCK_RP2350 [RP2350 Pico 2 DWire32A Bench Simulator - 520KB SRAM]".to_string());
    Ok(ports)
}

/// Connects to a serial port (or initializes mock simulator) and retrieves device telemetry.
pub fn pico_connect_and_test(port_name: String, baud_rate: u32) -> Result<PicoStatusResponse, String> {
    let clean = extract_port_name(&port_name);
    if clean.starts_with("MOCK") {
        let is_rp2350 = clean.contains("2350") || port_name.contains("2350");
        let (soc_name, sram_desc) = if is_rp2350 {
            ("RP2350 (Pico 2)", "520 KB SRAM / 64 KB Cache Setor")
        } else {
            ("RP2040 (Pico 1)", "264 KB SRAM / 8 KB Cache Microsetor")
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
    if clean == "MOCK" {
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
    if clean == "MOCK" {
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

/// Current status of the GDB TCP-to-Serial bridge server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdbBridgeStatus {
    pub running: bool,
    pub port_name: String,
    pub listen_addr: String,
    pub connected_clients: u32,
    pub last_event: String,
}

/// AM34 CPU register file state for UI inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Am34RegisterState {
    pub d: [u32; 4],
    pub a: [u32; 4],
    pub mdr: u32,
    pub psw: u32,
    pub pc: u32,
    pub sp: u32,
    pub is_halted: bool,
}

struct GdbBridgeController {
    running: Arc<AtomicBool>,
    client_connected: Arc<AtomicBool>,
    client_count: Arc<AtomicU32>,
    port_name: Mutex<String>,
    listen_addr: Mutex<String>,
    last_event: Mutex<String>,
    thread_handle: Mutex<Option<thread::JoinHandle<()>>>,
    registers: Mutex<Am34RegisterState>,
}

static GDB_CONTROLLER: OnceLock<GdbBridgeController> = OnceLock::new();

fn get_controller() -> &'static GdbBridgeController {
    GDB_CONTROLLER.get_or_init(|| GdbBridgeController {
        running: Arc::new(AtomicBool::new(false)),
        client_connected: Arc::new(AtomicBool::new(false)),
        client_count: Arc::new(AtomicU32::new(0)),
        port_name: Mutex::new(String::new()),
        listen_addr: Mutex::new("127.0.0.1:3333".to_string()),
        last_event: Mutex::new("Servidor GDB offline".to_string()),
        thread_handle: Mutex::new(None),
        registers: Mutex::new(Am34RegisterState {
            d: [0x00000000, 0x00000000, 0x00000000, 0x00000000],
            a: [0x00000000, 0x00000000, 0x00000000, 0x0003FFF0],
            mdr: 0x00000000,
            psw: 0x00000100,
            pc: 0x80000000,
            sp: 0x0003FFF0,
            is_halted: true,
        }),
    })
}

/// Starts the GDB TCP-to-Serial bridge server in a background thread.
pub fn start_gdb_bridge(port_name: String, listen_addr: String) -> Result<GdbBridgeStatus, String> {
    let ctrl = get_controller();
    if ctrl.running.load(Ordering::Relaxed) {
        return get_gdb_bridge_status();
    }

    let clean_port = extract_port_name(&port_name);
    let addr = if listen_addr.trim().is_empty() {
        "127.0.0.1:3333".to_string()
    } else {
        listen_addr.trim().to_string()
    };

    ctrl.running.store(true, Ordering::Relaxed);
    ctrl.client_connected.store(false, Ordering::Relaxed);
    ctrl.client_count.store(0, Ordering::Relaxed);

    *ctrl.port_name.lock().unwrap() = clean_port.clone();
    *ctrl.listen_addr.lock().unwrap() = addr.clone();
    *ctrl.last_event.lock().unwrap() = format!("Ouvindo conexões GDB em {}", addr);

    let running_flag = ctrl.running.clone();
    let connected_flag = ctrl.client_connected.clone();
    let count_flag = ctrl.client_count.clone();
    let thread_port = clean_port.clone();
    let thread_addr = addr.clone();

    let handle = thread::spawn(move || {
        let listener = match TcpListener::bind(&thread_addr) {
            Ok(l) => {
                let _ = l.set_nonblocking(true);
                l
            }
            Err(e) => {
                eprintln!("[GDB Bridge] Falha ao vincular porta TCP {}: {}", thread_addr, e);
                running_flag.store(false, Ordering::Relaxed);
                return;
            }
        };

        if thread_port.starts_with("MOCK") {
            // Mock Simulation Loop
            while running_flag.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut tcp_stream, _peer)) => {
                        connected_flag.store(true, Ordering::Relaxed);
                        count_flag.fetch_add(1, Ordering::Relaxed);
                        let _ = tcp_stream.set_read_timeout(Some(Duration::from_millis(200)));

                        let mut buf = [0u8; 1024];
                        while running_flag.load(Ordering::Relaxed) && connected_flag.load(Ordering::Relaxed) {
                            match tcp_stream.read(&mut buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    let packet = String::from_utf8_lossy(&buf[..n]);
                                    if packet.contains('?') {
                                        let _ = tcp_stream.write_all(b"+$S05#b8");
                                    } else if packet.contains('g') {
                                        let mock_g = "+$00000000000000000000000000000000000000000000000000000000f0ff0300000000000001000000000080f0ff0300#44";
                                        let _ = tcp_stream.write_all(mock_g.as_bytes());
                                    } else if packet.contains("qSupported") {
                                        let _ = tcp_stream.write_all(b"+$PacketSize=800;qXfer:memory-map:read+#3f");
                                    } else if packet.contains('c') || packet.contains('s') {
                                        let _ = tcp_stream.write_all(b"+$S05#b8");
                                    } else if packet.contains('k') || packet.contains('D') {
                                        let _ = tcp_stream.write_all(b"+$OK#9a");
                                        break;
                                    } else {
                                        let _ = tcp_stream.write_all(b"+$#00");
                                    }
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                                    thread::sleep(Duration::from_millis(10));
                                }
                                Err(_) => break,
                            }
                        }
                        connected_flag.store(false, Ordering::Relaxed);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(_) => break,
                }
            }
        } else {
            // Hardware Serial Loop
            let mut serial_port = match serialport::new(&thread_port, 115200)
                .timeout(Duration::from_millis(100))
                .open()
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[GDB Bridge] Falha ao abrir porta serial {}: {}", thread_port, e);
                    running_flag.store(false, Ordering::Relaxed);
                    return;
                }
            };

            let _ = serial_port.write_all(b"\r\nGDB\r\n");

            while running_flag.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut tcp_stream, _peer)) => {
                        connected_flag.store(true, Ordering::Relaxed);
                        count_flag.fetch_add(1, Ordering::Relaxed);
                        let _ = tcp_stream.set_nonblocking(false);
                        let _ = tcp_stream.set_read_timeout(Some(Duration::from_millis(50)));

                        let mut s_buf = [0u8; 1024];
                        let mut t_buf = [0u8; 1024];

                        while running_flag.load(Ordering::Relaxed) && connected_flag.load(Ordering::Relaxed) {
                            // Read TCP -> Write Serial
                            match tcp_stream.read(&mut t_buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    let _ = serial_port.write_all(&t_buf[..n]);
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
                                Err(_) => break,
                            }

                            // Read Serial -> Write TCP
                            match serial_port.read(&mut s_buf) {
                                Ok(0) => {}
                                Ok(n) => {
                                    if tcp_stream.write_all(&s_buf[..n]).is_err() {
                                        break;
                                    }
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                                Err(_) => break,
                            }
                        }
                        connected_flag.store(false, Ordering::Relaxed);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(_) => break,
                }
            }
        }
    });

    *ctrl.thread_handle.lock().unwrap() = Some(handle);
    get_gdb_bridge_status()
}

/// Stops the GDB TCP-to-Serial bridge server.
pub fn stop_gdb_bridge() -> Result<GdbBridgeStatus, String> {
    let ctrl = get_controller();
    if !ctrl.running.load(Ordering::Relaxed) {
        return get_gdb_bridge_status();
    }

    ctrl.running.store(false, Ordering::Relaxed);
    ctrl.client_connected.store(false, Ordering::Relaxed);

    // Wake up listener if blocking
    let addr = ctrl.listen_addr.lock().unwrap().clone();
    let _ = TcpStream::connect_timeout(&addr.parse().unwrap_or("127.0.0.1:3333".parse().unwrap()), Duration::from_millis(100));

    if let Some(h) = ctrl.thread_handle.lock().unwrap().take() {
        let _ = h.join();
    }

    *ctrl.last_event.lock().unwrap() = "Servidor GDB finalizado".to_string();
    get_gdb_bridge_status()
}

/// Retrieves the current status and metrics of the GDB bridge.
pub fn get_gdb_bridge_status() -> Result<GdbBridgeStatus, String> {
    let ctrl = get_controller();
    Ok(GdbBridgeStatus {
        running: ctrl.running.load(Ordering::Relaxed),
        port_name: ctrl.port_name.lock().unwrap().clone(),
        listen_addr: ctrl.listen_addr.lock().unwrap().clone(),
        connected_clients: ctrl.client_count.load(Ordering::Relaxed),
        last_event: ctrl.last_event.lock().unwrap().clone(),
    })
}

/// Retrieves the current AM34 register state.
pub fn get_am34_registers() -> Result<Am34RegisterState, String> {
    let ctrl = get_controller();
    Ok(ctrl.registers.lock().unwrap().clone())
}
