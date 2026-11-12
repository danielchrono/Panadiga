/**
 * @file dwire32a_diag_ocd.h
 * @brief Matsushita AM34 OCD opcode sweep, Cold Break-In handshake, and memory dump routines.
 *
 * Groups `SCAN`, `HANDSHAKE`, `DUMP`, and `BREAK_IN` commands separately from
 * electrical pin/bus diagnostics.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef DWIRE32A_DIAG_OCD_H
#define DWIRE32A_DIAG_OCD_H

#include "dwire32a_types.h"

#ifdef __cplusplus
extern "C" {
#endif

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
 * @brief Execute systematic sweep across all Cold Break-In modes and 256 OCD opcodes.
 */
void dwire_diag_run_opcode_sweep(dwire_scan_stats_t *stats);

/**
 * @brief Handle `SCAN` CLI command and print summary report.
 */
void dwire_diag_cmd_scan(void);

/**
 * @brief Handle `HANDSHAKE` CLI command (multi-mode Cold Break-In + Flash autodetect).
 */
void dwire_diag_cmd_handshake(void);

/**
 * @brief Handle `DUMP [addr] [cnt]` CLI command via DWire32A OCD memory reads.
 */
void dwire_diag_cmd_dump(const char *arg);

/**
 * @brief Handle `BREAK_IN [0..3]` explicit Cold Break-In trigger CLI command.
 */
void dwire_diag_cmd_break_in(const char *arg);

#ifdef __cplusplus
}
#endif

#endif /* DWIRE32A_DIAG_OCD_H */
