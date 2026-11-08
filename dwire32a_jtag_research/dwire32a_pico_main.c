/**
 * @file dwire32a_pico_main.c
 * @brief Host Firmware for Raspberry Pi Pico (RP2040) controlling Panasonic DWire32A.
 *
 * Implements an interactive USB CDC serial console and automated command protocol
 * for in-circuit debugging, opcode scanning, RAM loader injection, and NVRAM
 * sanitization on the Matsushita MN2DS0016 / AM34 core (Panasonic DMR-EH55
 * Main Digital Board VEP79138).
 *
 * Physical Connections (RP2040 -> DWire32A CN55001):
 *   GP2  (Pin 4)  -> EXTRG  (via 100R series resistor)
 *   GP3  (Pin 5)  -> SCLK   (via 47R..100R series resistor)
 *   GP4  (Pin 6)  -> SDATA  (via 100R series resistor + 4k7 pull-up to 3V3)
 *   GP5  (Pin 7)  -> nRESET (Open-Drain: alternated between Output Low and Input Hi-Z)
 *   GP26 (ADC0)   -> VDD    (via 1k series resistor to 3.3V reference sense)
 *   GND  (Pin 38) -> GND    (Direct common ground connection)
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <inttypes.h>
#include "pico/stdlib.h"
#include "dwire32a_types.h"
#include "dwire32a_hw.h"
#include "ram_loader_payload.h"
#include "gdb_rsp.h"

/* =========================================================================
 * Onboard LED Visual Activity & Heartbeat Subsystem
 * ========================================================================= */

#ifndef PICO_DEFAULT_LED_PIN
#define PICO_DEFAULT_LED_PIN 25U
#endif

typedef enum {
    LED_MODE_HEARTBEAT_IDLE = 0, /* Gentle 1 Hz pulse: Firmware running & USB CDC ready */
    LED_MODE_DWIRE_ACTIVE   = 1, /* Solid ON: Bus transaction / Cold Break-In / Scan active */
    LED_MODE_GDB_RSP        = 2, /* Rapid 5 Hz pulse / toggle on packet: GDB/Ghidra debug session */
    LED_MODE_BUS_FAULT      = 3, /* Triple warning blink: Target VDD < 3.0V or bus grounded */
} pico_led_mode_t;

static pico_led_mode_t s_led_mode = LED_MODE_HEARTBEAT_IDLE;
static uint32_t s_led_last_ms = 0U;
static bool s_led_state = false;

static void led_init(void) {
#if !defined(PICO_HOST_TEST)
    gpio_init(PICO_DEFAULT_LED_PIN);
    gpio_set_dir(PICO_DEFAULT_LED_PIN, GPIO_OUT);
    gpio_put(PICO_DEFAULT_LED_PIN, 0);
#endif
}

static void led_set_mode(pico_led_mode_t mode) {
    s_led_mode = mode;
#if !defined(PICO_HOST_TEST)
    if (mode == LED_MODE_DWIRE_ACTIVE) {
        gpio_put(PICO_DEFAULT_LED_PIN, 1);
        s_led_state = true;
    }
#endif
}

static void led_notify_activity(void) {
#if !defined(PICO_HOST_TEST)
    s_led_state = !s_led_state;
    gpio_put(PICO_DEFAULT_LED_PIN, s_led_state ? 1 : 0);
#endif
}

static void led_tick(void) {
#if !defined(PICO_HOST_TEST)
    const uint32_t now = to_ms_since_boot(get_absolute_time());

    switch (s_led_mode) {
        case LED_MODE_HEARTBEAT_IDLE: {
            const uint32_t phase = now % 1000U;
            const bool target_state = (phase < 100U);
            if (target_state != s_led_state) {
                s_led_state = target_state;
                gpio_put(PICO_DEFAULT_LED_PIN, s_led_state ? 1 : 0);
            }
            break;
        }
        case LED_MODE_DWIRE_ACTIVE: {
            if (!s_led_state) {
                s_led_state = true;
                gpio_put(PICO_DEFAULT_LED_PIN, 1);
            }
            break;
        }
        case LED_MODE_GDB_RSP: {
            if (now - s_led_last_ms >= 100U) {
                s_led_last_ms = now;
                s_led_state = !s_led_state;
                gpio_put(PICO_DEFAULT_LED_PIN, s_led_state ? 1 : 0);
            }
            break;
        }
        case LED_MODE_BUS_FAULT: {
            const uint32_t phase = now % 1200U;
            const bool target_state = (phase < 600U) && (((phase / 100U) % 2U) == 0U);
            if (target_state != s_led_state) {
                s_led_state = target_state;
                gpio_put(PICO_DEFAULT_LED_PIN, s_led_state ? 1 : 0);
            }
            break;
        }
    }
#endif
}

