/**
 * @file constants/hardwareSpecs.ts
 * @brief Immutable Physical Hardware Specifications & Functional SSOT Builder.
 *
 * Strictly separates numeric physical invariants (addresses, bus frequencies, GPIOs,
 * voltage thresholds, USB IDs, flash geometries) from localized UI labels stored in
 * `src/locales/pt-BR.json` and `src/locales/en-US.json`.
 */

import enUSJson from '@/locales/en-US.json';
import ptBRJson from '@/locales/pt-BR.json';
import type {
  BreakInModeSpec,
  ColdBreakinMode,
  FlashGeometrySpec,
  HardwareSsotSpec,
  HardwareThresholds,
  MemoryRegionSpec,
  P7501PinMapping,
  PicoCommandArgOption,
  PicoCommandSpec,
  PicoUsbIdentity,
} from '@/models';

/* =========================================================================
 * 1. Spansion S29JL064H NOR Flash & NVRAM Physical Constants
 * ========================================================================= */

/** Total physical capacity of Spansion S29JL064H NOR Flash (8 MB / 64 Mbit) */
export const SPANSION_FLASH_SIZE_BYTES = 8 * 1024 * 1024; // 8,388,608 bytes

/** Base memory offset of the top NVRAM emulation sector in 8MB dump (0x007F_0000) */
export const SPANSION_NVRAM_BASE_OFFSET = 0x007f0000;

/** Total size of the NVRAM parameter storage partition (64 KB) */
export const SPANSION_NVRAM_SIZE_BYTES = 64 * 1024; // 65,536 bytes

/** Size of each individual microsector in top-boot partition (8 KB) */
export const SPANSION_MICROSECTOR_SIZE_BYTES = 8 * 1024; // 8,192 bytes

/** Total count of 8 KB microsectors (SA134 through SA141) */
export const SPANSION_MICROSECTOR_COUNT = 8;

/** First microsector numeric index in Spansion datasheet */
export const SPANSION_MICROSECTOR_BASE_INDEX = 134;

/** Microsector index where U99 error crash logs are written */
export const SPANSION_U99_MICROSECTOR_INDEX = 141;

/** Flash erased byte value */
export const FLASH_ERASED_BYTE_VALUE = 0xff;

/* =========================================================================
 * 2. Panasonic AM34 (MN103S Core) Memory Map Architecture
 * ========================================================================= */

/** Internal SRAM Base Physical Address (256 KB) */
export const AM34_SRAM_BASE_ADDR = 0x00000000;

/** Internal SRAM Size in Bytes */
export const AM34_SRAM_SIZE_BYTES = 0x00040000; // 262,144 bytes

/** Injected RAM Loader Entrypoint Physical Address */
export const AM34_RAM_LOADER_ENTRY_ADDR = 0x00004000;

/** Flash Memory Hardware Mirror Base Address (8 MB) */
export const AM34_FLASH_BASE_ADDR = 0x80000000;

/** NVRAM Top-Boot Partition Base Physical Address in CPU space */
export const AM34_NVRAM_BASE_ADDR = 0x807f0000;

/** Panasonic UniPhier ASIC MMIO Base Physical Address */
export const AM34_MMIO_BASE_ADDR = 0xc0000000;

/** PATA / IDE Controller MMIO Base Physical Address */
export const AM34_PATA_BASE_ADDR = 0xd4000000;

/* =========================================================================
 * 3. HDD & MEIHDFS Filesystem Forensic Constants
 * ========================================================================= */

/** Standard ATA/IDE sector size in bytes */
export const LBA_SECTOR_SIZE_BYTES = 512;

/** Panasonic MEIHDFS Storage Header magic string in ASCII */
export const MEIHDFS_MBR_SIGNATURE_ASCII = 'SH';

/** Backup MBR mirror fixed sector offset (LBA 57,344) */
export const MEIHDFS_BACKUP_MBR_LBA = 57344;

/** MEIHDFS partition block unit size in bytes */
export const MEIHDFS_BLOCK_SIZE_BYTES = 2048;

/* =========================================================================
 * 4. PANA_DVD.FRM Cryptanalysis & Feistel Cipher Constants
 * ========================================================================= */

/** Known Feistel 64-bit ECB Ciphertext for 8 consecutive 0x00 bytes */
export const ECB_CIPHERTEXT_NULL_BLOCK_HEX = 'E5D1F34A518B2690';

