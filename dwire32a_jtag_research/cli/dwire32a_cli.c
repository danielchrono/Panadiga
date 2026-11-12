/**
 * @file dwire32a_cli.c
 * @brief USB CDC CLI parser, safety interlocks, and modular command router.
 *
 * Routes incoming text commands cleanly to:
 *   - Hardware HAL (`hal/dwire32a_hw.h`)
 *   - OCD Protocol (`protocol/dwire32a_protocol.h`)
 *   - NOR Flash & SRAM Loader (`flash/dwire32a_flash.h`)
 *   - Electrical & OCD Diagnostics (`diag/dwire32a_diag_{electrical,ocd}.h`)
 *   - P7501 Service UART & FMU Probes (`uart/p7501_*.h`)
 *   - GDB Remote Serial Protocol Bridge (`gdb/gdb_rsp.h`)
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "cli/dwire32a_cli.h"
#include "diag/dwire32a_diag_electrical.h"
#include "diag/dwire32a_diag_ocd.h"
#include "flash/dwire32a_flash.h"
#include "flash/ram_loader_payload.h"
#include "uart/p7501_uart_capture.h"
#include "uart/p7501_fmu_probe.h"
#include "uart/p7501_uart_interactive.h"
#include "hal/dwire32a_hw.h"
#include "hal/dwire32a_led.h"
#include "gdb/gdb_rsp.h"

#include <ctype.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "pico/stdlib.h"
#include "pico/bootrom.h"

static bool cli_is_ws(char c) {
    return (c == ' ') || (c == '\t') || (c == '\r') || (c == '\n');
}

static bool cli_str_eq_upper(const char *a, const char *upper_b) {
    if ((a == NULL) || (upper_b == NULL)) {
        return false;
    }
    while ((*a != '\0') && (*upper_b != '\0')) {
        if ((char)toupper((unsigned char)*a) != *upper_b) {
            return false;
        }
        a++;
        upper_b++;
    }
    return (*a == '\0') && (*upper_b == '\0');
}

static void cli_copy_upper_token(const char *src, char *dst, size_t dst_cap) {
    size_t i = 0U;
    while ((src[i] != '\0') && (!cli_is_ws(src[i])) && (i + 1U < dst_cap)) {
        dst[i] = (char)toupper((unsigned char)src[i]);
        i++;
    }
    dst[i] = '\0';
}

static const char *cli_extract_arg(const char *trimmed_line) {
    const char *p = trimmed_line;
    while ((*p != '\0') && (!cli_is_ws(*p))) {
        p++;
    }
    while ((*p != '\0') && cli_is_ws(*p)) {
        p++;
    }
    return p;
}

static const char *cli_mcu_name(void) {
#if defined(PICO_RP2350) && PICO_RP2350
    return "RP2350 (Pico 2 / ARM Cortex-M33)";
#else
    return "RP2040 (Pico / ARM Cortex-M0+)";
#endif
}

static void cli_cmd_status(void) {
    const float vdd = dwire_hw_read_vdd();
    const char *state = "LOW / UNPOWERED";
    pico_led_mode_t next_led = LED_MODE_BUS_FAULT;

    if (vdd >= DWIRE_VDD_FLOATING_CEIL_V) {
        state = "FLOATING / NO TARGET";
    } else if (vdd >= DWIRE_VDD_THRESHOLD_V) {
        state = "OK";
        next_led = LED_MODE_HEARTBEAT_IDLE;
    }

    if (!gdb_rsp_is_active()) {
        dwire_led_set_mode(next_led);
    }
    printf("VDD: %.2fV (%s)\n", (double)vdd, state);
}

static void cli_cmd_flash_size(const char *arg) {
    if ((arg == NULL) || (*arg == '\0')) {
        const dwire_flash_geometry_t geo = dwire_flash_get_geometry(gdb_rsp_get_flash_size());
        printf("[FLASH_SIZE] Active: %" PRIu32 " MB (0x%08" PRIX32 ") | EEMUL NVRAM: 0x%08" PRIX32 "..0x%08" PRIX32 "\n",
               geo.mb, geo.bytes, geo.nvram_base, geo.nvram_end - 1UL);
        return;
    }

    uint32_t req = (uint32_t)strtoul(arg, NULL, 0);
    if (req <= 16U) {
        req *= 1024UL * 1024UL;
    }
    if (gdb_rsp_set_flash_size(req)) {
        const dwire_flash_geometry_t geo = dwire_flash_get_geometry(req);
        printf("[FLASH_SIZE] Configured %" PRIu32 " MB | EEMUL NVRAM: 0x%08" PRIX32 "..0x%08" PRIX32 "\n",
               geo.mb, geo.nvram_base, geo.nvram_end - 1UL);
    } else {
        printf("[FLASH_SIZE] ERROR: Unsupported size. Valid values: 2, 4, 8, 16 (MB).\n");
    }
}

static void cli_cmd_inject_loader(const char *arg) {
    if (!cli_str_eq_upper(arg, "CONFIRM")) {
        printf("[SAFETY INTERLOCK] Pass CONFIRM to inject AM34 SRAM loader: INJECT_LOADER CONFIRM\n");
        return;
    }

    dwire_led_set_mode(LED_MODE_DWIRE_ACTIVE);
    const dwire_status_t rc = ram_loader_inject_for_flash_size(
        AM34_SRAM_LOADER_BASE_ADDR,
        true,
        gdb_rsp_get_flash_size()
    );
    dwire_led_set_mode(LED_MODE_HEARTBEAT_IDLE);

    if (rc == DWIRE_OK) {
        printf("INJECT_LOADER: OK\n");
    } else {
        printf("INJECT_LOADER: ERROR (%d)\n", (int)rc);
    }
}

static void cli_cmd_erase_nvram(const char *arg) {
    if (!cli_str_eq_upper(arg, "CONFIRM")) {
        printf("[SAFETY INTERLOCK] Pass CONFIRM to erase Top-Boot EEMUL NVRAM: ERASE_NVRAM CONFIRM\n");
        return;
    }

    dwire_led_set_mode(LED_MODE_DWIRE_ACTIVE);
    const dwire_status_t rc = ram_loader_execute_erase_for_flash_size(gdb_rsp_get_flash_size());
    dwire_led_set_mode(LED_MODE_HEARTBEAT_IDLE);

    if (rc == DWIRE_OK) {
        printf("ERASE_NVRAM: OK\n");
    } else {
        printf("ERASE_NVRAM: ERROR (%d)\n", (int)rc);
    }
}

static void cli_cmd_led(const char *arg) {
    if (cli_str_eq_upper(arg, "ACTIVE")) {
        dwire_led_set_mode(LED_MODE_DWIRE_ACTIVE);
        printf("[LED] Mode set to DWIRE_ACTIVE.\n");
    } else if (cli_str_eq_upper(arg, "GDB")) {
        dwire_led_set_mode(LED_MODE_GDB_RSP);
        printf("[LED] Mode set to GDB_RSP.\n");
    } else if (cli_str_eq_upper(arg, "FAULT")) {
        dwire_led_set_mode(LED_MODE_BUS_FAULT);
        printf("[LED] Mode set to BUS_FAULT.\n");
    } else {
        dwire_led_set_mode(LED_MODE_HEARTBEAT_IDLE);
        printf("[LED] Mode set to HEARTBEAT_IDLE.\n");
    }
}

char *dwire_cli_trim_whitespace(char *str) {
    if (str == NULL) {
        return NULL;
    }
    while ((*str != '\0') && cli_is_ws(*str)) {
        str++;
    }
    size_t len = strlen(str);
    while ((len > 0U) && cli_is_ws(str[len - 1U])) {
        str[len - 1U] = '\0';
        len--;
    }
    return str;
}

void dwire_cli_print_banner(void) {
    printf("\n====================================================================\n");
    printf("   PANADIGA: Panasonic DWire32A / Matsushita AM34 Bench Probe\n");
    printf("   MCU Host : %s\n", cli_mcu_name());
    printf("   Bus Rate : %lu Hz (PIO Half-Duplex Synchronous OCD)\n", (unsigned long)DWIRE_BUS_FREQ_HZ);
    printf("   %s\n", gdb_rsp_get_status_info());
    printf("   Type HELP for available diagnostic, UART, Flash and GDB commands.\n");
    printf("====================================================================\n");
}

void dwire_cli_print_help(void) {
    printf("\n--- DWire32A Bench CLI Commands ---\n");
    printf("  PING / STATUS / VDD        - Probe heartbeat & GP26 3.3V VDD sense\n");
    printf("  HANDSHAKE                  - Multi-mode Cold Break-In + Flash autodetect\n");
    printf("  DUMP [hex_addr] [words]    - Read 32-bit words via AM34 OCD memory bus\n");
    printf("  SCAN                       - Sweep 256 OCD opcodes across 4 break-in modes\n");
    printf("  SNIFF [ms] / PINCHECK      - Passive transition sniffer / 6-pin electrical check\n");
    printf("  BUSDIAG / BREAK_IN [0..3]  - Bus contention check / manual Cold Break-In\n");
    printf("  AUTODETECT / CFI           - Passive Flash alias probe / Spansion CFI QRY\n");
    printf("  FLASH_SIZE [2|4|8|16]      - Get/set Flash geometry (default: 8 MB)\n");
    printf("  INJECT_LOADER CONFIRM      - Stream 80B Jedi loader into SRAM 0x00004000\n");
    printf("  ERASE_NVRAM CONFIRM        - Execute U99 surgical NVRAM micro-sector erase\n");
    printf("  GDB                        - Switch USB CDC into GDB RSP bridge mode\n");
    printf("  ISOLATE / ENGAGE / RESET   - Hi-Z isolation / active PIO / target pulse\n");
    printf("  BOOTCAP / UARTMON / JIGBOOT- P7501 115200 8N1 serial boot capture\n");
    printf("  FMUSCAN / FMUTEST / FMUBATCH / UARTSEND / UARTPROBE - P7501 UART tools\n");
    printf("  LED <IDLE|ACTIVE|GDB|FAULT> / BOOTSEL - Status LED / USB UF2 bootloader\n\n");
}

void dwire_cli_process_command(char *line) {
    char *trimmed = dwire_cli_trim_whitespace(line);
    if ((trimmed == NULL) || (*trimmed == '\0')) {
        return;
    }

    char cmd[32];
    cli_copy_upper_token(trimmed, cmd, sizeof(cmd));
    const char *arg = cli_extract_arg(trimmed);

    if (strcmp(cmd, "PING") == 0) {
        printf("PONG\n");
    } else if ((strcmp(cmd, "STATUS") == 0) || (strcmp(cmd, "VDD") == 0)) {
        cli_cmd_status();
    } else if ((strcmp(cmd, "HELP") == 0) || (strcmp(cmd, "?") == 0)) {
        dwire_cli_print_help();
    } else if (strcmp(cmd, "BANNER") == 0) {
        dwire_cli_print_banner();
    } else if (strcmp(cmd, "SCAN") == 0) {
        dwire_led_set_mode(LED_MODE_DWIRE_ACTIVE);
        dwire_diag_cmd_scan();
        dwire_led_set_mode(LED_MODE_HEARTBEAT_IDLE);
    } else if (strcmp(cmd, "HANDSHAKE") == 0) {
        dwire_led_set_mode(LED_MODE_DWIRE_ACTIVE);
        dwire_diag_cmd_handshake();
        dwire_led_set_mode(LED_MODE_HEARTBEAT_IDLE);
    } else if (strcmp(cmd, "DUMP") == 0) {
        dwire_led_set_mode(LED_MODE_DWIRE_ACTIVE);
        dwire_diag_cmd_dump(arg);
        dwire_led_set_mode(LED_MODE_HEARTBEAT_IDLE);
    } else if (strcmp(cmd, "SNIFF") == 0) {
        dwire_diag_cmd_sniff(arg);
    } else if (strcmp(cmd, "BREAK_IN") == 0) {
        dwire_diag_cmd_break_in(arg);
    } else if (strcmp(cmd, "PINCHECK") == 0) {
        dwire_diag_cmd_pincheck();
    } else if (strcmp(cmd, "BUSDIAG") == 0) {
        dwire_diag_cmd_busdiag();
    } else if (strcmp(cmd, "BOOTCAP") == 0) {
        p7501_cmd_boot_capture(arg, true, false, "BOOTCAP");
    } else if (strcmp(cmd, "UARTMON") == 0) {
        p7501_cmd_boot_capture(arg, false, false, "UARTMON");
    } else if (strcmp(cmd, "JIGBOOT") == 0) {
        p7501_cmd_boot_capture(arg, true, true, "JIGBOOT");
    } else if (strcmp(cmd, "FMUSCAN") == 0) {
        p7501_cmd_fmu_scan(arg);
    } else if (strcmp(cmd, "FMUTEST") == 0) {
        p7501_cmd_fmu_test(arg);
    } else if (strcmp(cmd, "FMUBATCH") == 0) {
        p7501_cmd_fmu_batch(arg);
    } else if (strcmp(cmd, "UARTSEND") == 0) {
        p7501_cmd_uart_send(arg);
    } else if (strcmp(cmd, "UARTPROBE") == 0) {
        p7501_cmd_uart_probe(arg);
    } else if (strcmp(cmd, "GDB") == 0) {
        printf("Switching to GDB Remote Serial Protocol (RSP) Bridge Mode...\n");
        fflush(stdout);
        gdb_rsp_set_active(true);
        dwire_led_set_mode(LED_MODE_GDB_RSP);
    } else if ((strcmp(cmd, "ISOLATE") == 0) || (strcmp(cmd, "HIZ") == 0)) {
        dwire_hw_isolate_bus_hiz();
        printf("[BUS] P7501/CN55001 pins placed in High-Impedance Isolation (Hi-Z).\n");
    } else if (strcmp(cmd, "ENGAGE") == 0) {
        dwire_hw_engage_bus_active();
        printf("[BUS] Active DWire32A PIO drive engaged on P7501/CN55001.\n");
    } else if ((strcmp(cmd, "AUTODETECT") == 0) || (strcmp(cmd, "PROBE") == 0)) {
        const dwire_flash_geometry_t geo = dwire_probe_flash_geometry();
        printf("[AUTODETECT] Passive Address Aliasing Probe: %" PRIu32 " MB NOR Flash detected (EEMUL NVRAM @ 0x%08" PRIX32 ").\n",
               geo.mb, geo.nvram_base);
    } else if (strcmp(cmd, "CFI") == 0) {
        uint32_t cfi_bytes = 0U;
        if (dwire_hw_query_cfi(&cfi_bytes)) {
            printf("[CFI] Spansion QRY valid: %" PRIu32 " bytes (%" PRIu32 " MB).\n",
                   cfi_bytes, cfi_bytes / (1024UL * 1024UL));
        } else {
            printf("[CFI] No valid QRY response (check Cold Break-In & bus connections).\n");
        }
    } else if (strcmp(cmd, "FLASH_SIZE") == 0) {
        cli_cmd_flash_size(arg);
    } else if (strcmp(cmd, "INJECT_LOADER") == 0) {
        cli_cmd_inject_loader(arg);
    } else if (strcmp(cmd, "ERASE_NVRAM") == 0) {
        cli_cmd_erase_nvram(arg);
    } else if (strcmp(cmd, "RESET") == 0) {
        dwire_hw_reset_assert();
        sleep_ms(DWIRE_RESET_ASSERT_MS);
        dwire_hw_reset_release();
        printf("[RESET] Open-drain nRESET pulse complete.\n");
    } else if (strcmp(cmd, "LED") == 0) {
        cli_cmd_led(arg);
    } else if (strcmp(cmd, "BOOTSEL") == 0) {
        printf("[BOOTSEL] Rebooting into USB UF2 Mass-Storage Bootloader...\n");
        fflush(stdout);
        sleep_ms(50U);
        reset_usb_boot(0U, 0U);
    } else {
        printf("Unknown command: '%s'. Type HELP for commands.\n", cmd);
    }
}
