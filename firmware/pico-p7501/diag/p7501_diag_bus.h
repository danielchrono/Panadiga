/**
 * @file p7501_diag_bus.h
 * @brief AM34 P7501 boot profile trigger, monitor probe sweep, and memory dump commands.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_DIAG_BUS_H
#define P7501_DIAG_BUS_H

#include "p7501_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/** Statistics gathered during a P7501 monitor opcode probe sweep. */
typedef struct p7501_scan_stats {
    uint32_t total_probes;
    uint32_t hits_detected;
    uint32_t open_bus_high;
    uint32_t open_bus_low;
} p7501_scan_stats_t;

/** Sweep P7501 boot profiles and monitor opcodes safely over UART. */
void p7501_diag_run_monitor_sweep(p7501_scan_stats_t *stats);

/** Handle `SCAN` CLI command and print summary report. */
void p7501_diag_cmd_scan(void);

/** Handle `HANDSHAKE` CLI command (discrete SW_UART+XRESET profile + monitor DCSR check). */
void p7501_diag_cmd_handshake(void);

/** Handle `DUMP [addr] [cnt]` CLI command via monitor aperture reads. */
void p7501_diag_cmd_dump(const char *arg);

/** Handle `BREAK_IN [0..3]` P7501 boot/reset profile trigger CLI command. */
void p7501_diag_cmd_boot_profile(const char *arg);

#ifdef __cplusplus
}
#endif

#endif /* P7501_DIAG_BUS_H */