/** Known Feistel 64-bit ECB Ciphertext for 8 consecutive 0xFF bytes (Virgin Flash) */
export const ECB_CIPHERTEXT_VIRGIN_FLASH_HEX = '035065C87BB8E090';

/** Feistel block cipher chunk size in bytes */
export const FEISTEL_BLOCK_SIZE_BYTES = 8;

/** Optical Firmware Container Magic Signature */
export const FRM_CONTAINER_MAGIC_ASCII = 'PANA_DVD_ROM';

/* =========================================================================
 * 5. Hardware Interfacing & Raspberry Pi Pico Bench Constants
 * ========================================================================= */

/** Nominal 3.3V power supply rail for Spansion and Panasonic digital logic */
export const PICO_NOMINAL_VDD_VOLTS = 3.3;

/** Minimum acceptable VDD threshold before signaling target powered / brownout (matches p7501_types.h 2.50V) */
export const PICO_MIN_VDD_THRESHOLD_VOLTS = 2.5;

/** Safe nominal LVTTL operating minimum rail (3.00V) */
export const PICO_SAFE_OPERATING_MIN_VOLTS = 3.0;

/** Maximum safe VDD ceiling (3.45V) to protect Spansion I/O pins (matches p7501_types.h 3.45V) */
export const PICO_MAX_VDD_CEILING_VOLTS = 3.45;

/** Alias kept for backwards compatibility */
export const PICO_MAX_VDD_THRESHOLD_VOLTS = PICO_MAX_VDD_CEILING_VOLTS;

/** Default serial baud rate for USB CDC UART bridge */
export const PICO_DEFAULT_BAUD_RATE = 115200;

/** GDB Remote Serial Protocol TCP default listening port */
export const GDB_DEFAULT_TCP_PORT = 3333;

/** GDB Remote Serial Protocol default loopback interface */
export const GDB_DEFAULT_LISTEN_ADDR = '127.0.0.1:3333';

/** Raspberry Pi USB Vendor ID (VID) */
export const RASPBERRY_PI_VID = 0x2e8a;

/** RP2040 CDC Serial Product ID (PID) */
export const PICO_RP2040_CDC_PID = 0x000a;

/** RP2040 W CYW43 CDC Serial Product ID (PID) */
export const PICO_RP2040_W_CDC_PID = 0x0014;

/** RP2350 (Pico 2) CDC Serial Product ID (PID) */
export const PICO_RP2350_CDC_PID = 0x000f;

/** RP2350 W (Pico 2W) CDC Serial Product ID (PID) */
export const PICO_RP2350_W_CDC_PID = 0x001f;

/** RP2350 RISC-V Hazard3 CDC Serial Product ID (PID) */
export const PICO_RP2350_RISCV_CDC_PID = 0x000e;

/** RP2040 / RP2350 USB BOOTSEL Mass Storage PID */
export const PICO_BOOTSEL_PID = 0x0003;

/* =========================================================================
 * 6. Canonical Frozen SSOT Blueprints + Functional i18n Spec Builder
 * ========================================================================= */

const SSOT_LOCALE_TABLES = Object.freeze({
  'pt-BR': ptBRJson.ssot,
  'en-US': enUSJson.ssot,
});

export type SupportedLocale = keyof typeof SSOT_LOCALE_TABLES;

export const HARDWARE_THRESHOLDS_SPEC: HardwareThresholds = Object.freeze({
  bus_freq_hz: 1_000_000,
  nominal_vdd_volts: PICO_NOMINAL_VDD_VOLTS,
  min_vdd_threshold_volts: PICO_MIN_VDD_THRESHOLD_VOLTS,
  safe_operating_min_volts: PICO_SAFE_OPERATING_MIN_VOLTS,
  max_vdd_ceiling_volts: PICO_MAX_VDD_CEILING_VOLTS,
  default_baud_rate: PICO_DEFAULT_BAUD_RATE,
  default_gdb_socket: GDB_DEFAULT_LISTEN_ADDR,
  ram_loader_base_hex: '0x00004000',
  default_nvram_base_hex: '0x807F0000',
});

export const PICO_USB_IDENTITY_SPEC: PicoUsbIdentity = Object.freeze({
  raspberry_pi_vid: RASPBERRY_PI_VID,
  rp2040_cdc_pid: PICO_RP2040_CDC_PID,
  rp2040_w_cdc_pid: PICO_RP2040_W_CDC_PID,
  rp2350_cdc_pid: PICO_RP2350_CDC_PID,
  rp2350_w_cdc_pid: PICO_RP2350_W_CDC_PID,
  rp2350_riscv_cdc_pid: PICO_RP2350_RISCV_CDC_PID,
  bootsel_pid: PICO_BOOTSEL_PID,
});

