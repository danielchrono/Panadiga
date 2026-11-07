//! Multi-Platform Host Controller (RP2040 / RP2350 / RP2 Family) Serial & GDB RSP Bridge Module
//!
//! Provides cross-platform hardware discovery, diagnostic telemetry probing, and a TCP-to-Serial
//! GDB Remote Serial Protocol (RSP) bridge connecting high-level debuggers
//! (mn10300-elf-gdb, gdb-multiarch, Ghidra, IDA Pro) to the Panasonic DWire32A OCD bus.
//! Supports Raspberry Pi Pico/Pico W, Pico 2 (ARM & RISC-V), Arduino Nano RP2040 Connect,
//! Seeed XIAO, Adafruit, SparkFun, and Waveshare RP2-family controllers across Windows, Linux, and macOS.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serialport::SerialPortType;

// ============================================================================
// Vendor & Product ID Constants for Multi-Platform RP2 Family Controllers
// ============================================================================

/// Raspberry Pi USB Vendor ID
pub const RASPBERRY_PI_VID: u16 = 0x2E8A;
/// Standard Raspberry Pi Pico (RP2040) USB CDC Serial Product ID
pub const PICO_RP2040_CDC_PID: u16 = 0x000A;
/// Raspberry Pi Pico W (RP2040 + CYW43439) USB CDC Serial Product ID
pub const PICO_RP2040_W_CDC_PID: u16 = 0x0014;
/// Standard Raspberry Pi Pico 2 (RP2350 ARM Cortex-M33) USB CDC Serial Product ID
pub const PICO_RP2350_CDC_PID: u16 = 0x000F;
/// Raspberry Pi Pico 2 W (RP2350 + Wireless) USB CDC Serial Product ID
pub const PICO_RP2350_W_CDC_PID: u16 = 0x001F;
/// Raspberry Pi Pico 2 (RP2350 Hazard3 RISC-V) USB CDC Serial Product ID
pub const PICO_RP2350_RISCV_CDC_PID: u16 = 0x000E;
/// Raspberry Pi BOOTSEL Mode Product ID
pub const PICO_BOOTSEL_PID: u16 = 0x0003;

/// Arduino USB Vendor ID (Arduino Nano RP2040 Connect)
pub const ARDUINO_VID: u16 = 0x2341;
pub const ARDUINO_NANO_RP2040_CONNECT_PID: u16 = 0x005E;
pub const ARDUINO_NANO_RP2040_BOOT_PID: u16 = 0x805E;

/// Seeed Studio USB Vendor ID (XIAO RP2040 / XIAO RP2350)
pub const SEEED_VID: u16 = 0x2886;
pub const SEEED_XIAO_RP2040_PID: u16 = 0x0042;
pub const SEEED_XIAO_RP2040_BOOT_PID: u16 = 0x8042;
pub const SEEED_XIAO_RP2350_PID: u16 = 0x0044;

/// Adafruit Industries USB Vendor ID (Feather, QT Py, Trinkey RP2040/RP2350)
pub const ADAFRUIT_VID: u16 = 0x239A;

/// SparkFun Electronics USB Vendor ID (Pro Micro, Thing Plus RP2040/RP2350)
pub const SPARKFUN_VID: u16 = 0x1B4F;

/// Waveshare / OpenMoko / TinyUSB Generic USB Vendor ID
pub const WAVESHARE_TINYUSB_VID: u16 = 0x1209;

// ============================================================================
// Architecture and Platform Classification
// ============================================================================

/// Architecture classification of connected Pico controller (Backwards-compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PicoArchitecture {
    Rp2040,
    Rp2350,
    Bootloader,
    Unknown,
}

impl std::fmt::Display for PicoArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rp2040 => write!(f, "Raspberry Pi Pico (RP2040)"),
            Self::Rp2350 => write!(f, "Raspberry Pi Pico 2 (RP2350)"),
            Self::Bootloader => write!(f, "Raspberry Pi BOOTSEL Mode"),
            Self::Unknown => write!(f, "Unknown USB Serial Device"),
        }
    }
}

