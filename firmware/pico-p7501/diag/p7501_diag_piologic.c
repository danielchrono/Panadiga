/**
 * @file p7501_diag_piologic.c
 * @brief Simultaneous 4-pin PIO0 logic sampler (`PIOLOGIC`: GP2/GP3/GP4/GP5).
 *
 * Strictly leaves GP3 (NSW5V) as Hi-Z input without pulls and never invokes
 * `pio_gpio_init` on any pin, eliminating electrical contention risk.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "diag/p7501_piologic_codec.h"
#include "hal/p7501_hw.h"
#include "p7501_logic_sampler.pio.h"

#include "hardware/gpio.h"
#include "hardware/pio.h"
#include "pico/stdlib.h"

#include <inttypes.h>
#include <stdio.h>
#include <string.h>

#define PIOLOGIC_MAX_EDGES 1024u

static uint8_t piologic_read_pins4_now(void) {
    const uint32_t all = gpio_get_all();
    return (uint8_t)((all >> P7501_PIN_RXD) & 0x0Fu);
}

static uint32_t piologic_fmt_min_us(uint32_t val) {
    return (val == UINT32_MAX) ? 0u : val;
}

void p7501_cmd_piologic(const char *arg) {
    static p7501_logic_edge_t s_edges[PIOLOGIC_MAX_EDGES];
    p7501_window_boot_spec_t boot;
    uint32_t sample_khz = 1000u;

    p7501_logic_parse_args(arg, &boot, &sample_khz);

    /* Hardware safety invariant: GP3 (NSW5V) MUST stay Hi-Z Input without pulls. */
    gpio_set_dir(P7501_PIN_NSW5V, GPIO_IN);
    gpio_disable_pulls(P7501_PIN_NSW5V);

    if (!pio_can_add_program(pio0, &p7501_logic_sampler_program)) {
        printf("PIOLOGIC: ERROR (No PIO0 instruction slots left)\n");
        return;
    }
    const int sm_signed = pio_claim_unused_sm(pio0, false);
    if (sm_signed < 0) {
        printf("PIOLOGIC: ERROR (No free PIO0 state machine)\n");
        return;
    }
    const uint sm = (uint)sm_signed;
    const uint offset = pio_add_program(pio0, &p7501_logic_sampler_program);
    p7501_logic_sampler_program_init(
        pio0, sm, offset, P7501_PIN_RXD, sample_khz * 1000u
    );

    bool hold_jig_active = false;
    bool release_xreset_pending = false;
    if (boot.boot_action != P7501_BOOT_ACTION_PASSIVE) {
        if (boot.boot_action == P7501_BOOT_ACTION_JIG) {
            p7501_hw_sw_uart_set_low(true);
            hold_jig_active = (boot.jig_hold_ms > 0u);
        } else {
            p7501_hw_sw_uart_set_low(false);
        }
        p7501_hw_reset_assert();
        sleep_ms(30);
        release_xreset_pending = true;
    }

    const uint8_t initial_state4 = piologic_read_pins4_now();
    uint8_t last_state4 = initial_state4;
    uint32_t rep_word = (uint32_t)last_state4 * 0x11111111u;
    size_t edge_count = 0u;
    bool overflow = false;

    const uint64_t total_samples = ((uint64_t)boot.window_ms * (uint64_t)sample_khz);
    uint64_t sample_idx = 0u;
    const uint64_t reset_release_sample = (uint64_t)sample_khz; /* Release XRESET at +1 ms inside capture */
    const uint64_t hold_samples = (uint64_t)boot.jig_hold_ms * (uint64_t)sample_khz;
    const uint64_t hard_deadline_us = time_us_64() + ((uint64_t)boot.window_ms * 1000ULL) + 50000ULL;

    pio_sm_clear_fifos(pio0, sm);
    pio0->fdebug = (1u << (PIO_FDEBUG_RXSTALL_LSB + sm));
    pio_sm_set_enabled(pio0, sm, true);

    while (sample_idx < total_samples && time_us_64() < hard_deadline_us) {
        if (release_xreset_pending && sample_idx >= reset_release_sample) {
            p7501_hw_reset_release();
            release_xreset_pending = false;
        }
        if (hold_jig_active && sample_idx >= hold_samples) {
            p7501_hw_sw_uart_set_low(false);
            hold_jig_active = false;
        }
        if (pio_sm_is_rx_fifo_empty(pio0, sm)) {
            tight_loop_contents();
            continue;
        }
        const uint32_t w = pio_sm_get(pio0, sm);
        if (w != rep_word) {
            for (uint32_t nib = 0u; nib < 8u; ++nib) {
                const uint8_t cur = (uint8_t)((w >> (nib * 4u)) & 0x0Fu);
                if (cur != last_state4) {
                    if (edge_count < PIOLOGIC_MAX_EDGES) {
                        const uint64_t abs_sample = sample_idx + nib;
                        s_edges[edge_count].t_us = (uint32_t)((abs_sample * 1000ULL) / sample_khz);
                        s_edges[edge_count].state4 = cur;
                        ++edge_count;
                    } else {
                        overflow = true;
                    }
                    last_state4 = cur;
                }
            }
            rep_word = (uint32_t)last_state4 * 0x11111111u;
        }
        sample_idx += 8u;
    }

    const bool rx_stall = (pio0->fdebug & (1u << (PIO_FDEBUG_RXSTALL_LSB + sm))) != 0u;
    pio_sm_set_enabled(pio0, sm, false);
    pio_remove_program(pio0, &p7501_logic_sampler_program, offset);
    pio_sm_unclaim(pio0, sm);

    if (boot.boot_action == P7501_BOOT_ACTION_JIG) {
        p7501_hw_sw_uart_set_low(false);
    }

    p7501_logic_stats_t stats;
    p7501_logic_analyze_edges(s_edges, edge_count, initial_state4, &stats);

    printf("=== PIOLOGIC PIO0 (%" PRIu32 "ms @ %" PRIu32 "kHz, boot=%d, hold=%" PRIu32 "ms) ===\n",
           boot.window_ms, sample_khz, (int)boot.boot_action, boot.jig_hold_ms);
    printf("Initial: RXD=%u NSW5V=%u TXD=%u XRESET=%u | fifo_stall=%s | edges=%u%s\n",
           (initial_state4 >> 0) & 1u, (initial_state4 >> 1) & 1u,
           (initial_state4 >> 2) & 1u, (initial_state4 >> 3) & 1u,
           rx_stall ? "YES" : "NO",
           (unsigned)edge_count, overflow ? " (TRUNCATED)" : "");

    static const char *const k_names[P7501_LOGIC_CH_COUNT] = {
        "RXD   (GP2)", "NSW5V (GP3)", "TXD   (GP4)", "XRESET(GP5)"
    };
    for (size_t ch = 0u; ch < P7501_LOGIC_CH_COUNT; ++ch) {
        printf("  %s : Final=%u, Transitions=%" PRIu32 " (rise=%" PRIu32 " fall=%" PRIu32
               " minLow=%" PRIu32 "us minHigh=%" PRIu32 "us)\n",
               k_names[ch],
               (last_state4 >> ch) & 1u,
               stats.edges[ch],
               stats.rising[ch],
               stats.falling[ch],
               piologic_fmt_min_us(stats.min_low_us[ch]),
               piologic_fmt_min_us(stats.min_high_us[ch]));
    }
    if (stats.saw_reset_release) {
        printf("[PIOLOGIC] XRESET_rise -> first_RXD_fall: %" PRIu32 " us\n",
               stats.reset_to_first_rxd_us);
    }
    if (stats.saw_tx_to_rx) {
        printf("[PIOLOGIC] TXD_fall -> RXD_fall turnaround: min=%" PRIu32 " us max=%" PRIu32 " us\n",
               stats.min_tx_to_rx_us, stats.max_tx_to_rx_us);
    }

    const size_t preview = (edge_count > 32u) ? 32u : edge_count;
    for (size_t i = 0u; i < preview; ++i) {
        const uint8_t s = s_edges[i].state4;
        const uint32_t dt = (i == 0u) ? s_edges[i].t_us : (s_edges[i].t_us - s_edges[i - 1u].t_us);
        printf("  +%07" PRIu32 " us (+%5" PRIu32 "us): RXD=%u NSW5V=%u TXD=%u XRESET=%u\n",
               s_edges[i].t_us, dt, (s >> 0) & 1u, (s >> 1) & 1u, (s >> 2) & 1u, (s >> 3) & 1u);
    }
    printf("PIOLOGIC: OK\n");
}