export const INDEXED_SERVICE_MANUALS_COUNT = 30;

type PinLocaleKey = keyof typeof ptBRJson.ssot.pins;
type ModeLocaleKey = keyof typeof ptBRJson.ssot.breakinModes;
type FlashLocaleKey = keyof typeof ptBRJson.ssot.flashGeometries;
type RegionLocaleKey = keyof typeof ptBRJson.ssot.memoryRegions;
type CommandLocaleKey = keyof typeof ptBRJson.ssot.commands;

interface PinBlueprint {
  readonly key: PinLocaleKey;
  readonly manual_pin: number;
  readonly physical_pin: number;
  readonly pico_gpio: string;
}

const PINOUT_BLUEPRINTS: readonly PinBlueprint[] = Object.freeze([
  Object.freeze({ key: 'pin1', manual_pin: 1, physical_pin: 1, pico_gpio: 'GP26 (ADC0 / Pin 31)' }),
  Object.freeze({ key: 'pin2', manual_pin: 2, physical_pin: 2, pico_gpio: 'GP5 (Pin 7)' }),
  Object.freeze({ key: 'pin3', manual_pin: 3, physical_pin: 3, pico_gpio: 'GP4 (Pin 6)' }),
  Object.freeze({ key: 'pin4', manual_pin: 4, physical_pin: 4, pico_gpio: 'GND (Pin 38)' }),
  Object.freeze({ key: 'pin5', manual_pin: 5, physical_pin: 5, pico_gpio: 'GP3 (Pin 5)' }),
  Object.freeze({ key: 'pin6', manual_pin: 6, physical_pin: 6, pico_gpio: 'GP2 (Pin 4)' }),
]);

interface BreakInBlueprint {
  readonly key: ModeLocaleKey;
  readonly mode: ColdBreakinMode;
  readonly badge_variant: string;
}

const BREAKIN_BLUEPRINTS: readonly BreakInBlueprint[] = Object.freeze([
  Object.freeze({ key: 'mode0', mode: 0, badge_variant: 'secondary' }),
  Object.freeze({ key: 'mode1', mode: 1, badge_variant: 'secondary' }),
  Object.freeze({ key: 'mode2', mode: 2, badge_variant: 'success' }),
  Object.freeze({ key: 'mode3', mode: 3, badge_variant: 'warning' }),
]);

interface FlashGeometryBlueprint {
  readonly key: FlashLocaleKey;
  readonly size_mb: number;
  readonly size_bytes: number;
  readonly nvram_base_hex: string;
  readonly nvram_size_bytes: number;
  readonly microsector_range: string;
}

const FLASH_GEOMETRY_BLUEPRINTS: readonly FlashGeometryBlueprint[] = Object.freeze([
  Object.freeze({
    key: 'mb2',
    size_mb: 2,
    size_bytes: 0x0020_0000,
    nvram_base_hex: '0x801F0000',
    nvram_size_bytes: SPANSION_NVRAM_SIZE_BYTES,
    microsector_range: 'SA38..SA45',
  }),
  Object.freeze({
    key: 'mb4',
    size_mb: 4,
    size_bytes: 0x0040_0000,
    nvram_base_hex: '0x803F0000',
    nvram_size_bytes: SPANSION_NVRAM_SIZE_BYTES,
    microsector_range: 'SA70..SA77',
  }),
  Object.freeze({
    key: 'mb8',
    size_mb: 8,
    size_bytes: SPANSION_FLASH_SIZE_BYTES,
    nvram_base_hex: '0x807F0000',
    nvram_size_bytes: SPANSION_NVRAM_SIZE_BYTES,
    microsector_range: 'SA134..SA141',
  }),
  Object.freeze({
    key: 'mb16',
    size_mb: 16,
    size_bytes: 0x0100_0000,
    nvram_base_hex: '0x80FE0000',
    nvram_size_bytes: 0x0002_0000,
    microsector_range: 'SA254..SA269',
  }),
]);

interface MemoryRegionBlueprint {
  readonly key: RegionLocaleKey;
  readonly start: string;
  readonly end: string;
  readonly size_kb: number;
}