/* =========================================================================
 * Scanner Data Structures & Configuration
 * ========================================================================= */

#define DWIRE_CMD_BUFFER_LEN        128U
#define DWIRE_PROMPT                "DWIRE32A> "

/**
 * @brief Diagnostic statistics gathered during an opcode scan sequence.
 */
typedef struct dwire_scan_stats {
    uint32_t total_probes;
    uint32_t hits_detected;
    uint32_t open_bus_high;
    uint32_t open_bus_low;
} dwire_scan_stats_t;

/* =========================================================================
 * User Interface & Diagnostic Reporting
 * ========================================================================= */

/**
 * @brief Print system banner and connection metadata to USB CDC console.
 */
static void ui_print_banner(void) {
    printf("\n");
    printf("====================================================================\n");
    printf("   PANASONIC DWIRE32A / OCD INTERFACE RECONSTRUCTION ENGINE\n");
    printf("   Architecture : Matsushita AM34 / MN103S (UniPhier 32-bit RISC)\n");
    printf("   Target SoC   : MN2DS0016 (Panasonic DMR-EH55 Digital Board VEP79138)\n");
    printf("   Flash Memory : Spansion S29JL064H70TFI00 (64 Mbit / 8 MB TSOP-48)\n");
    printf("   Host Control : %s\n", gdb_rsp_get_status_info());
    printf("   Status LED   : GP25 (Heartbeat: 1Hz, Active: Solid, GDB: 5Hz, Fault: 3x)\n");
    printf("   C Standard   : ISO C11 Embedded System Architecture\n");
    printf("====================================================================\n");
    printf(" Type HELP for a list of available serial commands.\n\n");
}

/**
 * @brief Print command help menu with syntax and descriptions.
 */
static void ui_print_help(void) {
    printf("\n--- Panasonic DWire32A Command Interface Help ---\n");
    printf("  PING              - Connection heartbeat. Responds: PONG\n");
    printf("  STATUS | VDD      - Sample target 3.3V reference voltage via ADC0\n");
    printf("  HANDSHAKE         - Execute Cold Break-In and verify AM34 OCD DCSR status\n");
    printf("  DUMP [addr] [cnt] - Dump memory words via DWire32A (Default: 0x80000000, 16 words)\n");
    printf("  SNIFF [ms]        - Passively monitor SCLK and SDATA bus activity (Default: 100ms)\n");
    printf("  BREAK_IN [0..3]   - Execute Cold Break-In (32 SCLK clocks + reset sync)\n");
    printf("                      0: EXTRG High, Pre-Reset Only (Default)\n");
    printf("                      1: EXTRG Low, Pre-Reset Only (Inverted)\n");
    printf("                      2: EXTRG High, Pre + Post Clock Burst\n");
    printf("                      3: EXTRG Low, Pre + Post Clock Burst\n");
    printf("  SCAN              - Full automated sweep across all modes & opcodes\n");
    printf("  INJECT_LOADER CONFIRM - Inject AM34 JEDEC RAM loader into SRAM at 0x00004000\n");
    printf("  ERASE_NVRAM CONFIRM   - Execute RAM loader to erase SA134..SA141 (U99 fix)\n");
    printf("  GDB               - Enter GDB Remote Serial Protocol (RSP) server mode\n");
    printf("  LED [0..3]        - Test onboard LED mode (0:Heartbeat, 1:Active, 2:GDB, 3:Fault)\n");
    printf("  HELP              - Display this command reference\n");
    printf("--------------------------------------------------\n\n");
}

/**
 * @brief Print final summary table after opcode scanning completes.
 *
 * @param stats     Pointer to collected scan statistics.
 * @param vdd_volts Measured target VDD rail voltage.
 */
