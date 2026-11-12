//! CLI argument definitions for RP2040/RP2350 Pico hardware control and GDB RSP bridge.

use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct PicoArgs {
    #[command(subcommand)]
    pub action: PicoActions,
}

#[derive(Subcommand, Debug)]
pub enum PicoActions {
    /// Lists all attached serial COM ports and identifies RP2040 vs RP2350 Pico 2 hardware
    List,

    /// Probes a Pico board, verifies communication, and reads target 3.3V VDD rail voltage
    Ping {
        /// Serial port identifier (e.g. COM3 or /dev/ttyACM0)
        #[arg(short, long, value_name = "PORT")]
        port: String,

        /// Baud rate (defaults to 115200)
        #[arg(short, long, default_value_t = 115200)]
        baud: u32,
    },

    /// Starts a TCP-to-Serial GDB Remote Serial Protocol (RSP) bridge server
    GdbBridge {
        /// Serial port identifier connected to Raspberry Pi Pico (e.g. COM3)
        #[arg(short, long, value_name = "PORT")]
        port: String,

        /// Serial baud rate (defaults to 115200)
        #[arg(short, long, default_value_t = 115200)]
        baud: u32,

        /// TCP listener address and port for incoming GDB connections
        #[arg(short, long, default_value = "127.0.0.1:3333")]
        listen: String,
    },

    /// Automatically flashes the DWire32A firmware to a detected Raspberry Pi Pico (RP2040/RP2350)
    Flash {
        /// Optional serial port of a running Pico to trigger 1200-baud touch reset into BOOTSEL mode
        #[arg(short, long, value_name = "PORT")]
        port: Option<String>,

        /// Path to custom .uf2 firmware file (defaults to scanning dist_bin/firmware or dwire32a_jtag_research)
        #[arg(short, long, value_name = "UF2_FILE")]
        file: Option<PathBuf>,
    },
}

#[derive(Args, Debug)]
pub struct GdbConfigArgs {
    /// Serial port name (e.g. COM5, /dev/ttyACM0)
    #[arg(short, long, default_value = "COM5", value_name = "PORT")]
    pub port: String,

    /// TCP port for GDB remote protocol
    #[arg(short, long, default_value_t = 3333, value_name = "TCP_PORT")]
    pub tcp_port: u16,

    /// TCP listen host address
    #[arg(short = 'H', long, default_value = "127.0.0.1", value_name = "HOST")]
    pub host: String,
}