const MEMORY_REGION_BLUEPRINTS: readonly MemoryRegionBlueprint[] = Object.freeze([
  Object.freeze({ key: 'sram', start: '0x00000000', end: '0x0000FFFF', size_kb: 64 }),
  Object.freeze({ key: 'mmio', start: '0xC0000000', end: '0xC000FFFF', size_kb: 64 }),
  Object.freeze({ key: 'flash_ipl', start: '0x80000000', end: '0x800FFFFF', size_kb: 1024 }),
  Object.freeze({ key: 'rtos_fw', start: '0x80100000', end: '0x807EFFFF', size_kb: 7104 }),
  Object.freeze({ key: 'flash_nvram', start: '0x807F0000', end: '0x807FFFFF', size_kb: 64 }),
  Object.freeze({ key: 'sdram', start: '0x40000000', end: '0x43FFFFFF', size_kb: 65536 }),
]);

interface PicoCommandLocaleEntry {
  readonly label: string;
  readonly description: string;
  readonly arg_label?: string;
  readonly arg_placeholder?: string;
  readonly options?: Readonly<Record<string, string>>;
}

interface CommandBlueprint {
  readonly id: CommandLocaleKey;
  readonly category: PicoCommandSpec['category'];
  readonly syntax: string;
  readonly default_arg: string | null;
  readonly optionPairs?: ReadonlyArray<readonly [string, string]>;
  readonly dangerous?: boolean;
  readonly timeout_ms?: number;
}

