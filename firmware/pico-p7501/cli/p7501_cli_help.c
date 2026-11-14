/**
 * @file p7501_cli_help.c
 * @brief Banner and help reference rendering for the Panadiga P7501 bench CLI.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "cli/p7501_cli_help.h"

#include "gdb/gdb_rsp.h"
#include "p7501_types.h"
#include "uart/p7501_uart_pio_engine.h"

#include <inttypes.h>
#include <stdio.h>

static const char *cli_mcu_name(void) {
#if defined(PICO_RP2350) && (PICO_RP2350 != 0)
    return "RP2350 (Pico 2 / ARM Cortex-M33 Dual @ 150MHz)";
#elif defined(PANADIGA_BLE_ENABLED) && (PANADIGA_BLE_ENABLED != 0)
    return "RP2040-W (Pico W + CYW43 BLE / ARM Cortex-M0+)";
#else
    return "RP2040 (Pico / ARM Cortex-M0+)";
#endif
}

void p7501_cli_print_banner(void) {
    printf("\n====================================================================\n");
    printf("   PANADIGA: Panasonic DMR-EH55 P7501 Async UART & Boot Bench Probe\n");
    printf("   MCU Host : %s\n", cli_mcu_name());
    printf("   Bus      : P7501 (B6B-ZR-3R4) %" PRIu32 " 8N1 (PIO1) + SW_UART / XRESET / NSW5V(Hi-Z)\n",
           p7501_pio_get_baud());
    printf("   %s\n", gdb_rsp_get_status_info());
    printf("   Type HELP for available P7501 UART, PIO1, FMU, diag, Flash, LED & GDB commands.\n");
    printf("====================================================================\n");
}

void p7501_cli_print_help(void) {
    printf("\n--- Panadiga P7501 Bench CLI Commands ---\n");
    printf("  PING / STATUS / VDD                  - Heartbeat & GP26 ADC0 SW_UART sense\n");
    printf("  BOOTCAP / UARTMON / JIGBOOT          - Legacy 115200 8N1 boot/passive RX capture\n");
    printf("  UARTSEND / UARTPROBE                 - Legacy P7501 TX (Q7514 Hi-Z default) & probe\n");
    printf("  BAUD [bps] / BAUDSCAN [ms] [b] [APPLY] - PIO1 multi-baud & GP2 autobaud detector\n");
    printf("  UARTSTREAM [ms] [NORMAL|JIG[:ms]|NOPULL] - Live non-blocking PIO1 RX stream\n");
    printf("  HEXCAP [ms] [NORMAL|JIG[:ms]|NOPULL]     - Microsecond-delta hex+ASCII RX capture\n");
    printf("  UARTTXHEX [LOW:]<hex|BRK> [rx_ms]    - PIO1 arbitrary hex / 120ms BREAK inject\n");
    printf("  WAITTX <ms> <boot> \"<trig>\" <hex|BRK> [post] - Triggered TX inject on match\n");
    printf("  BRIDGE [baud] [NORMAL|JIG[:ms]|NOPULL]   - Raw USB-CDC <-> SC0 passthrough (~~~ exit)\n");
    printf("  PWRTRACE [ms] [boot]                 - Boot ADC0 rail & NSW5V/RXD/TXD glitch logger\n");
    printf("  FMUSCAN / FMUTEST / FMUBATCH         - Bootrom FMU handshake sweep & batch probes\n");
    printf("  PINCHECK / BUSDIAG / SNIFF           - 6-pin electrical matrix & NSW5V Hi-Z guard\n");
    printf("  ISOLATE / ENGAGE / RESET             - Hi-Z isolate / TX idle HIGH / XRESET pulse\n");
    printf("  HANDSHAKE / SCAN / BREAK_IN          - Boot profile sweep & synchronous monitor check\n");
    printf("  DUMP / AUTODETECT / CFI / FLASH_SIZE - Monitor aperture read & NOR Flash geometry\n");
    printf("  INJECT_LOADER / ERASE_NVRAM          - Guarded SRAM loader / EEMUL NVRAM erase\n");
    printf("  GDB / LED <IDLE|ACTIVE|GDB|FAULT|OFF|AUTO> / BOOTSEL\n\n");
}
