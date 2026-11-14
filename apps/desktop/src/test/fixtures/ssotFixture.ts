import type { HardwareSsotSpec, PicoCommandArgOption, PicoCommandSpec } from '@/models/pico';

function opt(label: string, value: string): PicoCommandArgOption {
  return { label, value };
}

function cmd(
  id: string,
  category: PicoCommandSpec['category'],
  label: string,
  description: string,
  syntax: string,
  argSpec?: [string, string, string, PicoCommandArgOption[]],
  dangerous = false,
  timeout_ms = 3000
): PicoCommandSpec {
  return {
    id,
    category,
    label,
    description,
    syntax,
    arg_label: argSpec?.[0] ?? null,
    arg_placeholder: argSpec?.[1] ?? null,
    default_arg: argSpec?.[2] ?? null,
    arg_options: argSpec?.[3] ?? [],
    dangerous,
    timeout_ms,
  };
}

/**
 * Fallback immediate SSOT specification mirroring `src/desktop/commands/dto/ssot.rs`.
 * Used prior to `get_hardware_ssot_spec` IPC resolution or in standalone web/test environments.
 */
export const DEFAULT_HARDWARE_SSOT_SPEC: HardwareSsotSpec = {
  thresholds: {
    bus_freq_hz: 1_000_000,
    nominal_vdd_volts: 3.3,
    min_vdd_threshold_volts: 2.5,
    safe_operating_min_volts: 3.0,
    max_vdd_ceiling_volts: 3.45,
    default_baud_rate: 115200,
    default_gdb_socket: '127.0.0.1:3333',
    ram_loader_base_hex: '0x00004000',
    default_nvram_base_hex: '0x807F0000',
  },
  usb_ids: {
    raspberry_pi_vid: 0x2e8a,
    rp2040_cdc_pid: 0x000a,
    rp2040_w_cdc_pid: 0x0014,
    rp2350_cdc_pid: 0x000f,
    rp2350_w_cdc_pid: 0x001f,
    rp2350_riscv_cdc_pid: 0x000e,
    bootsel_pid: 0x0003,
  },
  indexed_manuals_count: 30,
  pinout: [
    {
      manual_pin: 1,
      physical_pin: 1,
      pin_label: 'Pino 1 [P7501 SW_UART / XFWE]',
      pico_gpio: 'GP26 (ADC0 / Pin 31)',
      pico_pin: 'Pino 31',
      signal_name: 'SW_UART & ADC0 Sense (~3.25V Hi-Z / ~2.69V PD)',
      direction: 'Target <-> Pico (Q7510/QR7503)',
      voltage: '3.3V LVTTL',
      critical_note:
        'LOW força serial boot via XFWE (corta Q7514 TX). Mantenha Hi-Z para TX normal e NUNCA alimente o Pico por este pino.',
    },
    {
      manual_pin: 2,
      physical_pin: 2,
      pin_label: 'Pino 2 [P7501 XRESET]',
      pico_gpio: 'GP5 (Pin 7)',
      pico_pin: 'Pino 7',
      signal_name: 'XRESET Open-Drain (Nunca Push-Pull)',
      direction: 'Pico Open-Drain',
      voltage: '3.3V Hi-Z / 0V',
      critical_note: null,
    },
    {
      manual_pin: 3,
      physical_pin: 3,
      pin_label: 'Pino 3 [P7501 UARTM2P_T / Q7514]',
      pico_gpio: 'GP4 (Pin 6)',
      pico_pin: 'Pino 6',
      signal_name: 'UART TX -> AM34 RX via Q7514',
      direction: 'Pico -> Target (115200 8N1)',
      voltage: '3.3V LVTTL',
      critical_note:
        'Pull-up R7570 10kΩ para NSW5V. Requer SW_UART=Hi-Z para Q7514 conduzir até o SoC AM34.',
    },
    {
      manual_pin: 4,
      physical_pin: 4,
      pin_label: 'Pino 4 [P7501 GND]',
      pico_gpio: 'GND (Pin 38)',
      pico_pin: 'Pino 38',
      signal_name: 'Terra 0.0Ω Chassi',
      direction: 'Terra Comum',
      voltage: '0.0V (VSS)',
      critical_note:
        'Manual P7501 B6B-ZR-3R4 = PINO 4 (referência comum 0.0V). Confira continuidade 0.0Ω antes de ligar.',
    },
    {
      manual_pin: 5,
      physical_pin: 5,
      pin_label: 'Pino 5 [P7501 NSW5V / IC1506]',
      pico_gpio: 'GP3 (Pin 5)',
      pico_pin: 'Pino 5',
      signal_name: 'NSW5V Sense Hi-Z ONLY (~3.29V em repouso)',
      direction: 'Target -> Pico (Hi-Z Input Sem Pulls)',
      voltage: '~3.29V (IC1506 ~5.15V upstream)',
      critical_note:
        'CRÍTICO: No DMR-EH55 P7501 Pino 5 é barramento NSW5V (originado em IC1506 ~5.15V via D7510). SEMPRE Hi-Z Input sem pulls — NUNCA dirigir push-pull!',
    },
    {
      manual_pin: 6,
      physical_pin: 6,
      pin_label: 'Pino 6 [P7501 UARTP2M_T / MAX TXD]',
      pico_gpio: 'GP2 (Pin 4)',
      pico_pin: 'Pino 4',
      signal_name: 'UART RX <- AM34 TX 115200 8N1',
      direction: 'Target -> Pico (115200 8N1)',
      voltage: '3.3V LVTTL',
      critical_note:
        'Única linha UART TX ativa observada no DMR-EH55 P7501 (R7572 10kΩ para NSW5V).',
    },
  ],
  breakin_modes: [
    {
      mode: 0,
      label: 'Modo 0: Normal Hi-Z (SW_UART Aberto)',
      badge: 'Passivo',
      badge_variant: 'secondary',
      description:
        'Mantém SW_UART (GP26) em Hi-Z durante pulso Open-Drain de XRESET; Q7514 permanece conduzindo UARTM2P_T -> AM34.',
      signal_diagram: 'SW_UART: Hi-Z | XRESET: 10ms Open-Drain Low',
    },
    {
      mode: 1,
      label: 'Modo 1: Serial Pulse (XFWE Pré-Reset)',
      badge: 'FMU',
      badge_variant: 'secondary',
      description:
        'Pulsa SW_UART LOW apenas durante a liberação de XRESET e libera GP26 em Hi-Z para habilitar TX bidirecional após handshake.',
      signal_diagram: 'SW_UART: Pulse Low -> Hi-Z | Q7514 Re-enabled',
    },
    {
      mode: 2,
      label: 'Modo 2: JIG Hold LOW (SW_UART Retido)',
      badge: 'RECOMENDADO',
      badge_variant: 'success',
      description:
        'Padrão JIGBOOT para captura UART do AM34: mantém SW_UART em LOW via Q7510 durante toda a janela de inicialização e retorna a Hi-Z.',
      signal_diagram: 'SW_UART: Hold Low -> Isolate Hi-Z | 115200 8N1 RX',
    },
    {
      mode: 3,
      label: 'Modo 3: Hi-Z Estendido (Captura Longa)',
      badge: 'Seguro',
      badge_variant: 'warning',
      description:
        'Executa reset Open-Drain em XRESET mantendo os pinos UART em escuta Hi-Z para inspeção completa de CDDIAG e DDR CHECK.',
      signal_diagram: 'XRESET: Open-Drain | GP2 RX + GP3 NSW5V Hi-Z',
    },
  ],
  flash_geometries: [
    {
      size_mb: 2,
      size_bytes: 0x0020_0000,
      chip_name: 'Spansion S29AL016J NOR Flash (2 MB / 16 Mbit)',
      nvram_base_hex: '0x801F0000',
      nvram_size_bytes: 0x0001_0000,
      microsector_range: 'SA38..SA45',
    },
    {
      size_mb: 4,
      size_bytes: 0x0040_0000,
      chip_name: 'Spansion S29JL032H NOR Flash (4 MB / 32 Mbit)',
      nvram_base_hex: '0x803F0000',
      nvram_size_bytes: 0x0001_0000,
      microsector_range: 'SA70..SA77',
    },
    {
      size_mb: 8,
      size_bytes: 0x0080_0000,
      chip_name: 'Spansion S29JL064H NOR Flash (8 MB / 64 Mbit)',
      nvram_base_hex: '0x807F0000',
      nvram_size_bytes: 0x0001_0000,
      microsector_range: 'SA134..SA141',
    },
    {
      size_mb: 16,
      size_bytes: 0x0100_0000,
      chip_name: 'Spansion S29GL128P NOR Flash (16 MB / 128 Mbit)',
      nvram_base_hex: '0x80FE0000',
      nvram_size_bytes: 0x0002_0000,
      microsector_range: 'SA254..SA269',
    },
  ],
  memory_regions: [
    { name: 'System SRAM / Work', start: '0x00000000', end: '0x0000FFFF', size_kb: 64, perms: 'RWX' },
    { name: 'AM34 Internal OCD MMIO', start: '0xC0000000', end: '0xC000FFFF', size_kb: 64, perms: 'RW' },
    { name: 'NOR Flash Bootloader / IPL', start: '0x80000000', end: '0x800FFFFF', size_kb: 1024, perms: 'RX (Protected)' },
    { name: 'Main RTOS & AV Firmware', start: '0x80100000', end: '0x807EFFFF', size_kb: 7104, perms: 'RX (Protected)' },
    { name: 'Top-Boot NVRAM (EEMUL / U99)', start: '0x807F0000', end: '0x807FFFFF', size_kb: 64, perms: 'RW (8x8KB)' },
    { name: 'DDR1 SDRAM (UniPhier)', start: '0x40000000', end: '0x43FFFFFF', size_kb: 65536, perms: 'RWX' },
  ],
  pico_commands: [
    cmd('PINCHECK', 'electrical', 'Matriz Elétrica (PINCHECK)', 'Diagnostica os 6 pinos do conector P7501 em Hi-Z, Pull-Down e Pull-Up + rise time do XRESET.', 'PINCHECK', undefined, false, 4000),
    cmd('BUSDIAG', 'electrical', 'Contenção & Boot (BUSDIAG)', 'Varre drive strengths (2..12 mA) em GP4/UARTM2P_T e perfil de boot em GP2 (UART RX) com SW_UART Hi-Z vs LOW.', 'BUSDIAG', undefined, false, 12000),
    cmd('SNIFF', 'electrical', 'Monitor Passivo (SNIFF)', 'Conta transições de borda passivas em GP2..GP5 durante janela configurável (ms).', 'SNIFF [ms]', ['Janela (ms)', '250', '250', [opt('250 ms', '250'), opt('1000 ms', '1000'), opt('3000 ms', '3000')]], false, 5000),
    cmd('ISOLATE', 'electrical', 'Isolar Barramento (Hi-Z)', 'Coloca GP2..GP5 em alta impedância para não interferir no boot normal do DMR-EH55.', 'ISOLATE', undefined, false, 1500),
    cmd('ENGAGE', 'electrical', 'Engajar UART / PIO', 'Ativa drive P7501 PIO / UART em 3.3V mantendo NSW5V (GP3) em Hi-Z.', 'ENGAGE', undefined, false, 1500),
    cmd('RESET', 'electrical', 'Pulso Open-Drain XRESET', 'Puxa XRESET para 0V em dreno aberto por 10 ms e libera em Hi-Z.', 'RESET', undefined, false, 2000),
    cmd('HANDSHAKE', 'ocd_flash', 'Executar Boot JIG', 'Executa sequência JIGBOOT / P7501 (SW_UART+XRESET) e lê resposta inicial da ROM.', 'HANDSHAKE', undefined, false, 4000),
    cmd('BREAK_IN', 'ocd_flash', 'Perfil Boot/Reset', 'Executa perfil de boot/reset discreto no P7501 (modos 0..3: Hi-Z, Serial Pulse, JIG Hold LOW, Extended).', 'BREAK_IN [0..3]', ['Modo P7501', '2', '2', [opt('Modo 0 (SW_UART Hi-Z)', '0'), opt('Modo 1 (Serial Pulse)', '1'), opt('Modo 2 (JIG Hold LOW)', '2'), opt('Modo 3 (Hi-Z Extended)', '3')]], false, 3000),
    cmd('SCAN', 'ocd_flash', 'Sondar Barramento UART', 'Audita estabilidade elétrica e detecção automática de atividade UART no P7501.', 'SCAN', undefined, false, 18000),
    cmd('AUTODETECT', 'ocd_flash', 'Autodetectar Flash (Leitura)', 'Mede espelhamento passivo de endereço (2/4/8/16 MB) sem gravação na NOR Flash.', 'AUTODETECT', undefined, false, 3000),
    cmd('CFI', 'ocd_flash', 'Consulta JEDEC CFI', 'Envia sequência JEDEC 0x98 para leitura CFI (assinatura QRY / expoente 2^N da Spansion).', 'CFI', undefined, false, 3000),
    cmd('FLASH_SIZE', 'ocd_flash', 'Definir Geometria Flash', 'Ajusta manualmente o tamanho esperado da NOR Flash para leituras/GDB RSP.', 'FLASH_SIZE [2|4|8|16]', ['Tamanho Flash', '8', '8', [opt('2 MB (S29AL016J)', '2'), opt('4 MB (S29JL032H)', '4'), opt('8 MB (S29JL064H)', '8'), opt('16 MB (S29GL128P)', '16')]], false, 2000),
    cmd('DUMP', 'ocd_flash', 'Dump Memória', 'Lê palavras de 32 bits em endereço arbitrário da SRAM/Flash/MMIO do AM34.', 'DUMP <hex_addr> <words>', ['Endereço e Contagem', '0x807F0000 16', '0x807F0000 16', [opt('NVRAM Header (0x807F0000 16)', '0x807F0000 16'), opt('Bootloader Reset (0x80000000 16)', '0x80000000 16'), opt('SRAM Loader (0x00004000 16)', '0x00004000 16')]], false, 8000),
    cmd('BOOTCAP', 'uart_fmu', 'Captura Boot UART (BOOTCAP)', 'Reinicia o gravador e captura telemetria 115200 8N1 em GP2 durante a janela configurada.', 'BOOTCAP [ms]', ['Duração (ms)', '7500', '7500', [opt('3.0 s', '3000'), opt('7.5 s', '7500'), opt('15.0 s', '15000')]], false, 32000),
    cmd('UARTMON', 'uart_fmu', 'Monitor UART Passivo', 'Escuta canal GP2 115200 8N1 sem acionar reset no gravador.', 'UARTMON [ms]', ['Duração (ms)', '5000', '5000', [opt('2.0 s', '2000'), opt('5.0 s', '5000'), opt('10.0 s', '10000')]], false, 32000),
    cmd('JIGBOOT', 'uart_fmu', 'Boot Modo Jig (JIGBOOT)', 'Inicializa o chip mantendo pino SW_UART em LOW durante a janela de captura.', 'JIGBOOT [ms]', ['Duração (ms)', '7500', '7500', [opt('5.0 s', '5000'), opt('7.5 s', '7500')]], false, 32000),
    cmd('FMUSCAN', 'uart_fmu', 'Sweep Bootrom FMU (FMUSCAN)', 'Varre sequências de handshake FMU da Bootrom Panasonic na janela Wait/Serial.', 'FMUSCAN [modo]', ['Modo FMU', 'all', 'all', [opt('Todos (all)', 'all'), opt('Modo 0', '0'), opt('Modo 1', '1'), opt('Modo 2 (seri)', '2'), opt('Modo 3 (DDR)', '3')]], false, 20000),
    cmd('FMUTEST', 'uart_fmu', 'Injetar Payload Hex FMU', 'Transmite payload hexadecimal arbitrário durante handshake SW_UART FMU e captura eco.', 'FMUTEST <hex>', ['Payload Hex', '0D', '0D', [opt('CR (0D)', '0D'), opt('CR+LF (0D0A)', '0D0A'), opt('Help (3F0D)', '3F0D')]], false, 10000),
    cmd('FMUBATCH', 'uart_fmu', 'Lote Multi-Comando FMU', 'Executa lote automatizado de probes Bootrom FMU em sequência controlada.', 'FMUBATCH [step]', ['Etapa / Batch', '', '', []], false, 22000),
    cmd('UARTSEND', 'uart_fmu', 'Enviar String TX P7501', 'Transmite string ASCII + CRLF com SW_UART=Hi-Z padrão (Q7514 em condução) ou prefixo LOW: e escuta 2.5s de resposta RX.', 'UARTSEND <texto>', ['Texto ASCII TX', 'help', 'help', [opt('help', 'help'), opt('ver', 'ver'), opt('info', 'info')]], false, 5000),
    cmd('UARTPROBE', 'uart_fmu', 'Probe Interativo UART', 'Testa modos 0..8 de estímulo UART/BREAK durante boot para descobrir console oculto.', 'UARTPROBE [0..8]', ['Modo Probe', '5', '5', [opt('Modo 5 (SW_UART Low)', '5'), opt('Modo 6 (Auto Probe)', '6'), opt('Modo 7 (Break 550ms)', '7')]], false, 8000),
    cmd('PING', 'probe', 'Ping USB CDC / BLE', 'Verifica responsividade imediata da camada CLI do firmware Pico.', 'PING', undefined, false, 1500),
    cmd('STATUS', 'probe', 'Ler Telemetria VDD/ADC', 'Lê tensão alvo no canal ADC0 (GP26) com detector ativo de pino flutuante.', 'STATUS', undefined, false, 1500),
    cmd('BANNER', 'probe', 'Versão & Banner Firmware', 'Exibe identificação completa da compilação RP2040/RP2350 em execução na sonda.', 'BANNER', undefined, false, 1500),
    cmd('LED', 'probe', 'Modo LED Onboard GP25', 'Alterna o padrão de sinalização visual do LED onboard do Pico.', 'LED <modo>', ['Padrão LED', 'IDLE', 'IDLE', [opt('IDLE (1Hz Heartbeat)', 'IDLE'), opt('ACTIVE (Sólido)', 'ACTIVE'), opt('GDB (5Hz)', 'GDB'), opt('FAULT (Triplo)', 'FAULT')]], false, 1500),
    cmd('BOOTSEL', 'probe', 'Reiniciar para BOOTSEL USB', 'Reinicia o Pico para bootloader USB Mass Storage (`RPI-RP2` / `RP2350`).', 'BOOTSEL', undefined, true, 1500),
  ],
};
