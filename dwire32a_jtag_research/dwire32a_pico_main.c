/**
 * @file dwire32a_pico_main.c
 * @brief Host Firmware for Raspberry Pi Pico (RP2040) controlling Panasonic DWire32A.
 *
 * Implements an interactive USB CDC serial console and automated command protocol
 * for in-circuit debugging, opcode scanning, RAM loader injection, and NVRAM
 * sanitization on the Matsushita UniPhier AM34 / MN103S SoC family across
 * Panasonic DIGA recorders (DMR-E / DMR-ExxH / DMR-ES / DMR-EH / DMR-EX / DMR-EZ).
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
#include "dwire32a_led.h"

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

/**
 * @brief Flash geometry metadata resolved via passive address aliasing.
 */
typedef struct {
    uint32_t bytes;
    uint32_t mb;
    uint32_t nvram_size;
    uint32_t nvram_base;
} flash_geometry_info_t;

/**
 * @brief Discovers Flash geometry and synchronizes GDB memory window limits.
 */
static inline flash_geometry_info_t dwire_probe_flash_geometry(void) {
    flash_geometry_info_t info;
    info.bytes = dwire_hw_autodetect_flash_size();
    (void)gdb_rsp_set_flash_size(info.bytes);
    info.mb = info.bytes / (1024UL * 1024UL);
    info.nvram_size = (info.bytes == 0x01000000UL) ? 0x00020000UL : 0x00010000UL;
    info.nvram_base = 0x80000000UL + (info.bytes - info.nvram_size);
    return info;
}

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
    printf("   Target Family: Panasonic DIGA (DMR-E / ExxH / ES / EH / EX / EZ)\n");
    printf("   NOR Flash    : Multi-Geometry Top-Boot (2MB / 4MB / 8MB / 16MB)\n");
    printf("   Host Control : %s\n", gdb_rsp_get_status_info());
#if defined(PANADIGA_BLE_ENABLED)
    printf("   Wireless     : CYW43439 Bluetooth LE NUS (Nordic UART Service Enabled)\n");
#else
    printf("   Wireless     : Disabled (USB CDC Only / Standard RP2040/RP2350)\n");
