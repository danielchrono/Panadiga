/**
 * @file p7501_diag_pwrtrace.c
 * @brief Boot rail voltage (ADC0 SW_UART) & NSW5V/RXD/TXD glitch logger (`PWRTRACE`).
 *
 * Strictly preserves GP3 (`NSW5V`) as Hi-Z input (no pulls) at all times.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "diag/p7501_diag_pwrtrace.h"

#include "hal/p7501_hw.h"
#include "hal/p7501_led.h"
#include "p7501_types.h"
#include "uart/p7501_uart_codec.h"
#include "uart/p7501_uart_pio_engine.h"

#include "hardware/adc.h"
#include "hardware/gpio.h"
#include "pico/stdlib.h"
#include "pico/time.h"

#include <inttypes.h>
#include <stdio.h>

void p7501_cmd_pwrtrace(const char *arg) {
    p7501_window_boot_spec_t spec;
    p7501_parse_window_boot_args(arg, 4000u, 100u, 30000u, &spec);

    printf("=== PWRTRACE: window=%" PRIu32 "ms boot=%d hold=%" PRIu32 "ms ===\n",
           spec.window_ms, (int)spec.boot_action, spec.jig_hold_ms);
    fflush(stdout);

    p7501_led_set_auto_mode(LED_MODE_BUS_ACTIVE);
    p7501_pio_begin_rx_session_ex(spec.boot_action, spec.jig_hold_ms);

    if (spec.boot_action != P7501_BOOT_ACTION_JIG || spec.jig_hold_ms > 0u) {
        p7501_hw_restore_sw_uart_adc0();
    }

    const absolute_time_t end_t = make_timeout_time_ms(spec.window_ms);
    float min_v = 9.99f;
    float max_v = 0.00f;
    uint32_t nsw5v_edges = 0u;
    uint32_t rxd_edges = 0u;
    uint32_t txd_edges = 0u;
    bool prev_nsw = gpio_get(P7501_PIN_NSW5V);
    bool prev_rxd = gpio_get(P7501_PIN_RXD);
    bool prev_txd = gpio_get(P7501_PIN_TXD);
    absolute_time_t next_adc_t = make_timeout_time_ms(10u);

    while (!time_reached(end_t)) {
        if (p7501_pio_poll_sw_uart_release()) {
            p7501_hw_restore_sw_uart_adc0();
        }
        const bool cur_nsw = gpio_get(P7501_PIN_NSW5V);
        const bool cur_rxd = gpio_get(P7501_PIN_RXD);
        const bool cur_txd = gpio_get(P7501_PIN_TXD);

        if (cur_nsw != prev_nsw) {
            ++nsw5v_edges;
            prev_nsw = cur_nsw;
        }
        if (cur_rxd != prev_rxd) {
            ++rxd_edges;
            prev_rxd = cur_rxd;
        }
        if (cur_txd != prev_txd) {
            ++txd_edges;
            prev_txd = cur_txd;
        }

        if (time_reached(next_adc_t)) {
            next_adc_t = make_timeout_time_ms(10u);
            p7501_led_tick();
            if (!gpio_is_dir_out(P7501_PIN_SW_UART_ADC)) {
                adc_select_input(P7501_ADC_CH_VDD);
                const float v = ((float)adc_read()) * (P7501_ADC_VREF / P7501_ADC_RESOLUTION);
                if (v < min_v) {
                    min_v = v;
                }
                if (v > max_v) {
                    max_v = v;
                }
            }
        }
    }

    p7501_pio_end_session();
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);

    printf("[PWRTRACE] GP26_SWUART: min=%.2fV max=%.2fV | Edges: RXD=%" PRIu32
           " NSW5V=%" PRIu32 " TXD=%" PRIu32 " (final NSW5V=%d)\n",
           (double)min_v, (double)max_v, rxd_edges, nsw5v_edges, txd_edges,
           gpio_get(P7501_PIN_NSW5V) ? 1 : 0);
    printf("PWRTRACE: OK\n");
}
