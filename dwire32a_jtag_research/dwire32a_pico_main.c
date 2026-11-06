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
    printf("   Host Control : Raspberry Pi Pico (RP2040) via PIO0 Transceiver\n");
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
    printf("  BREAK_IN [0..3]   - Execute Cold Break-In (32 SCLK clocks + reset sync)\n");
    printf("                      0: EXTRG High, Pre-Reset Only (Default)\n");
    printf("                      1: EXTRG Low, Pre-Reset Only (Inverted)\n");
    printf("                      2: EXTRG High, Pre + Post Clock Burst\n");
    printf("                      3: EXTRG Low, Pre + Post Clock Burst\n");
    printf("  SCAN              - Full automated sweep across all modes & opcodes\n");
    printf("  INJECT_LOADER     - Inject AM34 JEDEC RAM loader into SRAM at 0x00004000\n");
    printf("  ERASE_NVRAM       - Execute RAM loader to erase SA134..SA141 (U99 fix)\n");
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
        printf("[ERASE_NVRAM] Initiating Spansion flash NVRAM sector erasure...\n");
        const dwire_status_t status = ram_loader_execute_erase();
        if (status == DWIRE_OK) {
            printf("ERASE_NVRAM: OK (8 micro-sectors 0x007F0000..0x007FFFFF erased, U99 cleared)\n");
        } else {
            printf("ERASE_NVRAM: ERROR (Code %d)\n", (int)status);
        }
    }
    else if (strcmp(cmd, "HELP") == 0) {
        ui_print_help();
    }
    else {
        printf("ERROR: Unknown command '%s'. Type HELP for available commands.\n", cmd);
    }
}

/* =========================================================================
 * Application Entry Point & Non-Blocking CDC Loop
 * ========================================================================= */

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

    /* Sample and display target power reference */
    const float initial_vdd = dwire_hw_read_vdd();
    printf("[Init] Target VDD Rail: %.2f V (%s)\n",
           initial_vdd,
           (initial_vdd >= DWIRE_VDD_THRESHOLD_V) ? "POWERED / OK" : "UNPOWERED / LOW");

    /* Print initial interactive prompt */
    printf("\n%s", DWIRE_PROMPT);

    /* Command input line buffer */
    char cmd_buffer[DWIRE_CMD_BUFFER_LEN];
    size_t cmd_index = 0U;

    /* Main interactive non-blocking serial CDC loop */
    while (true) {
        const int ch = getchar_timeout_us(1000);
        if (ch == PICO_ERROR_TIMEOUT) {
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