static void ui_print_summary(const dwire_scan_stats_t *stats, float vdd_volts) {
    printf("\n");
    printf("--------------------------------------------------------------------\n");
    printf("   SCAN EXECUTION SUMMARY & DIAGNOSTICS\n");
    printf("--------------------------------------------------------------------\n");
    printf(" Target VDD Rail        : %.2f V (%s)\n",
           vdd_volts,
           (vdd_volts >= DWIRE_VDD_THRESHOLD_V) ? "POWERED / OK" : "UNPOWERED / LOW");
    printf(" Total Probes Executed  : %" PRIu32 "\n", stats->total_probes);
    printf(" Open Bus Pull-Up (0xFF): %" PRIu32 "\n", stats->open_bus_high);
    printf(" Open Bus Ground  (0x00): %" PRIu32 "\n", stats->open_bus_low);
    printf(" Valid Responses (Hits) : %" PRIu32 "\n", stats->hits_detected);
    printf("--------------------------------------------------------------------\n");
    if (stats->hits_detected > 0U) {
        printf(" [SUCCESS] OCD bus activity detected! Review candidate opcodes above.\n");
    } else {
        printf(" [INFO] No active response captured. Verify hardware connections,\n");
        printf("        ensure board is powered, or check Spansion Pin 14 (WP#/ACC).\n");
    }
    printf("====================================================================\n\n");
}

/* =========================================================================
 * Opcode Scanner Subsystem
 * ========================================================================= */

/**
 * @brief Execute systematic sweep across all Cold Break-In modes and opcodes.
 *
 * @param[out] stats Pointer to statistics structure populated during scan.
 */
static void scanner_run_sweep(dwire_scan_stats_t *stats) {
    if (stats == NULL) {
        return;
    }

    stats->total_probes  = 0U;
    stats->hits_detected = 0U;
    stats->open_bus_high = 0U;
    stats->open_bus_low  = 0U;

    printf("[Scanner] Starting automated DWire32A opcode discovery sweep...\n");
    printf("[Scanner] Command range: 0x%02X to 0x%02X (%u commands per mode)\n",
           DWIRE_OPCODE_MIN, DWIRE_OPCODE_MAX, (DWIRE_OPCODE_MAX - DWIRE_OPCODE_MIN + 1U));

    for (uint32_t mode = 0U; mode < (uint32_t)DWIRE_BREAKIN_MODE_COUNT; ++mode) {
        const dwire_breakin_mode_t breakin_mode = (dwire_breakin_mode_t)mode;

        printf("\n[Scanner] ---> Testing Break-In Profile Mode %" PRIu32 " (%s)...\n",
               mode,
               (mode == DWIRE_BREAKIN_MODE_EXTRG_HIGH_PRE_ONLY) ? "EXTRG High, Pre-Reset Only" :
               (mode == DWIRE_BREAKIN_MODE_EXTRG_LOW_PRE_ONLY)  ? "EXTRG Low, Pre-Reset Only" :
               (mode == DWIRE_BREAKIN_MODE_EXTRG_HIGH_PRE_POST) ? "EXTRG High, Pre + Post Burst" :
                                                                  "EXTRG Low, Pre + Post Burst");

        for (uint32_t cmd = DWIRE_OPCODE_MIN; cmd <= DWIRE_OPCODE_MAX; ++cmd) {
            /* Execute cold break-in sequence to capture CPU at reset vector */
            dwire_hw_cold_break_in(breakin_mode);

            /* Transmit candidate 8-bit OCD command word */
            dwire_hw_write_bits(cmd, DWIRE_OPCODE_BIT_LEN);

            /* Receive 32-bit response word from target */
            const uint32_t resp = dwire_hw_read_bits(DWIRE_RESP_BIT_LEN);
            stats->total_probes++;

            /* Filter out open bus states (floating high due to pull-up, or ground short) */
            if (resp == DWIRE_BUS_RESP_OPEN_HIGH) {
                stats->open_bus_high++;
            } else if (resp == DWIRE_BUS_RESP_OPEN_LOW) {
                stats->open_bus_low++;
            } else {
                /* Valid active bus response detected */
                stats->hits_detected++;
                printf("  [HIT!] Mode=%" PRIu32 " | CMD=0x%02" PRIX32 " | RESP=0x%08" PRIX32 "\n",
                       mode, cmd, resp);
            }
        }
    }

    printf("\n[Scanner] Opcode sweep finished.\n");
}