/// Detailed multi-platform host controller hardware classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostControllerPlatform {
    /// Official Raspberry Pi Pico (RP2040 Dual Cortex-M0+ @ 133MHz, 264KB SRAM)
    RaspberryPiPico,
    /// Official Raspberry Pi Pico W (RP2040 + Wi-Fi)
    RaspberryPiPicoW,
    /// Official Raspberry Pi Pico 2 (RP2350 Dual Cortex-M33 @ 150MHz, 520KB SRAM)
    RaspberryPiPico2Arm,
    /// Official Raspberry Pi Pico 2 (RP2350 Dual Hazard3 RISC-V @ 150MHz, 520KB SRAM)
    RaspberryPiPico2Riscv,
    /// Official Raspberry Pi Pico 2 W
    RaspberryPiPico2W,
    /// Arduino Nano RP2040 Connect
    ArduinoNanoRp2040,
    /// Seeed Studio XIAO RP2040
    SeeedXiaoRp2040,
    /// Seeed Studio XIAO RP2350
    SeeedXiaoRp2350,
    /// Adafruit RP2040 Board (Feather, QT Py, ItsyBitsy)
    AdafruitRp2040,
    /// Adafruit RP2350 Board (Feather, Metro)
    AdafruitRp2350,
    /// SparkFun RP2040 Board (Pro Micro)
    SparkfunRp2040,
    /// SparkFun RP2350 Board
    SparkfunRp2350,
    /// Waveshare RP2040 Board (Zero, One, Plus)
    WaveshareRp2040,
    /// Waveshare RP2350 Board (Plus, ETH)
    WaveshareRp2350,
    /// Generic RP2040 Microcontroller
    GenericRp2040,
    /// Generic RP2350 Microcontroller
    GenericRp2350,
    /// Generic USB CDC Device Running Panadiga DWire32A Firmware
    GenericCompatibleCdc,
    /// RP2-family BOOTSEL UF2 Bootloader Mode
    BootloaderMode,
    /// Unknown Serial Device
    UnknownDevice,
}

impl HostControllerPlatform {
    /// Maps platform to high-level architecture category
    pub fn architecture(&self) -> PicoArchitecture {
        match self {
            Self::RaspberryPiPico
            | Self::RaspberryPiPicoW
            | Self::ArduinoNanoRp2040
            | Self::SeeedXiaoRp2040
            | Self::AdafruitRp2040
            | Self::SparkfunRp2040
            | Self::WaveshareRp2040
            | Self::GenericRp2040 => PicoArchitecture::Rp2040,

            Self::RaspberryPiPico2Arm
            | Self::RaspberryPiPico2Riscv
            | Self::RaspberryPiPico2W
            | Self::SeeedXiaoRp2350
            | Self::AdafruitRp2350
            | Self::SparkfunRp2350
            | Self::WaveshareRp2350
            | Self::GenericRp2350 => PicoArchitecture::Rp2350,

            Self::BootloaderMode => PicoArchitecture::Bootloader,
            Self::GenericCompatibleCdc | Self::UnknownDevice => PicoArchitecture::Unknown,
        }
    }

    /// Returns human-readable silicon processor family and frequency
    pub fn chip_family(&self) -> &'static str {
        match self.architecture() {
            PicoArchitecture::Rp2040 => "RP2040 (Dual ARM Cortex-M0+ @ 133 MHz)",
            PicoArchitecture::Rp2350 => match self {
                Self::RaspberryPiPico2Riscv => "RP2350 (Dual Hazard3 RISC-V @ 150 MHz)",
                _ => "RP2350 (Dual ARM Cortex-M33 @ 150 MHz)",
            },
            PicoArchitecture::Bootloader => "RP2 Family Bootloader (ROM UF2)",
            PicoArchitecture::Unknown => "Universal Microcontroller",
        }
    }

    /// Total on-chip SRAM capacity in bytes
    pub fn sram_size_bytes(&self) -> u32 {
        match self.architecture() {
            PicoArchitecture::Rp2040 => 264 * 1024, // 264 KB
            PicoArchitecture::Rp2350 => 520 * 1024, // 520 KB
            _ => 128 * 1024,
        }
    }

    /// Flash sector cache capacity allocated in firmware for DWire32A operations
    pub fn sector_cache_capacity_bytes(&self) -> u32 {
        match self.architecture() {
            PicoArchitecture::Rp2040 => 8 * 1024,   // 8 KB microsector cache
            PicoArchitecture::Rp2350 => 64 * 1024,  // 64 KB full sector cache
            _ => 4 * 1024,
        }
    }

    /// True if the platform is known to natively support the DWire32A PIO state machine
    pub fn supports_dwire32a_pio(&self) -> bool {
        matches!(
            self.architecture(),
            PicoArchitecture::Rp2040 | PicoArchitecture::Rp2350
        )
    }
}

