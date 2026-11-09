/**
 * Raspberry Pi Pico (RP2040) DWire32A Hardware Bench Models
 * Matches dwire32a_jtag_research/dwire32a_types.h & dwire32a_pico_main.c
 */

export type ColdBreakinMode = 0 | 1 | 2 | 3;

export interface ColdBreakinModeInfo {
  mode: ColdBreakinMode;
  name: string;
  description: string;
  timing: string;
  recommended: boolean;
}

export type OcdCoreState = 'HALTED' | 'RUNNING' | 'RESET' | 'UNKNOWN' | 'DISCONNECTED';

export interface PicoDeviceStatus {
  connected: boolean;
  port: string;
  baudRate: number;
  vdd_volts: number;
  target_powered: boolean;
  breakin_mode: ColdBreakinMode;
  core_state: OcdCoreState;
  bus_freq_hz: number;
  firmware_version: string;
  chip_id: string; // e.g. "RP2040-B2"
  total_probes: number;
  hits_detected: number;
  open_bus_high: number;
  open_bus_low: number;
  ram_loader_injected: boolean;
  last_keepalive: string;
}

export type LogLevel = 'info' | 'warn' | 'error' | 'success' | 'tx' | 'rx';

export interface SerialLogEntry {
  id: string;
  timestamp: string;
  level: LogLevel;
  source: 'HOST' | 'PICO' | 'TARGET';
  message: string;
  hexData?: string;
}

export interface InCircuitNvramEraseResult {
  success: boolean;
  elapsed_ms: number;
  voltage_before: number;
  voltage_after: number;
  ram_loader_addr: string;
  bytes_erased: number;
  verification_sha256: string;
  log_lines: string[];
}

export type PicoArchitecture = 'Rp2040' | 'Rp2350' | 'Bootloader' | 'Unknown';

export interface PicoDeviceInfo {
  port_name: string;
  architecture: PicoArchitecture;
  vid: number;
  pid: number;
  description: string;
}

export interface PicoStatusResponse {
  connected: boolean;
  port_name: string;
  baud_rate: number;
  vdd_voltage: number;
  target_powered: boolean;
  firmware_banner: string;
  is_mock: boolean;
  message: string;
}

export interface GdbBridgeStatus {
  running: boolean;
  port_name: string;
  listen_addr: string;
  connected_clients: number;
  last_event: string;
}

export interface Am34RegisterState {
  d: [number, number, number, number];
  a: [number, number, number, number];
  mdr: number;
  psw: number;
  pc: number;
  sp: number;
  is_halted: boolean;
}

export interface GdbPacketTelemetryEvent {
  timestamp_ms: number;
  direction: string;
  command_type: string;
  raw_packet: string;
  checksum_valid: boolean;
}

export interface DigaModelProfile {
  model_id: string;
  series: string;
  flash_size_mb: number;
  flash_size_bytes: number;
  nvram_base_hex: string;
  sector_geometry: string;
  soc_name: string;
}