const PICO_COMMAND_BLUEPRINTS: readonly CommandBlueprint[] = Object.freeze([
  { id: 'PINCHECK', category: 'electrical', syntax: 'PINCHECK', default_arg: null, timeout_ms: 4000 },
  { id: 'BUSDIAG', category: 'electrical', syntax: 'BUSDIAG', default_arg: null, timeout_ms: 12000 },
  {
    id: 'SNIFF',
    category: 'electrical',
    syntax: 'SNIFF [ms]',
    default_arg: '250',
    optionPairs: [['w250', '250'], ['w1000', '1000'], ['w3000', '3000']],
    timeout_ms: 5000,
  },
  { id: 'ISOLATE', category: 'electrical', syntax: 'ISOLATE', default_arg: null, timeout_ms: 1500 },
  { id: 'ENGAGE', category: 'electrical', syntax: 'ENGAGE', default_arg: null, timeout_ms: 1500 },
  { id: 'RESET', category: 'electrical', syntax: 'RESET', default_arg: null, timeout_ms: 2000 },
  { id: 'HANDSHAKE', category: 'ocd_flash', syntax: 'HANDSHAKE', default_arg: null, timeout_ms: 4000 },
  {
    id: 'BREAK_IN',
    category: 'ocd_flash',
    syntax: 'BREAK_IN [0..3]',
    default_arg: '2',
    optionPairs: [['m0', '0'], ['m1', '1'], ['m2', '2'], ['m3', '3']],
    timeout_ms: 3000,
  },
  { id: 'SCAN', category: 'ocd_flash', syntax: 'SCAN', default_arg: null, timeout_ms: 18000 },
  { id: 'AUTODETECT', category: 'ocd_flash', syntax: 'AUTODETECT', default_arg: null, timeout_ms: 3000 },
  { id: 'CFI', category: 'ocd_flash', syntax: 'CFI', default_arg: null, timeout_ms: 3000 },
  {
    id: 'FLASH_SIZE',
    category: 'ocd_flash',
    syntax: 'FLASH_SIZE [2|4|8|16]',
    default_arg: '8',
    optionPairs: [['mb2', '2'], ['mb4', '4'], ['mb8', '8'], ['mb16', '16']],
    timeout_ms: 2000,
  },
  {
    id: 'DUMP',
    category: 'ocd_flash',
    syntax: 'DUMP <hex_addr> <words>',
    default_arg: '0x807F0000 16',
    optionPairs: [
      ['nvram', '0x807F0000 16'],
      ['bootloader', '0x80000000 16'],
      ['sram', '0x00004000 16'],
    ],
    timeout_ms: 8000,
  },
  {
    id: 'BOOTCAP',
    category: 'uart_fmu',
    syntax: 'BOOTCAP [ms]',
    default_arg: '7500',
    optionPairs: [['s3', '3000'], ['s7_5', '7500'], ['s15', '15000']],
    timeout_ms: 32000,
  },
  {
    id: 'UARTMON',
    category: 'uart_fmu',
    syntax: 'UARTMON [ms]',
    default_arg: '5000',
    optionPairs: [['s2', '2000'], ['s5', '5000'], ['s10', '10000']],
    timeout_ms: 32000,
  },
  {
    id: 'JIGBOOT',
    category: 'uart_fmu',
    syntax: 'JIGBOOT [ms]',
    default_arg: '7500',
    optionPairs: [['s5', '5000'], ['s7_5', '7500']],
    timeout_ms: 32000,
  },
  {
    id: 'BAUD',
    category: 'uart_fmu',
    syntax: 'BAUD [ms]',
    default_arg: '1500',
    optionPairs: [['s1_5', '1500'], ['s3_0', '3000']],
    timeout_ms: 4500,
  },
  {
    id: 'BAUDSCAN',
    category: 'uart_fmu',
    syntax: 'BAUDSCAN [ms] [NORMAL|JIG] [APPLY]',
    default_arg: '3500 NORMAL',
    optionPairs: [['normal3_5', '3500 NORMAL'], ['jig4_0', '4000 JIG']],
    timeout_ms: 8000,
  },
  {
    id: 'UARTSTREAM',
    category: 'uart_fmu',
    syntax: 'UARTSTREAM [ms] [NORMAL|JIG[:ms]|NOPULL]',
    default_arg: '8000 JIG:350',
    optionPairs: [['jig8', '8000 JIG:350'], ['normal8', '8000 NORMAL'], ['passive5', '5000']],
    timeout_ms: 32000,
  },
  {
    id: 'HEXCAP',
    category: 'uart_fmu',
    syntax: 'HEXCAP [ms] [NORMAL|JIG[:ms]|NOPULL]',
    default_arg: '5000 JIG:350',
    optionPairs: [['jig5', '5000 JIG:350'], ['normal4', '4000 NORMAL']],
    timeout_ms: 32000,
  },
  {
    id: 'UARTTXHEX',
    category: 'uart_fmu',
    syntax: 'UARTTXHEX <hex> [BRK] [rx_ms]',
    default_arg: '0D0A 1500',
    optionPairs: [['crlf', '0D0A 1500'], ['help', '68656C700D 2000']],
    timeout_ms: 5000,
  },
  {
    id: 'WAITTX',
    category: 'uart_fmu',
    syntax: 'WAITTX <ms> <NORMAL|JIG[:ms]|NOPULL> <trigger> <hex|BRK> [post_ms]',
    default_arg: '8000 JIG:350 "wait 1" 0D 4000',
    optionPairs: [
      ['wait1', '8000 JIG:350 "wait 1" 0D 4000'],
      ['hitkey', '20000 JIG:350 "Hit Any Key" 1B0D 6000'],
    ],
    timeout_ms: 32000,
  },
  {
    id: 'PWRTRACE',
    category: 'electrical',
    syntax: 'PWRTRACE [ms] [NORMAL|JIG[:ms]|NOPULL]',
    default_arg: '4000 JIG:350',
    optionPairs: [['jig4', '4000 JIG:350'], ['normal3', '3000 NORMAL']],
    timeout_ms: 15000,
  },
  {
    id: 'BRIDGE',
    category: 'uart_fmu',
    syntax: 'BRIDGE [baud] [NORMAL|JIG[:ms]|NOPULL]',
    default_arg: '115200 JIG:350',
    optionPairs: [['jig115200', '115200 JIG:350'], ['passive115200', '115200']],
    dangerous: true,
    timeout_ms: 15000,
  },
  {
    id: 'FMUSCAN',
    category: 'uart_fmu',
    syntax: 'FMUSCAN [modo]',
    default_arg: 'all',
    optionPairs: [['all', 'all'], ['m0', '0'], ['m1', '1'], ['m2', '2'], ['m3', '3']],
    timeout_ms: 20000,
  },
  {
    id: 'FMUTEST',
    category: 'uart_fmu',
    syntax: 'FMUTEST <hex>',
    default_arg: '0D',
    optionPairs: [['cr', '0D'], ['crlf', '0D0A'], ['help', '3F0D']],
    timeout_ms: 10000,
  },
  {
    id: 'FMUBATCH',
    category: 'uart_fmu',
    syntax: 'FMUBATCH [step]',
    default_arg: '',
    timeout_ms: 22000,
  },
  {
    id: 'UARTSEND',
    category: 'uart_fmu',
    syntax: 'UARTSEND <texto>',
    default_arg: 'help',
    optionPairs: [['help', 'help'], ['ver', 'ver'], ['info', 'info']],
    timeout_ms: 5000,
  },
  {
    id: 'UARTPROBE',
    category: 'uart_fmu',
    syntax: 'UARTPROBE [0..8]',
    default_arg: '5',
    optionPairs: [['m5', '5'], ['m6', '6'], ['m7', '7']],
    timeout_ms: 8000,
  },
  { id: 'PING', category: 'probe', syntax: 'PING', default_arg: null, timeout_ms: 1500 },
  { id: 'STATUS', category: 'probe', syntax: 'STATUS', default_arg: null, timeout_ms: 1500 },
  { id: 'BANNER', category: 'probe', syntax: 'BANNER', default_arg: null, timeout_ms: 1500 },
  {
    id: 'LED',
    category: 'probe',
    syntax: 'LED <modo>',
    default_arg: 'IDLE',
    optionPairs: [
      ['idle', 'IDLE'],
      ['active', 'ACTIVE'],
      ['gdb', 'GDB'],
      ['fault', 'FAULT'],
      ['off', 'OFF'],
      ['auto', 'AUTO'],
    ],
    timeout_ms: 1500,
  },
  { id: 'BOOTSEL', category: 'probe', syntax: 'BOOTSEL', default_arg: null, dangerous: true, timeout_ms: 1500 },
]);

