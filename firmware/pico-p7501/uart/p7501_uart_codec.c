/**
 * @file p7501_uart_codec.c
 * @brief Pure ISO C11 UART baudrate math, PIO CLKDIV computation, and rolling trigger matching.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_uart_codec.h"

#include <limits.h>
#include <stddef.h>
#include <string.h>

bool p7501_baud_is_valid(uint32_t baud) {
    return baud >= P7501_UART_MIN_BAUD && baud <= P7501_UART_MAX_BAUD;
}

uint32_t p7501_baud_to_bit_ns(uint32_t baud) {
    if (!p7501_baud_is_valid(baud)) {
        baud = P7501_UART_DEFAULT_BAUD;
    }
    return 1000000000u / baud;
}

void p7501_calc_pio_clkdiv_fp16(
    uint32_t sys_clk_hz,
    uint32_t baud,
    uint16_t *out_div_int,
    uint8_t *out_div_frac
) {
    if (!p7501_baud_is_valid(baud)) {
        baud = P7501_UART_DEFAULT_BAUD;
    }
    const uint64_t denom = 8ULL * (uint64_t)baud;
    uint64_t scaled = (((uint64_t)sys_clk_hz) * 256ULL + (denom / 2ULL)) / denom;
    if (scaled < 256ULL) {
        scaled = 256ULL;
    }
    if (scaled > 0xFFFF00ULL) {
        scaled = 0xFFFF00ULL;
    }
    if (out_div_int) {
        *out_div_int = (uint16_t)(scaled >> 8);
    }
    if (out_div_frac) {
        *out_div_frac = (uint8_t)(scaled & 0xFFu);
    }
}

uint32_t p7501_snap_standard_baud(uint32_t min_low_us, uint32_t *out_raw_bps) {
    static const uint32_t k_rates[] = {
        460800u, 230400u, 115200u, 57600u, 38400u, 31250u, 19200u, 9600u, 4800u, 2400u, 1200u
    };
    uint32_t raw_bps = 0u;
    uint32_t best_rate = 0u;
    uint32_t best_diff = UINT32_MAX;

    if (min_low_us == 0u || min_low_us > 2000u) {
        if (out_raw_bps) {
            *out_raw_bps = 0u;
        }
        return 0u;
    }

    raw_bps = 1000000u / min_low_us;
    if (out_raw_bps) {
        *out_raw_bps = raw_bps;
    }

    for (size_t i = 0u; i < (sizeof(k_rates) / sizeof(k_rates[0])); ++i) {
        const uint32_t cand = k_rates[i];
        const uint32_t diff = (raw_bps > cand) ? (raw_bps - cand) : (cand - raw_bps);
        if ((diff * 100u) <= (cand * 16u) && diff < best_diff) {
            best_diff = diff;
            best_rate = cand;
        }
    }
    return best_rate;
}

bool p7501_rolling_match_push(
    uint8_t *ring,
    size_t *ring_len,
    size_t cap,
    uint8_t byte,
    const char *needle
) {
    const size_t nlen = needle ? strlen(needle) : 0u;

    if (!ring || !ring_len || cap == 0u || nlen == 0u || nlen > cap) {
        return false;
    }
    if (*ring_len >= cap) {
        memmove(ring, ring + 1u, cap - 1u);
        *ring_len = cap - 1u;
    }
    ring[(*ring_len)++] = byte;

    if (*ring_len < nlen) {
        return false;
    }
    return memcmp(ring + (*ring_len - nlen), needle, nlen) == 0;
}
