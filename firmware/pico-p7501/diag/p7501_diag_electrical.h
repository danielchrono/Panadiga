/**
 * @file p7501_diag_electrical.h
 * @brief Electrical bench diagnostics and passive P7501 6-pin monitoring.
 *
 * Groups `SNIFF`, `PINCHECK`, and `BUSDIAG` routines separately from
 * boot/monitor memory commands.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_DIAG_ELECTRICAL_H
#define P7501_DIAG_ELECTRICAL_H

#include "p7501_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/** Transition counts accumulated across the 4 digital P7501 lines. */
typedef struct p7501_bus_transitions {
    uint32_t rxd;
    uint32_t nsw5v;
    uint32_t txd;
    uint32_t xreset;
} p7501_bus_transitions_t;

/** Snapshot of logical pin states for the 4 digital P7501 lines. */
typedef struct p7501_bus_levels {
    bool rxd;
    bool nsw5v;
    bool txd;
    bool xreset;
} p7501_bus_levels_t;

/** Sample edge transitions across GP2 (RXD), GP3 (NSW5V), GP4 (TXD), and GP5 (XRESET). */
void p7501_diag_sample_transitions(
    uint32_t duration_ms,
    p7501_bus_transitions_t *out_counts,
    p7501_bus_levels_t *out_final_levels
);

/** Handle `SNIFF [ms]` passive P7501 bus monitor CLI command. */
void p7501_diag_cmd_sniff(const char *arg);

/** Handle `PINCHECK` P7501 6-pin electrical matrix & XRESET rise-time test. */
void p7501_diag_cmd_pincheck(void);

/** Handle `BUSDIAG` P7501 TX drive-strength contention & boot transition profiler. */
void p7501_diag_cmd_busdiag(void);

#ifdef __cplusplus
}
#endif

#endif /* P7501_DIAG_ELECTRICAL_H */
