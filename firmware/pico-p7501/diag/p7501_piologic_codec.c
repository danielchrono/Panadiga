/**
 * @file p7501_piologic_codec.c
 * @brief Pure ISO C11 argument parser and edge/turnaround analyzer for `PIOLOGIC`.
 *
 * Compiled identically on Pico ARM/RISC-V and host C11 test targets (`PICO_HOST_TEST`).
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "diag/p7501_piologic_codec.h"

#include <string.h>

void p7501_logic_parse_args(
    const char *arg,
    p7501_window_boot_spec_t *out_boot,
    uint32_t *out_sample_khz
) {
    if (out_boot != NULL) {
        p7501_parse_window_boot_args(arg, 4000u, 100u, 15000u, out_boot);
    }
    if (out_sample_khz == NULL) {
        return;
    }
    *out_sample_khz = 1000u;
    if (arg == NULL) {
        return;
    }

    char tok[24];
    const char *cur = p7501_next_token(arg, tok, sizeof(tok));
    if (cur != NULL) {
        cur = p7501_next_token(cur, tok, sizeof(tok));
    }
    if (cur != NULL && p7501_next_token(cur, tok, sizeof(tok)) != NULL) {
        *out_sample_khz = p7501_parse_uint32_clamped(tok, 1000u, 100u, 2000u);
    }
}

void p7501_logic_analyze_edges(
    const p7501_logic_edge_t *edges,
    size_t count,
    uint8_t initial_state4,
    p7501_logic_stats_t *out
) {
    if (out == NULL) {
        return;
    }
    memset(out, 0, sizeof(*out));
    for (size_t ch = 0u; ch < P7501_LOGIC_CH_COUNT; ++ch) {
        out->min_low_us[ch] = UINT32_MAX;
        out->min_high_us[ch] = UINT32_MAX;
    }
    if (edges == NULL || count == 0u) {
        return;
    }

    uint8_t prev = (uint8_t)(initial_state4 & 0x0Fu);
    uint32_t ch_edge_t[P7501_LOGIC_CH_COUNT] = { 0u, 0u, 0u, 0u };
    bool ch_has_edge[P7501_LOGIC_CH_COUNT] = { false, false, false, false };
    bool tx_falling_pending = false;
    uint32_t tx_falling_us = 0u;
    uint32_t reset_release_us = 0u;

    for (size_t i = 0u; i < count; ++i) {
        const uint8_t cur = (uint8_t)(edges[i].state4 & 0x0Fu);
        const uint32_t t_us = edges[i].t_us;
        const uint8_t diff = (uint8_t)(prev ^ cur);

        for (size_t ch = 0u; ch < P7501_LOGIC_CH_COUNT; ++ch) {
            const uint8_t mask = (uint8_t)(1u << ch);
            if ((diff & mask) == 0u) {
                continue;
            }
            const bool now_high = (cur & mask) != 0u;
            ++out->edges[ch];
            if (now_high) {
                ++out->rising[ch];
            } else {
                ++out->falling[ch];
            }

            if (ch_has_edge[ch]) {
                const uint32_t pulse = t_us - ch_edge_t[ch];
                if (now_high && pulse < out->min_low_us[ch]) {
                    out->min_low_us[ch] = pulse;
                } else if (!now_high && pulse < out->min_high_us[ch]) {
                    out->min_high_us[ch] = pulse;
                }
            }
            ch_edge_t[ch] = t_us;
            ch_has_edge[ch] = true;
        }

        if ((diff & (1u << P7501_LOGIC_CH_XRESET)) != 0u &&
            (cur & (1u << P7501_LOGIC_CH_XRESET)) != 0u &&
            !out->saw_reset_release) {
            out->saw_reset_release = true;
            reset_release_us = t_us;
        }

        if ((diff & (1u << P7501_LOGIC_CH_TXD)) != 0u &&
            (cur & (1u << P7501_LOGIC_CH_TXD)) == 0u) {
            tx_falling_pending = true;
            tx_falling_us = t_us;
        }

        if ((diff & (1u << P7501_LOGIC_CH_RXD)) != 0u &&
            (cur & (1u << P7501_LOGIC_CH_RXD)) == 0u) {
            if (out->saw_reset_release && out->reset_to_first_rxd_us == 0u &&
                t_us >= reset_release_us) {
                out->reset_to_first_rxd_us = t_us - reset_release_us;
            }
            if (tx_falling_pending && t_us >= tx_falling_us) {
                const uint32_t lat = t_us - tx_falling_us;
                if (!out->saw_tx_to_rx) {
                    out->saw_tx_to_rx = true;
                    out->min_tx_to_rx_us = lat;
                    out->max_tx_to_rx_us = lat;
                } else {
                    if (lat < out->min_tx_to_rx_us) out->min_tx_to_rx_us = lat;
                    if (lat > out->max_tx_to_rx_us) out->max_tx_to_rx_us = lat;
                }
                tx_falling_pending = false;
            }
        }
        prev = cur;
    }
}
