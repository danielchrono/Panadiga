/**
 * @file p7501_uart_codec.h
 * @brief Portable ISO C11 UART argument, hex, baudrate, and pattern-trigger helpers.
 *
 * Compiled identically on ARM/RISC-V Pico targets and x86_64 CI hosts (`PICO_HOST_TEST`).
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_UART_CODEC_H
#define P7501_UART_CODEC_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define P7501_UART_MIN_BAUD 1200u
#define P7501_UART_DEFAULT_BAUD 115200u
#define P7501_UART_MAX_BAUD 460800u
#define P7501_UART_HEX_MAX_BYTES 128u
#define P7501_WAITTX_MAX_TRIGGER 48u

typedef enum p7501_boot_action {
    P7501_BOOT_ACTION_PASSIVE = 0,
    P7501_BOOT_ACTION_NORMAL = 1,
    P7501_BOOT_ACTION_JIG = 2
} p7501_boot_action_t;

typedef struct p7501_waittx_spec {
    uint32_t timeout_ms;
    p7501_boot_action_t boot_action;
    uint32_t jig_hold_ms;
    char trigger[P7501_WAITTX_MAX_TRIGGER + 1u];
    uint8_t tx_bytes[P7501_UART_HEX_MAX_BYTES];
    size_t tx_len;
    uint32_t post_rx_ms;
    bool tx_break;
} p7501_waittx_spec_t;

uint32_t p7501_parse_uint32_clamped(
    const char *arg,
    uint32_t default_val,
    uint32_t min_val,
    uint32_t max_val
);

const char *p7501_next_token(const char *cursor, char *out, size_t out_cap);

bool p7501_decode_hex_bytes(
    const char *hex,
    uint8_t *out,
    size_t max_out,
    size_t *out_len
);

bool p7501_baud_is_valid(uint32_t baud);
uint32_t p7501_baud_to_bit_ns(uint32_t baud);

void p7501_calc_pio_clkdiv_fp16(
    uint32_t sys_clk_hz,
    uint32_t baud,
    uint16_t *out_div_int,
    uint8_t *out_div_frac
);

uint32_t p7501_snap_standard_baud(uint32_t min_low_us, uint32_t *out_raw_bps);

p7501_boot_action_t p7501_parse_boot_action(const char *token);

bool p7501_rolling_match_push(
    uint8_t *ring,
    size_t *ring_len,
    size_t cap,
    uint8_t byte,
    const char *needle
);

bool p7501_parse_waittx_args(const char *arg, p7501_waittx_spec_t *out);

#include "common/p7501_hex.h"

/** Parsed capture window + boot stimulation specification (`UARTSTREAM`/`HEXCAP`/`PWRTRACE`). */
typedef struct p7501_window_boot_spec {
    uint32_t window_ms;
    p7501_boot_action_t boot_action;
    uint32_t jig_hold_ms;
} p7501_window_boot_spec_t;

/** Parse boot action (`NORMAL`, `JIG`, `JIG:<hold_ms>`, `NOPULL`) with optional timed SW_UART release. */
p7501_boot_action_t p7501_parse_boot_spec(const char *token, uint32_t *out_jig_hold_ms);

/** Parse standard `<ms> [NORMAL|JIG|JIG:<ms>|NOPULL]` arguments (DRY helper across stream/diag commands). */
void p7501_parse_window_boot_args(
    const char *arg,
    uint32_t default_ms,
    uint32_t min_ms,
    uint32_t max_ms,
    p7501_window_boot_spec_t *out_spec
);

#ifdef __cplusplus
}
#endif

#endif /* P7501_UART_CODEC_H */
