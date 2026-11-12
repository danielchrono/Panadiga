/**
 * @file dwire32a_diag_electrical.h
 * @brief Electrical bench diagnostics and passive P7501 signal monitoring.
 *
 * Groups `SNIFF`, `PINCHECK`, and `BUSDIAG` routines separately from OCD
 * opcode scanning, handshake probing, and Flash/RAM memory dumps.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef DWIRE32A_DIAG_ELECTRICAL_H
#define DWIRE32A_DIAG_ELECTRICAL_H

#include "dwire32a_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Transition counts accumulated across the 4 digital P7501 lines.
 */
typedef struct dwire_bus_transitions {
    uint32_t extrg;
    uint32_t sclk;
    uint32_t sdata;
    uint32_t nreset;
} dwire_bus_transitions_t;

/**
 * @brief Snapshot of logical pin states for the 4 digital P7501 lines.
 */
typedef struct dwire_bus_levels {
    bool extrg;
    bool sclk;
    bool sdata;
    bool nreset;
} dwire_bus_levels_t;

/**
 * @brief Sample edge transitions across EXTRG, SCLK, SDATA, and nRESET for `duration_ms`.
 */
void dwire_diag_sample_transitions(
    uint32_t duration_ms,
    dwire_bus_transitions_t *out_counts,
    dwire_bus_levels_t *out_final_levels
);

/**
 * @brief Handle `SNIFF [ms]` passive bus monitor CLI command.
 */
void dwire_diag_cmd_sniff(const char *arg);

/**
 * @brief Handle `PINCHECK` 6-pin electrical pull-up/pull-down & reset rise-time test.
 */
void dwire_diag_cmd_pincheck(void);

/**
 * @brief Handle `BUSDIAG` P7501 drive-strength contention & boot transition profiler.
 */
void dwire_diag_cmd_busdiag(void);

#ifdef __cplusplus
}
#endif

#endif /* DWIRE32A_DIAG_ELECTRICAL_H */