/* =========================================================================
 * Interactive & Automated Command Processing Engine
 * ========================================================================= */

/**
 * @brief Trim leading and trailing whitespace from string.
 *
 * @param str In-out pointer to character buffer.
 * @return Pointer to trimmed string.
 */
static char *trim_whitespace(char *str) {
    if (str == NULL) {
        return NULL;
    }
    while (isspace((unsigned char)*str)) {
        str++;
    }
    if (*str == '\0') {
        return str;
    }
    char *end = str + strlen(str) - 1;
    while (end > str && isspace((unsigned char)*end)) {
        *end = '\0';
        end--;
    }
    return str;
}

/**
 * @brief Execute a parsed command line from serial input.
 *
 * @param line Input line buffer.
 */
static void process_serial_command(char *line) {
    char *trimmed = trim_whitespace(line);
    if (trimmed == NULL || *trimmed == '\0') {
        return;
    }

    /* Split command token and argument */
    char *cmd = trimmed;
    char *arg = NULL;
    char *space = strchr(trimmed, ' ');
    if (space != NULL) {
        *space = '\0';
        arg = trim_whitespace(space + 1);
    }

    /* Convert command to uppercase */
    for (char *p = cmd; *p != '\0'; ++p) {
        *p = (char)toupper((unsigned char)*p);
    }

    /* Command Dispatcher */
    if (strcmp(cmd, "PING") == 0) {
        printf("PONG\n");
    }
    else if (strcmp(cmd, "STATUS") == 0 || strcmp(cmd, "VDD") == 0) {
        const float vdd = dwire_hw_read_vdd();
        const bool powered = (vdd >= DWIRE_VDD_THRESHOLD_V);
        printf("VDD: %.2fV (%s)\n", vdd, powered ? "OK" : "LOW");
    }
    else if (strcmp(cmd, "HANDSHAKE") == 0) {
        printf("[HANDSHAKE] Executing Cold Break-In and querying AM34 OCD DCSR...\n");
        const float vdd = dwire_hw_read_vdd();
        if (vdd < DWIRE_VDD_THRESHOLD_V) {
            printf("[HANDSHAKE] ABORT: Target VDD is %.2fV (< %.2fV threshold). Is the recorder powered on?\n",
                   vdd, DWIRE_VDD_THRESHOLD_V);
        } else {
            dwire_hw_cold_break_in(DWIRE_BREAKIN_MODE_EXTRG_HIGH_PRE_ONLY);
            const uint32_t dcsr = dwire_hw_read_dcsr();
            const bool halted = (dcsr != DWIRE_BUS_RESP_OPEN_HIGH &&
                                 dcsr != DWIRE_BUS_RESP_OPEN_LOW &&
                                 ((dcsr & 0x00000001UL) != 0U));
            printf("DCSR: 0x%08" PRIX32 "\n", dcsr);
            if (dcsr == DWIRE_BUS_RESP_OPEN_HIGH) {
                printf("[HANDSHAKE] FAILED: Bus floating High (0xFFFFFFFF). Check SCLK / SDATA / GND lines.\n");
            } else if (dcsr == DWIRE_BUS_RESP_OPEN_LOW) {
                printf("[HANDSHAKE] FAILED: Bus shorted to Ground (0x00000000). Check for wiring short-circuit.\n");
            } else if (halted) {
                printf("[HANDSHAKE] SUCCESS! Matsushita AM34 core is HALTED in On-Chip Debug mode.\n");
            } else {
                printf("[HANDSHAKE] ACTIVE: Bus responded (DCSR=0x%08" PRIX32 ") but CPU halt bit not set.\n", dcsr);
            }
        }
    }
    else if (strcmp(cmd, "DUMP") == 0) {
        /* Syntax: DUMP [hex_addr] [count_words] */
        uint32_t addr = 0x80000000UL; /* Default: Base of uncached Spansion Flash aperture */
        uint32_t count = 16U;         /* Default: 16 words (64 bytes) */

        if (arg != NULL && *arg != '\0') {
            char *next = NULL;
            addr = (uint32_t)strtoul(arg, &next, 16);
            if (next != NULL && *next != '\0') {
                char *cnt_str = trim_whitespace(next);
                if (cnt_str != NULL && *cnt_str != '\0') {
                    count = (uint32_t)strtoul(cnt_str, NULL, 0);
                }
            }
        }
        addr &= ~0x03UL; /* Enforce 32-bit word alignment on AM34 bus */

        if (count == 0U) {
            count = 1U;
        }
        if (count > 1024U) {
            count = 1024U; /* Protect against inadvertent massive dump requests */
        }

        printf("[DUMP] Reading %" PRIu32 " words (%" PRIu32 " bytes) from 0x%08" PRIX32 "...\n",
               count, count * 4U, addr);

        for (uint32_t i = 0U; i < count; ++i) {
            const uint32_t curr_addr = addr + (i * 4U);
            const uint32_t val = dwire_hw_read_mem32(curr_addr);
            if ((i % 4U) == 0U) {
                printf("\n%08" PRIX32 ": ", curr_addr);
            }
            printf("%08" PRIX32 " ", val);
        }
        printf("\n\nDUMP: OK\n");
    }
    else if (strcmp(cmd, "SNIFF") == 0) {
        uint32_t duration_ms = 100U;
        if (arg != NULL && *arg != '\0') {
            duration_ms = (uint32_t)atoi(arg);
            if (duration_ms == 0U) duration_ms = 100U;
            if (duration_ms > 10000U) duration_ms = 10000U;
        }

        printf("[SNIFF] Passively monitoring SDATA (GP4) and SCLK (GP3) for %" PRIu32 " ms...\n", duration_ms);
        /* Safely detach pins from PIO to SIO as high-impedance inputs with pulls disabled */
        dwire_hw_detach_pins_for_sniff();

        uint32_t sclk_transitions = 0U;
        uint32_t sdata_transitions = 0U;
        bool last_sclk = gpio_get(DWIRE_PIN_SCLK);
        bool last_sdata = gpio_get(DWIRE_PIN_SDATA);

        absolute_time_t end_time = make_timeout_time_ms(duration_ms);
        while (get_absolute_time() < end_time) {
            bool cur_sclk = gpio_get(DWIRE_PIN_SCLK);
            bool cur_sdata = gpio_get(DWIRE_PIN_SDATA);
            if (cur_sclk != last_sclk) {
                sclk_transitions++;
                last_sclk = cur_sclk;
            }
            if (cur_sdata != last_sdata) {
                sdata_transitions++;
                last_sdata = cur_sdata;
            }
        }

        /* Safely re-attach pins to the PIO state machine */
        dwire_hw_attach_pins_after_sniff();

        printf("SNIFF RESULT (%" PRIu32 " ms):\n", duration_ms);
        printf("  SCLK (GP3)  : Initial=%d, Transitions=%" PRIu32 "\n", last_sclk ? 1 : 0, sclk_transitions);
        printf("  SDATA (GP4) : Initial=%d, Transitions=%" PRIu32 "\n", last_sdata ? 1 : 0, sdata_transitions);
        if (sclk_transitions == 0U && sdata_transitions == 0U) {
            if (last_sclk && last_sdata) {
                printf("  [STATUS] Lines idle HIGH (3.3V). Target is powered and pull-ups are intact.\n");
            } else if (!last_sclk && !last_sdata) {
                printf("  [STATUS] Lines steady LOW (0V). Target may be unpowered or lines shorted to GND.\n");
            } else {
                printf("  [STATUS] Lines static: SCLK=%d, SDATA=%d.\n", last_sclk ? 1 : 0, last_sdata ? 1 : 0);
            }
        } else {
            printf("  [STATUS] ACTIVE BUS ACTIVITY DETECTED! Clock transitions: %" PRIu32 "\n", sclk_transitions);
        }
        printf("SNIFF: OK\n");
    }
    else if (strcmp(cmd, "BREAK_IN") == 0) {
        uint32_t mode = 0U;
        if (arg != NULL && *arg != '\0') {
            mode = (uint32_t)atoi(arg);
            if (mode >= (uint32_t)DWIRE_BREAKIN_MODE_COUNT) {
                printf("[BREAK_IN] Invalid mode %" PRIu32 "! Defaulting to mode 0.\n", mode);
                mode = 0U;
            }
        }
        printf("[BREAK_IN] Executing Cold Break-In mode %" PRIu32 " (%s)...\n",
               mode,
               (mode == 0U) ? "EXTRG High, Pre-Reset" :
               (mode == 1U) ? "EXTRG Low, Pre-Reset" :
               (mode == 2U) ? "EXTRG High, Pre+Post Burst" : "EXTRG Low, Pre+Post Burst");

        dwire_hw_cold_break_in((dwire_breakin_mode_t)mode);
        printf("BREAK_IN: OK (Mode %" PRIu32 ")\n", mode);
    }
    else if (strcmp(cmd, "SCAN") == 0) {
        dwire_scan_stats_t scan_stats;
        const float vdd = dwire_hw_read_vdd();
        scanner_run_sweep(&scan_stats);
        ui_print_summary(&scan_stats, vdd);
        printf("SCAN: COMPLETED (Hits: %" PRIu32 ", Probes: %" PRIu32 ")\n",
               scan_stats.hits_detected, scan_stats.total_probes);
    }
    else if (strcmp(cmd, "INJECT_LOADER") == 0) {
        if (arg == NULL || strcmp(arg, "CONFIRM") != 0) {
            printf("[INJECT_LOADER] SAFETY INTERLOCK ACTIVE!\n");
            printf("  CRITICAL: Target SRAM code injection requested.\n");
            printf("  Mandatory protocol: Extract a verified 8MB flash backup (DUMP) first\n");
            printf("  and verify disassembly before executing code in target SRAM.\n");
            printf("  To execute, explicitly confirm with: INJECT_LOADER CONFIRM\n");
            return;
        }
        printf("[INJECT_LOADER] Initiating SRAM transfer via DWire32A...\n");
        const dwire_status_t status = ram_loader_inject(AM34_SRAM_LOADER_BASE_ADDR, true);
        if (status == DWIRE_OK) {
            printf("INJECT_LOADER: OK (%u bytes written, %u words verified)\n",
                   (unsigned)AM34_RAM_LOADER_SIZE_BYTES, (unsigned)AM34_RAM_LOADER_WORD_COUNT);
        } else {
            printf("INJECT_LOADER: ERROR (Code %d)\n", (int)status);
        }
    }
    else if (strcmp(cmd, "ERASE_NVRAM") == 0) {
        if (arg == NULL || strcmp(arg, "CONFIRM") != 0) {
            printf("[ERASE_NVRAM] SAFETY INTERLOCK ACTIVE!\n");
            printf("  CRITICAL: Destructive flash sector erasure requested.\n");
            printf("  Engineering protocol requires a verified full read backup (DUMP)\n");
            printf("  before performing any write/erase operations.\n");
            printf("  To execute, explicitly confirm with: ERASE_NVRAM CONFIRM\n");
            return;
        }
        printf("[ERASE_NVRAM] Initiating Spansion flash NVRAM sector erasure...\n");
        const dwire_status_t status = ram_loader_execute_erase();
        if (status == DWIRE_OK) {
            printf("ERASE_NVRAM: OK (8 micro-sectors 0x007F0000..0x007FFFFF erased, U99 cleared)\n");
        } else {
            printf("ERASE_NVRAM: ERROR (Code %d)\n", (int)status);
        }
    }
    else if (strcmp(cmd, "GDB") == 0) {
        printf("[GDB RSP] Entering GDB Remote Serial Protocol mode (%s).\n", gdb_rsp_get_status_info());
        printf("[GDB RSP] Auto-ready for mn10300-elf-gdb / gdb-multiarch.\n");
        gdb_rsp_set_active(true);
        return;
    }
    else if (strcmp(cmd, "LED") == 0) {
        uint32_t mode = 0U;
        if (arg != NULL && *arg != '\0') {
            mode = (uint32_t)atoi(arg);
        }
        if (mode > 3U) mode = 0U;
        led_set_mode((pico_led_mode_t)mode);
        printf("[LED] Mode set to %" PRIu32 " (%s)\n",
               mode,
               (mode == 0U) ? "HEARTBEAT 1Hz" :
               (mode == 1U) ? "ACTIVE SOLID" :
               (mode == 2U) ? "GDB 5Hz" : "BUS FAULT TRIPLE BLINK");
    }
    else if (strcmp(cmd, "HELP") == 0) {
        ui_print_help();
    }
    else {
        printf("ERROR: Unknown command '%s'. Type HELP for available commands.\n", cmd);
    }
}

