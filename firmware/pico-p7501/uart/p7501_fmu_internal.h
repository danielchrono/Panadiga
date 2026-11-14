/**
 * @file p7501_fmu_internal.h
 * @brief Internal shared phase state and trigger helpers for FMU boot probes.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_FMU_INTERNAL_H
#define P7501_FMU_INTERNAL_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct p7501_fmu_phase_state {
    bool sw_uart_flipped;
    bool sent_after_serial_msg;
    bool wait1_seen;
    uint32_t wait1_us;
} p7501_fmu_phase_state_t;

bool p7501_fmu_buf_contains_tag_at_eol(
    const uint8_t *buf,
    uint32_t len,
    uint32_t start_min,
    const char *tag,
    size_t tag_len
);

void p7501_fmu_handle_phase_triggers(
    p7501_fmu_phase_state_t *st,
    const uint8_t *rx_buf,
    uint32_t fmode,
    uint32_t rx_len,
    uint32_t now_us
);

void p7501_fmu_start_phase_session(
    bool use_12ma,
    bool *out_prev_high,
    p7501_fmu_phase_state_t *out_st
);

#endif /* P7501_FMU_INTERNAL_H */