impl std::fmt::Display for HostControllerPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RaspberryPiPico => write!(f, "Raspberry Pi Pico (RP2040)"),
            Self::RaspberryPiPicoW => write!(f, "Raspberry Pi Pico W (RP2040)"),
            Self::RaspberryPiPico2Arm => write!(f, "Raspberry Pi Pico 2 ARM (RP2350)"),
            Self::RaspberryPiPico2Riscv => write!(f, "Raspberry Pi Pico 2 RISC-V (RP2350)"),
            Self::RaspberryPiPico2W => write!(f, "Raspberry Pi Pico 2 W (RP2350)"),
            Self::ArduinoNanoRp2040 => write!(f, "Arduino Nano RP2040 Connect"),
            Self::SeeedXiaoRp2040 => write!(f, "Seeed Studio XIAO RP2040"),
            Self::SeeedXiaoRp2350 => write!(f, "Seeed Studio XIAO RP2350"),
            Self::AdafruitRp2040 => write!(f, "Adafruit RP2040 Controller"),
            Self::AdafruitRp2350 => write!(f, "Adafruit RP2350 Controller"),
            Self::SparkfunRp2040 => write!(f, "SparkFun RP2040 Pro Micro"),
            Self::SparkfunRp2350 => write!(f, "SparkFun RP2350 Controller"),
            Self::WaveshareRp2040 => write!(f, "Waveshare RP2040 Board"),
            Self::WaveshareRp2350 => write!(f, "Waveshare RP2350 Board"),
            Self::GenericRp2040 => write!(f, "Generic RP2040 Microcontroller"),
            Self::GenericRp2350 => write!(f, "Generic RP2350 Microcontroller"),
            Self::GenericCompatibleCdc => write!(f, "Compatible USB CDC DWire32A Probe"),
            Self::BootloaderMode => write!(f, "RP2 Family BOOTSEL UF2 Mode"),
            Self::UnknownDevice => write!(f, "Unknown USB Serial Device"),
        }
    }
}

/// Metadata and hardware attributes of a detected serial port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoDeviceInfo {
    pub port_name: String,
    pub architecture: PicoArchitecture,
    pub platform: HostControllerPlatform,
    pub vid: u16,
    pub pid: u16,
    pub description: String,
}

impl PicoDeviceInfo {
    /// Silicon chip family description
    pub fn chip_family(&self) -> &'static str {
        self.platform.chip_family()
    }

    /// On-chip SRAM size in bytes
    pub fn sram_size_bytes(&self) -> u32 {
        self.platform.sram_size_bytes()
    }

    /// Flash sector cache capacity in firmware
    pub fn sector_cache_capacity_bytes(&self) -> u32 {
        self.platform.sector_cache_capacity_bytes()
    }

    /// True if device belongs to the RP2 family (RP2040 or RP2350)
    pub fn is_rp2_family(&self) -> bool {
        self.platform.supports_dwire32a_pio()
    }
}

// ============================================================================
// Multi-Platform Device Enumeration and Discovery
// ============================================================================