// ---------------------------------------------------------------------------
// Section: Application Entry Point & Non-Blocking CDC Loop
// ---------------------------------------------------------------------------

int main(void) {
    /* Initialize USB CDC standard I/O */
    stdio_init_all();

    /* Allow host USB enumeration delay */
    sleep_ms(DWIRE_HOST_BOOT_DELAY_MS);

    /* Display engine banner */
    ui_print_banner();

    /* Initialize hardware layer (GPIOs, ADC0, PIO state machine) */
    const dwire_status_t init_status = dwire_hw_init();
    if (init_status != DWIRE_OK) {
        printf("[FATAL] Hardware initialization failed (error code %d)!\n", (int)init_status);
        while (true) {
            tight_loop_contents();
        }
    }
    printf("[Init] Hardware abstraction layer initialized successfully.\n");

    /* Initialize GDB Remote Serial Protocol subsystem */
    gdb_rsp_init();
    printf("[Init] GDB RSP server ready: %s\n", gdb_rsp_get_status_info());

    /* Sample and display target power reference */
    const float initial_vdd = dwire_hw_read_vdd();
    printf("[Init] Target VDD Rail: %.2f V (%s)\n",
           initial_vdd,
           (initial_vdd >= DWIRE_VDD_THRESHOLD_V) ? "POWERED / OK" : "UNPOWERED / LOW");

    /* Initialize onboard LED controller */
    led_init();
    if (initial_vdd >= DWIRE_VDD_THRESHOLD_V) {
        led_set_mode(LED_MODE_HEARTBEAT_IDLE);
    } else {
        led_set_mode(LED_MODE_BUS_FAULT);
    }

    /* Print initial interactive prompt */
    printf("\n%s", DWIRE_PROMPT);

    /* Command input line buffer */
    char cmd_buffer[DWIRE_CMD_BUFFER_LEN];
    size_t cmd_index = 0U;

    /* Main interactive non-blocking serial CDC loop */
    while (true) {
        led_tick();
        const int ch = getchar_timeout_us(1000);
        if (ch == PICO_ERROR_TIMEOUT) {
            continue;
        }

        /* If GDB RSP mode is currently active, route bytes directly to RSP parser */
        if (gdb_rsp_is_active()) {
            led_set_mode(LED_MODE_GDB_RSP);
            led_notify_activity();
            gdb_rsp_process_char((char)ch);
            continue;
        }

        /* Auto-switch to GDB RSP mode on standard GDB packet header '$' or Ctrl-C */
        if (ch == '$' || ch == 0x03) {
            led_set_mode(LED_MODE_GDB_RSP);
            led_notify_activity();
            gdb_rsp_set_active(true);
            gdb_rsp_process_char((char)ch);
            continue;
        }

        /* Handle carriage return and newline (Command Execution) */
        if (ch == '\r' || ch == '\n') {
            if (cmd_index > 0U) {
                cmd_buffer[cmd_index] = '\0';
                putchar('\n');
                process_serial_command(cmd_buffer);
                cmd_index = 0U;
            } else {
                putchar('\n');
            }
            printf("%s", DWIRE_PROMPT);
        }
        /* Handle Backspace / Delete */
        else if (ch == '\b' || ch == 127) {
            if (cmd_index > 0U) {
                cmd_index--;
                /* Erase character from terminal: backspace, space, backspace */
                printf("\b \b");
            }
        }
        /* Handle regular printable ASCII characters */
        else if (ch >= 32 && ch <= 126) {
            if (cmd_index < (DWIRE_CMD_BUFFER_LEN - 1U)) {
                cmd_buffer[cmd_index++] = (char)ch;
                putchar(ch); /* Echo to terminal */
            }
        }
    }

    return 0;
}
