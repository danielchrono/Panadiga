/**
 * @file p7501_diag_bus.c
 * @brief Implementation of P7501 boot sequencing, monitor sweep, and aperture memory dump.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "diag/p7501_diag_bus.h"
#include "cli/p7501_cli.h"
#include "flash/p7501_flash.h"
#include "flash/ram_loader_payload.h"
#include "hal/p7501_hw.h"
#include "protocol/p7501_bus_protocol.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>

static const char *boot_mode_label(uint32_t mode) {
    switch (mode) {
        case P7501_BOOT_MODE_NORMAL_HIZ:         return "SW_UART Hi-Z, XRESET Pulse";
        case P7501_BOOT_MODE_SERIAL_LOW_RELEASE: return "SW_UART Low -> Hi-Z Release";
        case P7501_BOOT_MODE_SERIAL_LOW_HOLD:    return "SW_UART Low Hold (XFWE JIG)";
        default:                                 return "SW_UART Hi-Z Extended Settle";
    }
}

void p7501_diag_run_monitor_sweep(p7501_scan_stats_t *stats) {
    if (stats == NULL) {
        return;
    }

    stats->total_probes  = 0U;
    stats->hits_detected = 0U;
    stats->open_bus_high = 0U;
    stats->open_bus_low  = 0U;

    printf("[Scanner] Starting P7501 boot profile & UART monitor sweep...\n");
    printf("[Scanner] Probe range: 0x%02X to 0x%02X (%u probes per profile)\n",
           P7501_PROBE_MIN, P7501_PROBE_MAX, (P7501_PROBE_MAX - P7501_PROBE_MIN + 1U));

    for (uint32_t mode = 0U; mode < (uint32_t)P7501_BOOT_MODE_COUNT; ++mode) {
        const p7501_boot_mode_t boot_mode = (p7501_boot_mode_t)mode;
        printf("\n[Scanner] ---> Testing Boot Profile Mode %" PRIu32 " (%s)...\n",
               mode, boot_mode_label(mode));

        for (uint32_t cmd = P7501_PROBE_MIN; cmd <= P7501_PROBE_MAX; ++cmd) {
            p7501_hw_boot_sequence(boot_mode);
            const uint32_t resp = p7501_proto_probe_opcode((uint8_t)cmd);
            stats->total_probes++;

            if (resp == P7501_BUS_RESP_OPEN_HIGH) {
                stats->open_bus_high++;
            } else if (resp == P7501_BUS_RESP_OPEN_LOW) {
                stats->open_bus_low++;
            } else {
                stats->hits_detected++;
                printf("  [HIT!] Mode=%" PRIu32 " | CMD=0x%02" PRIX32 " | RESP=0x%08" PRIX32 "\n",
                       mode, cmd, resp);
            }
        }
    }

    p7501_hw_isolate_bus_hiz();
    printf("\n[Scanner] Monitor sweep finished.\n");
}

static void print_scan_summary(const p7501_scan_stats_t *stats, float vdd_volts) {
    printf("\n--------------------------------------------------------------------\n");
    printf("   P7501 SCAN EXECUTION SUMMARY & DIAGNOSTICS\n");
    printf("--------------------------------------------------------------------\n");
    printf(" SW_UART Sense (GP26)   : %.2f V (%s)\n",
           vdd_volts,
           (vdd_volts >= P7501_VDD_THRESHOLD_V) ? "POWERED / OK" : "UNPOWERED / LOW");
    printf(" Total Probes Executed  : %" PRIu32 "\n", stats->total_probes);
    printf(" Open Idle High (0xFF)  : %" PRIu32 "\n", stats->open_bus_high);
    printf(" Open Grounded  (0x00)  : %" PRIu32 "\n", stats->open_bus_low);
    printf(" Valid Monitor Hits     : %" PRIu32 "\n", stats->hits_detected);
    printf("--------------------------------------------------------------------\n");
    if (stats->hits_detected > 0U) {
        printf(" [SUCCESS] Target monitor activity detected! Review hits above.\n");
    } else {
        printf(" [INFO] P7501 is async 115200 8N1 UART. Use BOOTCAP / UARTMON / UARTSEND / FMUSCAN\n");
        printf("        to inspect AM34 Bootrom and main console frames directly.\n");
    }
    printf("====================================================================\n\n");
}

void p7501_diag_cmd_scan(void) {
    p7501_scan_stats_t scan_stats;
    const float vdd = p7501_hw_read_vdd();
    p7501_diag_run_monitor_sweep(&scan_stats);
    print_scan_summary(&scan_stats, vdd);
    printf("SCAN: COMPLETED (Hits: %" PRIu32 ", Probes: %" PRIu32 ")\n",
           scan_stats.hits_detected, scan_stats.total_probes);
}

void p7501_diag_cmd_handshake(void) {
    printf("[HANDSHAKE] Executing Multi-Profile P7501 SW_UART+XRESET check...\n");
    uint32_t dcsr = P7501_BUS_RESP_OPEN_HIGH;
    uint32_t matched_mode = 0U;
    const bool halted = p7501_proto_execute_boot_handshake(&dcsr, &matched_mode);
    p7501_hw_isolate_bus_hiz();

    printf("DCSR: 0x%08" PRIX32 "\n", dcsr);
    if (dcsr == P7501_BUS_RESP_OPEN_HIGH) {
        printf("[HANDSHAKE] IDLE (0xFFFFFFFF): P7501 is Async UART 115200 8N1 (no synchronous OCD clock pin).\n");
    } else if (dcsr == P7501_BUS_RESP_OPEN_LOW) {
        printf("[HANDSHAKE] FAULT: Line held LOW (0x00000000). Check P7501 wiring continuity.\n");
    } else if (halted) {
        const p7501_flash_geometry_t geo = p7501_probe_flash_geometry();
        printf("[AUTODETECT] Passive Address Aliasing Probe: %" PRIu32 " MB NOR Flash detected (EEMUL NVRAM @ 0x%08" PRIX32 ").\n",
               geo.mb, geo.nvram_base);
        printf("[HANDSHAKE] SUCCESS! AM34 monitor active (Profile %" PRIu32 ").\n", matched_mode);
    } else {
        printf("[HANDSHAKE] ACTIVE: Target responded (DCSR=0x%08" PRIX32 ") while running.\n", dcsr);
    }
}

void p7501_diag_cmd_dump(const char *arg) {
    uint32_t addr = SPANSION_FLASH_UNCACHED_BASE;
    uint32_t count = 16U;

    if ((arg != NULL) && (*arg != '\0')) {
        char *next = NULL;
        addr = (uint32_t)strtoul(arg, &next, 16);
        if ((next != NULL) && (*next != '\0')) {
            char *cnt_str = p7501_cli_trim_whitespace(next);
            if ((cnt_str != NULL) && (*cnt_str != '\0')) {
                count = (uint32_t)strtoul(cnt_str, NULL, 0);
            }
        }
    }
    addr &= ~0x03UL;

    if (count == 0U) {
        count = 1U;
    }
    if (count > 32768U) {
        count = 32768U;
    }

    printf("[DUMP] Reading %" PRIu32 " words (%" PRIu32 " bytes) from 0x%08" PRIX32 "...\n",
           count, count * 4U, addr);

    for (uint32_t i = 0U; i < count; ++i) {
        const uint32_t curr_addr = addr + (i * 4U);
        const uint32_t val = p7501_hw_read_mem32(curr_addr);
        if ((i % 4U) == 0U) {
            printf("\n%08" PRIX32 ": ", curr_addr);
        }
        printf("%08" PRIX32 " ", val);
    }
    printf("\n\nDUMP: OK\n");
}

void p7501_diag_cmd_boot_profile(const char *arg) {
    uint32_t mode = 0U;
    if ((arg != NULL) && (*arg != '\0')) {
        mode = (uint32_t)atoi(arg);
        if (mode >= (uint32_t)P7501_BOOT_MODE_COUNT) {
            printf("[BREAK_IN] Invalid profile %" PRIu32 "! Defaulting to mode 0.\n", mode);
            mode = 0U;
        }
    }
    printf("[BREAK_IN] Executing P7501 SW_UART+XRESET profile %" PRIu32 " (%s)...\n",
           mode, boot_mode_label(mode));

    p7501_hw_boot_sequence((p7501_boot_mode_t)mode);
    printf("BREAK_IN: OK (Mode %" PRIu32 ")\n", mode);
}