/// Classifies a USB device based on its VID, PID, and product description
pub fn classify_usb_device(vid: u16, pid: u16, product_desc: Option<&str>) -> (HostControllerPlatform, String) {
    let desc_hint = product_desc.unwrap_or("").to_lowercase();

    match vid {
        RASPBERRY_PI_VID => match pid {
            PICO_RP2040_CDC_PID => (
                HostControllerPlatform::RaspberryPiPico,
                "Raspberry Pi Pico (RP2040) DWire32A Host (264KB SRAM)".to_string(),
            ),
            PICO_RP2040_W_CDC_PID => (
                HostControllerPlatform::RaspberryPiPicoW,
                "Raspberry Pi Pico W (RP2040) DWire32A Host".to_string(),
            ),
            PICO_RP2350_CDC_PID => (
                HostControllerPlatform::RaspberryPiPico2Arm,
                "Raspberry Pi Pico 2 ARM (RP2350) DWire32A Host (520KB SRAM)".to_string(),
            ),
            PICO_RP2350_RISCV_CDC_PID => (
                HostControllerPlatform::RaspberryPiPico2Riscv,
                "Raspberry Pi Pico 2 RISC-V (RP2350) DWire32A Host".to_string(),
            ),
            PICO_RP2350_W_CDC_PID => (
                HostControllerPlatform::RaspberryPiPico2W,
                "Raspberry Pi Pico 2 W (RP2350) DWire32A Host".to_string(),
            ),
            PICO_BOOTSEL_PID => (
                HostControllerPlatform::BootloaderMode,
                "Raspberry Pi BOOTSEL UF2 Mode (Ready for firmware flash)".to_string(),
            ),
            other => {
                if desc_hint.contains("rp2350") || desc_hint.contains("pico 2") {
                    (
                        HostControllerPlatform::GenericRp2350,
                        format!("Raspberry Pi RP2350 Custom PID 0x{:04X}", other),
                    )
                } else if desc_hint.contains("rp2040") || desc_hint.contains("pico") {
                    (
                        HostControllerPlatform::GenericRp2040,
                        format!("Raspberry Pi RP2040 Custom PID 0x{:04X}", other),
                    )
                } else {
                    (
                        HostControllerPlatform::GenericCompatibleCdc,
                        format!("Raspberry Pi Device PID 0x{:04X}", other),
                    )
                }
            }
        },

        ARDUINO_VID => match pid {
            ARDUINO_NANO_RP2040_CONNECT_PID => (
                HostControllerPlatform::ArduinoNanoRp2040,
                "Arduino Nano RP2040 Connect (DWire32A Compatible)".to_string(),
            ),
            ARDUINO_NANO_RP2040_BOOT_PID => (
                HostControllerPlatform::BootloaderMode,
                "Arduino Nano RP2040 Connect (BOOTSEL Mode)".to_string(),
            ),
            _ if desc_hint.contains("rp2040") => (
                HostControllerPlatform::ArduinoNanoRp2040,
                "Arduino RP2040 Board".to_string(),
            ),
            _ => (
                HostControllerPlatform::UnknownDevice,
                format!("Arduino Device PID 0x{:04X}", pid),
            ),
        },

        SEEED_VID => match pid {
            SEEED_XIAO_RP2040_PID => (
                HostControllerPlatform::SeeedXiaoRp2040,
                "Seeed Studio XIAO RP2040 (DWire32A Compatible)".to_string(),
            ),
            SEEED_XIAO_RP2040_BOOT_PID => (
                HostControllerPlatform::BootloaderMode,
                "Seeed Studio XIAO RP2040 (BOOTSEL Mode)".to_string(),
            ),
            SEEED_XIAO_RP2350_PID => (
                HostControllerPlatform::SeeedXiaoRp2350,
                "Seeed Studio XIAO RP2350 (DWire32A Compatible)".to_string(),
            ),
            _ if desc_hint.contains("rp2350") => (
                HostControllerPlatform::SeeedXiaoRp2350,
                "Seeed Studio RP2350 Board".to_string(),
            ),
            _ if desc_hint.contains("rp2040") => (
                HostControllerPlatform::SeeedXiaoRp2040,
                "Seeed Studio RP2040 Board".to_string(),
            ),
            _ => (
                HostControllerPlatform::UnknownDevice,
                format!("Seeed Device PID 0x{:04X}", pid),
            ),
        },

        ADAFRUIT_VID => {
            if desc_hint.contains("rp2350") {
                (
                    HostControllerPlatform::AdafruitRp2350,
                    "Adafruit RP2350 Controller".to_string(),
                )
            } else if desc_hint.contains("rp2040") || pid >= 0x80F0 {
                (
                    HostControllerPlatform::AdafruitRp2040,
                    "Adafruit RP2040 Controller (Feather/QT Py)".to_string(),
                )
            } else {
                (
                    HostControllerPlatform::UnknownDevice,
                    format!("Adafruit Device PID 0x{:04X}", pid),
                )
            }
        }

        SPARKFUN_VID => {
            if desc_hint.contains("rp2350") {
                (
                    HostControllerPlatform::SparkfunRp2350,
                    "SparkFun RP2350 Board".to_string(),
                )
            } else if desc_hint.contains("rp2040") || pid == 0x0025 || pid == 0x0026 {
                (
                    HostControllerPlatform::SparkfunRp2040,
                    "SparkFun Pro Micro RP2040".to_string(),
                )
            } else {
                (
                    HostControllerPlatform::UnknownDevice,
                    format!("SparkFun Device PID 0x{:04X}", pid),
                )
            }
        }

        WAVESHARE_TINYUSB_VID => {
            if desc_hint.contains("rp2350") {
                (
                    HostControllerPlatform::WaveshareRp2350,
                    "Waveshare RP2350 Board".to_string(),
                )
            } else if desc_hint.contains("rp2040") || desc_hint.contains("dwire32") {
                (
                    HostControllerPlatform::WaveshareRp2040,
                    "Waveshare RP2040 / TinyUSB DWire32A Probe".to_string(),
                )
            } else {
                (
                    HostControllerPlatform::GenericCompatibleCdc,
                    "USB CDC Serial Device (TinyUSB)".to_string(),
                )
            }
        }

        _ => {
            if desc_hint.contains("rp2350") {
                (
                    HostControllerPlatform::GenericRp2350,
                    format!("RP2350 Controller (VID: {:04X} PID: {:04X})", vid, pid),
                )
            } else if desc_hint.contains("rp2040") {
                (
                    HostControllerPlatform::GenericRp2040,
                    format!("RP2040 Controller (VID: {:04X} PID: {:04X})", vid, pid),
                )
            } else if desc_hint.contains("dwire32") || desc_hint.contains("panadiga") {
                (
                    HostControllerPlatform::GenericCompatibleCdc,
                    "Panadiga DWire32A Compatible Probe".to_string(),
                )
            } else {
                (
                    HostControllerPlatform::UnknownDevice,
                    product_desc.unwrap_or("USB Serial Device").to_string(),
                )
            }
        }
    }
}

