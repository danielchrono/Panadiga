import { PicoDeviceStatus, SerialLogEntry } from '@/models';

export const mockPicoDeviceStatus: PicoDeviceStatus = {
  connected: true,
  port: 'COM4 (RP2040 DWire32A Interface)',
  baudRate: 115200,
  vdd_volts: 3.28,
  target_powered: true,
  breakin_mode: 2,
  core_state: 'HALTED',
  bus_freq_hz: 1000000, // 1.0 MHz
  firmware_version: 'v1.4.2-dwire32a-panadiga',
  chip_id: 'RP2040-B2',
  total_probes: 256,
  hits_detected: 4,
  open_bus_high: 218,
  open_bus_low: 34,
  ram_loader_injected: false,
  last_keepalive: new Date().toLocaleTimeString(),
};

export const mockInitialSerialLogs: SerialLogEntry[] = [
  {
    id: '1',
    timestamp: '22:28:01.102',
    level: 'info',
    source: 'HOST',
    message: 'Iniciando handshake com Raspberry Pi Pico na porta COM4...',
  },
  {
    id: '2',
    timestamp: '22:28:01.145',
    level: 'success',
    source: 'PICO',
    message: 'RP2040 conectado! Firmware v1.4.2-dwire32a inicializado via PIO0.',
  },
  {
    id: '3',
    timestamp: '22:28:01.150',
    level: 'rx',
    source: 'PICO',
    message: 'ADC0 VDD Sense: 3.28V (Alimentação do DMR-EH55 confirmada acima de 2.50V).',
  },
  {
    id: '4',
    timestamp: '22:28:01.200',
    level: 'info',
    source: 'HOST',
    message: 'Aplicando Cold Break-In Modo 2 (EXTRG High / Pre+Post Burst 32 Clocks)...',
  },
  {
    id: '5',
    timestamp: '22:28:01.240',
    level: 'success',
    source: 'TARGET',
    message: 'AM34 Core capturado com sucesso! OCD Status: HALTED (PC: 0x80000000).',
  },
];