/**
 * Pure builder composing frozen physical hardware constants with i18next localization tables.
 */
export function buildCanonicalHardwareSsotSpec(locale: SupportedLocale = 'pt-BR'): HardwareSsotSpec {
  const dict = SSOT_LOCALE_TABLES[locale] ?? SSOT_LOCALE_TABLES['pt-BR'];
  const commandDict: Readonly<Record<CommandLocaleKey, PicoCommandLocaleEntry>> = dict.commands;

  const pinout: P7501PinMapping[] = PINOUT_BLUEPRINTS.map(bp => {
    const text = dict.pins[bp.key];
    return {
      manual_pin: bp.manual_pin,
      physical_pin: bp.physical_pin,
      pin_label: text.pin_label,
      pico_gpio: bp.pico_gpio,
      pico_pin: text.pico_pin,
      signal_name: text.signal_name,
      direction: text.direction,
      voltage: text.voltage,
      critical_note: text.critical_note || null,
    };
  });

  const breakin_modes: BreakInModeSpec[] = BREAKIN_BLUEPRINTS.map(bp => {
    const text = dict.breakinModes[bp.key];
    return {
      mode: bp.mode,
      label: text.label,
      badge: text.badge,
      badge_variant: bp.badge_variant,
      description: text.description,
      signal_diagram: text.signal_diagram,
    };
  });

  const flash_geometries: FlashGeometrySpec[] = FLASH_GEOMETRY_BLUEPRINTS.map(bp => ({
    size_mb: bp.size_mb,
    size_bytes: bp.size_bytes,
    chip_name: dict.flashGeometries[bp.key].chip_name,
    nvram_base_hex: bp.nvram_base_hex,
    nvram_size_bytes: bp.nvram_size_bytes,
    microsector_range: bp.microsector_range,
  }));

  const memory_regions: MemoryRegionSpec[] = MEMORY_REGION_BLUEPRINTS.map(bp => {
    const text = dict.memoryRegions[bp.key];
    return {
      name: text.name,
      start: bp.start,
      end: bp.end,
      size_kb: bp.size_kb,
      perms: text.perms,
    };
  });

  const pico_commands: PicoCommandSpec[] = PICO_COMMAND_BLUEPRINTS.map(bp => {
    const text = commandDict[bp.id];
    const options: PicoCommandArgOption[] = (bp.optionPairs ?? []).map(([optKey, value]) => ({
      label: text.options?.[optKey] ?? value,
      value,
    }));
    return {
      id: bp.id,
      category: bp.category,
      label: text.label,
      description: text.description,
      syntax: bp.syntax,
      arg_label: text.arg_label ?? null,
      arg_placeholder: text.arg_placeholder ?? null,
      default_arg: bp.default_arg,
      arg_options: options,
      dangerous: Boolean(bp.dangerous),
      timeout_ms: bp.timeout_ms ?? 3000,
    };
  });

  return Object.freeze({
    thresholds: HARDWARE_THRESHOLDS_SPEC,
    usb_ids: PICO_USB_IDENTITY_SPEC,
    indexed_manuals_count: INDEXED_SERVICE_MANUALS_COUNT,
    pinout,
    breakin_modes,
    flash_geometries,
    memory_regions,
    pico_commands,
  });
}

export const CANONICAL_HARDWARE_SSOT_SPEC: HardwareSsotSpec = buildCanonicalHardwareSsotSpec('pt-BR');