/// Discovers and enumerates all serial ports across Windows, Linux, and macOS,
/// classifying RP2040, RP2350, and compatible DWire32A hardware controllers.
pub fn list_pico_devices() -> Result<Vec<PicoDeviceInfo>> {
    let mut devices = Vec::new();
    let ports = serialport::available_ports().context("Failed to list serial ports on system")?;

    for p in ports {
        if let SerialPortType::UsbPort(info) = p.port_type {
            let (platform, desc) = classify_usb_device(info.vid, info.pid, info.product.as_deref());
            let arch = platform.architecture();

            devices.push(PicoDeviceInfo {
                port_name: p.port_name,
                architecture: arch,
                platform,
                vid: info.vid,
                pid: info.pid,
                description: desc,
            });
        }
    }

    Ok(devices)
}

// ============================================================================
// Telemetry and Connection Diagnostics
// ============================================================================

/// Probes a serial port to test connection and query target 3.3V VDD rail telemetry
pub fn ping_pico(port_name: &str, baud_rate: u32) -> Result<String> {
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(1500))
        .open()
        .with_context(|| format!("Failed to open serial port '{}'", port_name))?;

    // Send newline and PING
    port.write_all(b"\r\nPING\r\n")
        .context("Failed to write PING probe to serial port")?;
    thread::sleep(Duration::from_millis(100));

    let mut buf = [0u8; 1024];
    let n = port.read(&mut buf).unwrap_or(0);
    let banner = String::from_utf8_lossy(&buf[..n]).trim().to_string();

    // Query VDD voltage
    let _ = port.write_all(b"STATUS\r\n");
    thread::sleep(Duration::from_millis(100));
    let n_vdd = port.read(&mut buf).unwrap_or(0);
    let vdd_resp = String::from_utf8_lossy(&buf[..n_vdd]).trim().to_string();

    Ok(format!(
        "--- Host Controller Telemetry Report ---\n\
         Port: {}\n\
         Baud: {}\n\
         Banner:\n{}\n\
         Telemetry:\n{}\n",
        port_name, baud_rate, banner, vdd_resp
    ))
}

