/**
 * @file p7501_uart_baud.c
 * @brief PIO1 multi-baud configuration (`BAUD`) and hardware GP2 autobaud detector (`BAUDSCAN`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_uart_baud.h"

#include "common/p7501_hex.h"
#include "hal/p7501_led.h"
#include "p7501_types.h"
#include "uart/p7501_uart_codec.h"
#include "uart/p7501_uart_pio_engine.h"

#include "hardware/gpio.h"
#include "pico/stdlib.h"
#include "pico/time.h"

#include <inttypes.h>
#include <stdio.h>

void p7501_cmd_baud(const char *arg) {
    char tok[24] = {0};

    if (p7501_next_token(arg, tok, sizeof(tok)) == NULL || tok[0] == '\0') {
        printf("[BAUD] Active PIO1 baud: %" PRIu32 " bps (8N1)\n", p7501_pio_get_baud());
        printf("BAUD: OK\n");
        return;
    }

    const uint32_t req = p7501_parse_uint32_clamped(
        tok, 0u, P7501_UART_MIN_BAUD, P7501_UART_MAX_BAUD
    );
    const uint32_t raw_exact = p7501_parse_uint32_clamped(tok, 0u, 1u, 10000000u);
    if (!p7501_baud_is_valid(raw_exact) || !p7501_pio_set_baud(req)) {
        printf("[BAUD] ERROR: Invalid baudrate '%s' (valid range: %" PRIu32 "..%" PRIu32 ").\n",
               tok, (uint32_t)P7501_UART_MIN_BAUD, (uint32_t)P7501_UART_MAX_BAUD);
        return;
    }

    printf("[BAUD] PIO1 RX/TX configured for %" PRIu32 " bps (bit_ns=%" PRIu32 ").\n",
           p7501_pio_get_baud(), p7501_baud_to_bit_ns(p7501_pio_get_baud()));
    printf("BAUD: OK\n");
}

void p7501_cmd_baudscan(const char *arg) {
    p7501_window_boot_spec_t spec;
    char tok3[24] = {0};
    bool auto_apply = false;

    p7501_parse_window_boot_args(arg, 4000u, 200u, 30000u, &spec);
    {
        char t1[24], t2[24];
        const char *cur = p7501_next_token(arg, t1, sizeof(t1));
        if (cur != NULL) {
            cur = p7501_next_token(cur, t2, sizeof(t2));
            if (cur != NULL && p7501_next_token(cur, tok3, sizeof(tok3)) != NULL) {
                auto_apply = p7501_ascii_ieq(tok3, "APPLY");
            }
        }
    }

    printf("[BAUDSCAN] Measuring GP2 (P7501 RXD) pulse widths over %" PRIu32 " ms...\n",
           spec.window_ms);
    fflush(stdout);

    p7501_led_set_auto_mode(LED_MODE_BUS_ACTIVE);
    p7501_pio_begin_rx_session_ex(spec.boot_action, spec.jig_hold_ms);

    const absolute_time_t end_t = make_timeout_time_ms(spec.window_ms);
    uint32_t min_low_us = UINT32_MAX;
    uint32_t max_low_us = 0u;
    uint32_t edges = 0u;
    bool prev_level = gpio_get(P7501_PIN_RXD);
    uint32_t last_fall_us = time_us_32();

    while (!time_reached(end_t)) {
        (void)p7501_pio_poll_sw_uart_release();
        const bool cur = gpio_get(P7501_PIN_RXD);
        if (cur != prev_level) {
            const uint32_t now_us = time_us_32();
            ++edges;
            if (!cur) {
                last_fall_us = now_us;
            } else {
                const uint32_t low_us = now_us - last_fall_us;
                if (low_us >= 2u && low_us <= 2000u) {
                    if (low_us < min_low_us) {
                        min_low_us = low_us;
                    }
                    if (low_us > max_low_us) {
                        max_low_us = low_us;
                    }
                }
            }
            prev_level = cur;
        }
    }

    p7501_pio_end_session();
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);

    if (edges == 0u || min_low_us == UINT32_MAX) {
        printf("[BAUDSCAN] No valid UART low pulses observed on GP2 (edges=%" PRIu32 ").\n", edges);
        printf("BAUDSCAN: OK (no_signal)\n");
        return;
    }

    uint32_t raw_bps = 0u;
    const uint32_t snapped = p7501_snap_standard_baud(min_low_us, &raw_bps);
    printf("[BAUDSCAN] edges=%" PRIu32 " min_low_us=%" PRIu32 " max_low_us=%" PRIu32
           " raw_bps=%" PRIu32 " snapped_baud=%" PRIu32 "\n",
           edges, min_low_us, max_low_us, raw_bps, snapped);

    if (auto_apply && snapped > 0u && p7501_pio_set_baud(snapped)) {
        printf("[BAUDSCAN] Auto-applied PIO1 baudrate -> %" PRIu32 " bps.\n", snapped);
    }
    printf("BAUDSCAN: OK\n");
}