#endif
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
    printf("  ISOLATE | HIZ     - Tri-State (Hi-Z) all 6 pins so unified plug can stay connected\n");
    printf("  ENGAGE            - Re-engage active PIO drive on the 6-pin connector\n");
    printf("  AUTODETECT        - Passive read-only address aliasing probe (2MB / 4MB / 8MB / 16MB)\n");
    printf("  FLASH_SIZE [2|4|8|16] - Configure target NOR Flash size in MB (Default: Auto / 8MB)\n");
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
        printf("[HANDSHAKE] Executing Multi-Mode Cold Break-In and querying AM34 OCD DCSR...\n");
        const float vdd = dwire_hw_read_vdd();
        const bool probe_floating = (vdd >= DWIRE_VDD_FLOATING_CEIL_V);
        const bool target_unpowered = (!probe_floating && (vdd < DWIRE_VDD_THRESHOLD_V));

        if (probe_floating) {
            printf("[HANDSHAKE] ABORT: VDD reads %.2fV (>= %.2fV ceiling). "
                   "ADC input is FLOATING — probe cable is NOT connected to CN55001 "
                   "or the 1k series resistor on GP26 is missing/open.\n",
                   vdd, DWIRE_VDD_FLOATING_CEIL_V);
        } else if (target_unpowered) {
            printf("[HANDSHAKE] ABORT: Target VDD is %.2fV (< %.2fV threshold). "
                   "Is the Panasonic recorder powered on?\n",
                   vdd, DWIRE_VDD_THRESHOLD_V);
        } else {
            static const dwire_breakin_mode_t priority_modes[4] = {
                DWIRE_BREAKIN_MODE_EXTRG_HIGH_PRE_POST,
                DWIRE_BREAKIN_MODE_EXTRG_HIGH_PRE_ONLY,
                DWIRE_BREAKIN_MODE_EXTRG_LOW_PRE_POST,
                DWIRE_BREAKIN_MODE_EXTRG_LOW_PRE_ONLY
            };
            uint32_t dcsr = DWIRE_BUS_RESP_OPEN_HIGH;
            bool halted = false;
            uint32_t matched_mode = 0U;

            for (size_t i = 0U; i < 4U; ++i) {
                const dwire_breakin_mode_t m = priority_modes[i];
                dwire_hw_cold_break_in(m);
                dcsr = dwire_hw_read_dcsr();
                halted = (dcsr != DWIRE_BUS_RESP_OPEN_HIGH &&
                          dcsr != DWIRE_BUS_RESP_OPEN_LOW &&
                          ((dcsr & 0x00000001UL) != 0U));
                if (halted) {
                    matched_mode = (uint32_t)m;
                    break;
                }
            }

            printf("DCSR: 0x%08" PRIX32 "\n", dcsr);
            if (dcsr == DWIRE_BUS_RESP_OPEN_HIGH) {
                printf("[HANDSHAKE] FAILED: Bus floating High (0xFFFFFFFF). Check SCLK / SDATA / GND lines.\n");
            } else if (dcsr == DWIRE_BUS_RESP_OPEN_LOW) {
                printf("[HANDSHAKE] FAILED: Bus shorted to Ground (0x00000000). Check for wiring short-circuit.\n");
            } else if (halted) {
                const flash_geometry_info_t geo = dwire_probe_flash_geometry();
                printf("[AUTODETECT] Passive Address Aliasing Probe: %" PRIu32 " MB NOR Flash detected (EEMUL NVRAM @ 0x%08" PRIX32 ").\n",
                       geo.mb, geo.nvram_base);
                printf("[HANDSHAKE] SUCCESS! Matsushita AM34 core is HALTED in On-Chip Debug mode (Mode %" PRIu32 ").\n",
                       matched_mode);
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
        if (count > 32768U) {
            count = 32768U; /* Allow full 128KB NVRAM (32,768 words) dump in a single command */
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
        const dwire_status_t status = ram_loader_inject_for_flash_size(
            AM34_SRAM_LOADER_BASE_ADDR,
            true,
            gdb_rsp_get_flash_size()
        );
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
        const dwire_status_t status = ram_loader_execute_erase_for_flash_size(gdb_rsp_get_flash_size());
        if (status == DWIRE_OK) {
            const uint32_t eff_size = gdb_rsp_get_flash_size();
            const uint32_t nvram_size = (eff_size == 0x01000000UL) ? 0x00020000UL : 0x00010000UL;
            const uint32_t nvram_base = eff_size - nvram_size;
            const uint32_t micro_count = nvram_size / 0x2000UL;
            printf("ERASE_NVRAM: OK (%" PRIu32 " micro-sectors 0x%08" PRIX32 "..0x%08" PRIX32 " erased, U99 cleared)\n",
                   micro_count, nvram_base, eff_size - 1UL);
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
    else if (strcmp(cmd, "ISOLATE") == 0 || strcmp(cmd, "HIZ") == 0) {
        dwire_hw_isolate_bus_hiz();
        printf("ISOLATE: OK (All 6 pins in High-Impedance Hi-Z mode; unified JST ZR-6 plug is electrically invisible)\n");
    }
    else if (strcmp(cmd, "ENGAGE") == 0) {
        dwire_hw_engage_bus_active();
        printf("ENGAGE: OK (Active PIO OCD bus drive restored on 6-pin connector)\n");
    }
    else if (strcmp(cmd, "AUTODETECT") == 0) {
        const flash_geometry_info_t geo = dwire_probe_flash_geometry();
        printf("AUTODETECT: OK (%" PRIu32 " MB NOR Flash | EEMUL NVRAM: 0x%08" PRIX32 "..0x%08" PRIX32 ")\n",
               geo.mb, geo.nvram_base, 0x80000000UL + geo.bytes - 1UL);
    }
    else if (strcmp(cmd, "CFI") == 0) {
        uint32_t cfi_size = 0U;
        if (dwire_hw_query_cfi(&cfi_size)) {
            const uint32_t mb = cfi_size / (1024UL * 1024UL);
            const uint32_t nvram_size = (cfi_size == 0x01000000UL) ? 0x00020000UL : 0x00010000UL;
            const uint32_t nvram_base = 0x80000000UL + (cfi_size - nvram_size);
            (void)gdb_rsp_set_flash_size(cfi_size);
            printf("CFI: OK ('QRY' detected | Device Size: %" PRIu32 " MB | NVRAM @ 0x%08" PRIX32 ")\n",
                   mb, nvram_base);
        } else {
            printf("CFI: FAILED (No 'QRY' signature detected; chip may be in uninitialized state or non-CFI)\n");
        }
    }
    else if (strcmp(cmd, "FLASH_SIZE") == 0) {
        uint32_t mb = 8U;
        if (arg != NULL && *arg != '\0') {
            mb = (uint32_t)strtoul(arg, NULL, 10);
        }
        const uint32_t bytes = mb * 1024UL * 1024UL;
        if (gdb_rsp_set_flash_size(bytes)) {
            const uint32_t nvram_size = (bytes == 0x01000000UL) ? 0x00020000UL : 0x00010000UL;
            const uint32_t nvram_base = 0x80000000UL + (bytes - nvram_size);
            printf("FLASH_SIZE: OK (%" PRIu32 " MB configured; Top-Boot %" PRIu32 "KB EEMUL NVRAM @ 0x%08" PRIX32 ")\n",
                   mb, nvram_size / 1024UL, nvram_base);
        } else {
            printf("FLASH_SIZE: ERROR (Invalid size %" PRIu32 " MB; valid options: 2, 4, 8, 16)\n", mb);
        }
    }
    else if (strcmp(cmd, "LED") == 0) {
        uint32_t mode = 0U;
        if (arg != NULL && *arg != '\0') {
            mode = (uint32_t)atoi(arg);
        }
        if (mode > 3U) mode = 0U;
        dwire_led_set_mode((pico_led_mode_t)mode);
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
    dwire_led_init();
    if (initial_vdd >= DWIRE_VDD_THRESHOLD_V) {
        dwire_led_set_mode(LED_MODE_HEARTBEAT_IDLE);
    } else {
        dwire_led_set_mode(LED_MODE_BUS_FAULT);
    }

    /* Print initial interactive prompt */
    printf("\n%s", DWIRE_PROMPT);

    /* Command input line buffer */
    char cmd_buffer[DWIRE_CMD_BUFFER_LEN];
    size_t cmd_index = 0U;

    /* Main interactive non-blocking serial CDC loop */
    while (true) {
        dwire_led_tick();
        const int ch = getchar_timeout_us(1000);
        if (ch == PICO_ERROR_TIMEOUT) {
            continue;
        }

        /* If GDB RSP mode is currently active, route bytes directly to RSP parser */
        if (gdb_rsp_is_active()) {
            dwire_led_set_mode(LED_MODE_GDB_RSP);
            dwire_led_notify_activity();
            gdb_rsp_process_char((char)ch);
            continue;
        }

        /* Auto-switch to GDB RSP mode on standard GDB packet header '$' or Ctrl-C */
        if (ch == '$' || ch == 0x03) {
            dwire_led_set_mode(LED_MODE_GDB_RSP);
            dwire_led_notify_activity();
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
