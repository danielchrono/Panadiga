/**
 * @file p7501_cli.c
 * @brief USB CDC CLI parser, safety interlocks, and modular command router for P7501.
 *
 * Routes incoming text commands cleanly to:
 *   - Hardware HAL (`hal/p7501_hw.h`)
 *   - Bus Protocol (`protocol/p7501_bus_protocol.h`)
 *   - NOR Flash & SRAM Loader (`flash/p7501_flash.h`)
 *   - Electrical & Bus Diagnostics (`diag/p7501_diag_{electrical,bus}.h`)
 *   - P7501 Service UART & FMU Probes (`uart/p7501_*.h`)
 *   - GDB Remote Serial Protocol Bridge (`gdb/gdb_rsp.h`)
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "cli/p7501_cli.h"
#include "diag/p7501_diag_electrical.h"
#include "diag/p7501_diag_bus.h"
#include "flash/p7501_flash_cli.h"
#include "common/p7501_hex.h"
#include "uart/p7501_uart_codec.h"
#include "uart/p7501_uart_capture.h"
#include "uart/p7501_fmu_probe.h"
#include "uart/p7501_uart_interactive.h"
#include "cli/p7501_cli_help.h"
#include "uart/p7501_uart_stream.h"
#include "uart/p7501_uart_baud.h"
#include "uart/p7501_uart_tx.h"
#include "diag/p7501_diag_pwrtrace.h"
#include "hal/p7501_hw.h"
#include "hal/p7501_led.h"
#include "gdb/gdb_rsp.h"

#include <stdio.h>
#include <string.h>
#include "pico/stdlib.h"
#include "pico/bootrom.h"

static bool cli_is_ws(char c) {
    return (c == ' ') || (c == '\t') || (c == '\r') || (c == '\n');
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

static void cli_cmd_status(void) {
    const float vdd = p7501_hw_read_vdd();
    const char *state = "LOW / UNPOWERED";
    pico_led_mode_t next_led = LED_MODE_BUS_FAULT;

    if (vdd >= P7501_VDD_FLOATING_CEIL_V) {
        state = "FLOATING / NO TARGET";
    } else if (vdd >= P7501_VDD_THRESHOLD_V) {
        state = "OK";
        next_led = LED_MODE_HEARTBEAT_IDLE;
    }

    if (!gdb_rsp_is_active()) {
        p7501_led_set_auto_mode(next_led);
    }
    printf("VDD: %.2fV (%s)\n", (double)vdd, state);
}

static void cli_cmd_led(const char *arg) {
    pico_led_mode_t req_mode = LED_MODE_HEARTBEAT_IDLE;
    bool is_auto = false;
    bool is_query = false;

    if (!p7501_led_parse_arg(arg, &req_mode, &is_auto, &is_query)) {
        printf("[LED] ERROR: Invalid mode '%s'. Use IDLE, ACTIVE, GDB, FAULT, OFF, or AUTO.\n",
               arg ? arg : "");
        return;
    }

    if (is_auto) {
        const float vdd = p7501_hw_read_vdd();
        const bool vdd_ok = (vdd >= P7501_VDD_THRESHOLD_V && vdd < P7501_VDD_FLOATING_CEIL_V);
        p7501_led_unlock_auto(vdd_ok ? LED_MODE_HEARTBEAT_IDLE : LED_MODE_BUS_FAULT);
    } else if (!is_query) {
        p7501_led_set_mode(req_mode);
    }

    const p7501_led_state_t st = p7501_led_get_state();
    printf("[LED] Pico %s | mode=%s | lock=%s | pin=%d\n",
           p7501_led_backend_label(),
           p7501_led_mode_name(st.mode),
           st.manual_lock ? "MANUAL" : "AUTO",
           st.current_on ? 1 : 0);
    printf("LED: OK (%s, %s)\n",
           p7501_led_mode_name(st.mode),
           st.manual_lock ? "MANUAL_LOCK" : "AUTO");
}

char *p7501_cli_trim_whitespace(char *str) {
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

static void cli_cmd_ping(const char *arg) { (void)arg; printf("PONG\n"); }
static void cli_cmd_status_wrap(const char *arg) { (void)arg; cli_cmd_status(); }
static void cli_cmd_help_wrap(const char *arg) { (void)arg; p7501_cli_print_help(); }
static void cli_cmd_banner_wrap(const char *arg) { (void)arg; p7501_cli_print_banner(); }
static void cli_cmd_sniff_wrap(const char *arg) { p7501_diag_cmd_sniff(arg); }
static void cli_cmd_pincheck_wrap(const char *arg) { (void)arg; p7501_diag_cmd_pincheck(); }
static void cli_cmd_busdiag_wrap(const char *arg) { (void)arg; p7501_diag_cmd_busdiag(); }
static void cli_cmd_bootcap_wrap(const char *arg) { p7501_cmd_boot_capture(arg, true, false, "BOOTCAP"); }
static void cli_cmd_uartmon_wrap(const char *arg) { p7501_cmd_boot_capture(arg, false, false, "UARTMON"); }
static void cli_cmd_jigboot_wrap(const char *arg) { p7501_cmd_boot_capture(arg, true, true, "JIGBOOT"); }

static void cli_cmd_scan_wrap(const char *arg) {
    (void)arg;
    p7501_led_set_auto_mode(LED_MODE_BUS_ACTIVE);
    p7501_diag_cmd_scan();
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);
}

static void cli_cmd_handshake_wrap(const char *arg) {
    (void)arg;
    p7501_led_set_auto_mode(LED_MODE_BUS_ACTIVE);
    p7501_diag_cmd_handshake();
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);
}

static void cli_cmd_dump_wrap(const char *arg) {
    p7501_led_set_auto_mode(LED_MODE_BUS_ACTIVE);
    p7501_diag_cmd_dump(arg);
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);
}

static void cli_cmd_gdb(const char *arg) {
    (void)arg;
    printf("Switching to GDB Remote Serial Protocol (RSP) Bridge Mode...\n");
    printf("[GDB] Note: Without DWire32A hardware clock on P7501, use 'BRIDGE <baud>' for AM34 _dev_SERIAL GDB.\n");
    fflush(stdout);
    gdb_rsp_set_active(true);
    p7501_led_set_auto_mode(LED_MODE_GDB_RSP);
}

static void cli_cmd_isolate(const char *arg) {
    (void)arg;
    p7501_hw_isolate_bus_hiz();
    printf("[BUS] P7501 pins placed in High-Impedance Isolation (NSW5V/XRESET Hi-Z, GP26 ADC0).\n");
}

static void cli_cmd_engage(const char *arg) {
    (void)arg;
    p7501_hw_engage_uart_bus();
    printf("[BUS] P7501 UART TX (GP4) engaged idle HIGH; NSW5V (GP3) kept Hi-Z Input.\n");
}

static void cli_cmd_reset_pulse(const char *arg) {
    (void)arg;
    p7501_hw_reset_assert();
    sleep_ms(P7501_RESET_ASSERT_MS);
    p7501_hw_reset_release();
    printf("[RESET] Open-drain XRESET pulse complete.\n");
}

static void cli_cmd_bootsel(const char *arg) {
    (void)arg;
    printf("[BOOTSEL] Rebooting into USB UF2 Mass-Storage Bootloader...\n");
    fflush(stdout);
    sleep_ms(50U);
    reset_usb_boot(0U, 0U);
}

typedef void (*p7501_cli_handler_fn)(const char *arg);

typedef struct p7501_cli_cmd_entry {
    const char *name;
    p7501_cli_handler_fn handler;
} p7501_cli_cmd_entry_t;

static const p7501_cli_cmd_entry_t k_cli_commands[] = {
    { "PING",          cli_cmd_ping },
    { "STATUS",        cli_cmd_status_wrap },
    { "VDD",           cli_cmd_status_wrap },
    { "HELP",          cli_cmd_help_wrap },
    { "?",             cli_cmd_help_wrap },
    { "BANNER",        cli_cmd_banner_wrap },
    { "SCAN",          cli_cmd_scan_wrap },
    { "HANDSHAKE",     cli_cmd_handshake_wrap },
    { "DUMP",          cli_cmd_dump_wrap },
    { "SNIFF",         cli_cmd_sniff_wrap },
    { "BREAK_IN",      p7501_diag_cmd_boot_profile },
    { "BOOT_PROFILE",  p7501_diag_cmd_boot_profile },
    { "PINCHECK",      cli_cmd_pincheck_wrap },
    { "BUSDIAG",       cli_cmd_busdiag_wrap },
    { "BOOTCAP",       cli_cmd_bootcap_wrap },
    { "UARTMON",       cli_cmd_uartmon_wrap },
    { "JIGBOOT",       cli_cmd_jigboot_wrap },
    { "FMUSCAN",       p7501_cmd_fmu_scan },
    { "FMUTEST",       p7501_cmd_fmu_test },
    { "FMUBATCH",      p7501_cmd_fmu_batch },
    { "UARTSEND",      p7501_cmd_uart_send },
    { "UARTPROBE",     p7501_cmd_uart_probe },
    { "GDB",           cli_cmd_gdb },
    { "ISOLATE",       cli_cmd_isolate },
    { "HIZ",           cli_cmd_isolate },
    { "ENGAGE",        cli_cmd_engage },
    { "AUTODETECT",    p7501_flash_cmd_autodetect },
    { "PROBE",         p7501_flash_cmd_autodetect },
    { "CFI",           p7501_flash_cmd_cfi },
    { "FLASH_SIZE",    p7501_flash_cmd_size },
    { "INJECT_LOADER", p7501_flash_cmd_inject_loader },
    { "ERASE_NVRAM",   p7501_flash_cmd_erase_nvram },
    { "RESET",         cli_cmd_reset_pulse },
    { "LED",           cli_cmd_led },
    { "BAUD",          p7501_cmd_baud },
    { "BAUDSCAN",      p7501_cmd_baudscan },
    { "UARTSTREAM",    p7501_cmd_uart_stream },
    { "HEXCAP",        p7501_cmd_hexcap },
    { "UARTTXHEX",     p7501_cmd_uart_tx_hex },
    { "WAITTX",        p7501_cmd_waittx },
    { "PWRTRACE",      p7501_cmd_pwrtrace },
    { "BRIDGE",        p7501_cmd_bridge },
    { "BOOTSEL",       cli_cmd_bootsel },
};

void p7501_cli_process_command(char *line) {
    char *trimmed = p7501_cli_trim_whitespace(line);
    if ((trimmed == NULL) || (*trimmed == '\0')) {
        return;
    }

    char raw_cmd[32];
    (void)p7501_next_token(trimmed, raw_cmd, sizeof(raw_cmd));
    char cmd[32];
    p7501_ascii_upper_copy(cmd, sizeof(cmd), raw_cmd);
    const char *arg = cli_extract_arg(trimmed);

    for (size_t i = 0u; i < (sizeof(k_cli_commands) / sizeof(k_cli_commands[0])); ++i) {
        if (strcmp(cmd, k_cli_commands[i].name) == 0) {
            k_cli_commands[i].handler(arg);
            return;
        }
    }
    printf("Unknown command: '%s'. Type HELP for commands.\n", cmd);
}