// ============================================================================
// GDB Remote Serial Protocol (RSP) TCP-to-Serial Bridge
// ============================================================================

fn pump_stream<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    flag: Arc<AtomicBool>,
    on_eof_msg: Option<&str>,
) {
    let mut buf = [0u8; 2048];
    while flag.load(Ordering::Relaxed) {
        match reader.read(&mut buf) {
            Ok(0) => {
                if let Some(msg) = on_eof_msg {
                    println!("{}", msg);
                }
                break;
            }
            Ok(n) => {
                if writer.write_all(&buf[..n]).is_err() || writer.flush().is_err() {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }
    }
    flag.store(false, Ordering::Relaxed);
}

/// Runs bidirectional TCP-to-Serial GDB Remote Serial Protocol (RSP) bridge
pub fn run_gdb_bridge(port_name: &str, baud_rate: u32, listen_addr: &str) -> Result<()> {
    println!("====================================================================");
    println!("   PANASONIC AM34 / DWIRE32A GDB REMOTE SERIAL PROTOCOL (RSP) BRIDGE");
    println!("   Host Architecture: Multi-Platform (RP2040 / RP2350 Dual-Core)");
    println!("   Serial Interface : {} @ {} baud", port_name, baud_rate);
    println!("   TCP GDB Listener : {}", listen_addr);
    println!("====================================================================");

    let mut serial_port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()
        .with_context(|| format!("Failed to open serial port '{}'", port_name))?;

    // Request firmware switch to GDB mode
    println!("[Bridge] Requesting controller firmware enter GDB RSP server mode...");
    let _ = serial_port.write_all(b"\r\nGDB\r\n");
    thread::sleep(Duration::from_millis(100));

    let listener = TcpListener::bind(listen_addr)
        .with_context(|| format!("Failed to bind TCP listener on '{}'", listen_addr))?;

    println!("[Bridge] Listening for GDB debugger connections on {}...", listen_addr);
    println!("[Bridge] In your debugger terminal, run:");
    println!("         (gdb) target remote {}", listen_addr);
    println!("--------------------------------------------------------------------");

    for stream_res in listener.incoming() {
        match stream_res {
            Ok(mut tcp_stream) => {
                let peer_addr = tcp_stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".to_string());
                println!("[Bridge] GDB client connected from: {}", peer_addr);

                let running = Arc::new(AtomicBool::new(true));

                // Thread 1: Serial -> TCP
                let serial_reader = serial_port.try_clone().context("Failed to clone serial port handle")?;
                let tcp_writer = tcp_stream.try_clone().context("Failed to clone TCP stream handle")?;
                let running_clone = running.clone();

                let s2t_handle = thread::spawn(move || {
                    pump_stream(serial_reader, tcp_writer, running_clone, None);
                });

                // Thread 2: TCP -> Serial
                pump_stream(
                    &mut tcp_stream,
                    &mut serial_port,
                    running,
                    Some("[Bridge] GDB client disconnected."),
                );

                let _ = s2t_handle.join();
                println!("[Bridge] Session finished. Waiting for next GDB connection...");
            }
            Err(e) => {
                eprintln!("[Bridge] Warning: TCP connection failed: {}", e);
            }
        }
    }

    Ok(())
}
