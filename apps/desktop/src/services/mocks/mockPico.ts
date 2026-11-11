import { PicoDeviceStatus, SerialLogEntry } from '@/models';

export const mockPicoDeviceStatus: PicoDeviceStatus = {
  connected: false,
  port: '',
  baudRate: 115200,
  vdd_volts: 0.0,
  target_powered: false,
  breakin_mode: 2,
  core_state: 'DISCONNECTED',
  bus_freq_hz: 0,
  firmware_version: 'v2.0-PIO-DWire32A',
  chip_id: 'RP2040-B2',
  total_probes: 0,
  hits_detected: 0,
  open_bus_high: 0,
  open_bus_low: 0,
  ram_loader_injected: false,
  last_keepalive: '',
};

export const mockInitialSerialLogs: SerialLogEntry[] = [];
