/**
 * @file dwire32a_diag_ocd.c
 * @brief Implementation of AM34 OCD opcode sweeping, Cold Break-In handshake, and memory dumps.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "diag/dwire32a_diag_ocd.h"
#include "cli/dwire32a_cli.h"
#include "flash/dwire32a_flash.h"
#include "flash/ram_loader_payload.h"
#include "hal/dwire32a_hw.h"
#include "protocol/dwire32a_protocol.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>

static const char *breakin_mode_label(uint32_t mode) {
    switch (mode) {
        case DWIRE_BREAKIN_MODE_EXTRG_HIGH_PRE_ONLY: return "EXTRG High, Pre-Reset Only";
        case DWIRE_BREAKIN_MODE_EXTRG_LOW_PRE_ONLY:  return "EXTRG Low, Pre-Reset Only";
        case DWIRE_BREAKIN_MODE_EXTRG_HIGH_PRE_POST: return "EXTRG High, Pre + Post Burst";
        default:                                     return "EXTRG Low, Pre + Post Burst";
    }
}

void dwire_diag_run_opcode_sweep(dwire_scan_stats_t *stats) {
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
               mode, breakin_mode_label(mode));

        for (uint32_t cmd = DWIRE_OPCODE_MIN; cmd <= DWIRE_OPCODE_MAX; ++cmd) {
            dwire_hw_cold_break_in(breakin_mode);
            dwire_hw_write_bits(cmd, DWIRE_OPCODE_BIT_LEN);
            const uint32_t resp = dwire_hw_read_bits(DWIRE_RESP_BIT_LEN);
            stats->total_probes++;

            if (resp == DWIRE_BUS_RESP_OPEN_HIGH) {
                stats->open_bus_high++;
            } else if (resp == DWIRE_BUS_RESP_OPEN_LOW) {
                stats->open_bus_low++;
            } else {
                stats->hits_detected++;
                printf("  [HIT!] Mode=%" PRIu32 " | CMD=0x%02" PRIX32 " | RESP=0x%08" PRIX32 "\n",
                       mode, cmd, resp);
            }
        }
    }

    printf("\n[Scanner] Opcode sweep finished.\n");
}

static void print_scan_summary(const dwire_scan_stats_t *stats, float vdd_volts) {
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

void dwire_diag_cmd_scan(void) {
    dwire_scan_stats_t scan_stats;
    const float vdd = dwire_hw_read_vdd();
    dwire_diag_run_opcode_sweep(&scan_stats);
    print_scan_summary(&scan_stats, vdd);
    printf("SCAN: COMPLETED (Hits: %" PRIu32 ", Probes: %" PRIu32 ")\n",
           scan_stats.hits_detected, scan_stats.total_probes);
}

void dwire_diag_cmd_handshake(void) {
    printf("[HANDSHAKE] Executing Multi-Mode Cold Break-In and querying AM34 OCD DCSR...\n");
    uint32_t dcsr = DWIRE_BUS_RESP_OPEN_HIGH;
    uint32_t matched_mode = 0U;
    const bool halted = dwire_proto_execute_breakin_handshake(&dcsr, &matched_mode);

    printf("DCSR: 0x%08" PRIX32 "\n", dcsr);
    if (dcsr == DWIRE_BUS_RESP_OPEN_HIGH) {
        printf("[HANDSHAKE] FAILED: Bus floating High (0xFFFFFFFF). Check SCLK / SDATA / GND lines.\n");
    } else if (dcsr == DWIRE_BUS_RESP_OPEN_LOW) {
        printf("[HANDSHAKE] FAILED: Bus shorted to Ground (0x00000000). Check for wiring short-circuit.\n");
    } else if (halted) {
        const dwire_flash_geometry_t geo = dwire_probe_flash_geometry();
        printf("[AUTODETECT] Passive Address Aliasing Probe: %" PRIu32 " MB NOR Flash detected (EEMUL NVRAM @ 0x%08" PRIX32 ").\n",
               geo.mb, geo.nvram_base);
        printf("[HANDSHAKE] SUCCESS! Matsushita AM34 core is HALTED in On-Chip Debug mode (Mode %" PRIu32 ").\n",
               matched_mode);
    } else {
        printf("[HANDSHAKE] ACTIVE: Bus responded (DCSR=0x%08" PRIX32 ") but CPU halt bit not set.\n", dcsr);
    }
}

void dwire_diag_cmd_dump(const char *arg) {
    uint32_t addr = SPANSION_FLASH_UNCACHED_BASE;
    uint32_t count = 16U;

    if (arg != NULL && *arg != '\0') {
        char *next = NULL;
        addr = (uint32_t)strtoul(arg, &next, 16);
        if (next != NULL && *next != '\0') {
            char *cnt_str = dwire_cli_trim_whitespace(next);
            if (cnt_str != NULL && *cnt_str != '\0') {
                count = (uint32_t)strtoul(cnt_str, NULL, 0);
            }
        }
    }
    addr &= ~0x03UL;

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

void dwire_diag_cmd_break_in(const char *arg) {
    uint32_t mode = 0U;
    if (arg != NULL && *arg != '\0') {
        mode = (uint32_t)atoi(arg);
        if (mode >= (uint32_t)DWIRE_BREAKIN_MODE_COUNT) {
            printf("[BREAK_IN] Invalid mode %" PRIu32 "! Defaulting to mode 0.\n", mode);
            mode = 0U;
        }
    }
    printf("[BREAK_IN] Executing Cold Break-In mode %" PRIu32 " (%s)...\n",
           mode, breakin_mode_label(mode));

    dwire_hw_cold_break_in((dwire_breakin_mode_t)mode);
    printf("BREAK_IN: OK (Mode %" PRIu32 ")\n", mode);
}
