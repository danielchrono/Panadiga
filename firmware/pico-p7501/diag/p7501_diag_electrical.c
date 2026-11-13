/**
 * @file p7501_diag_electrical.c
 * @brief Implementation of P7501 6-pin electrical matrix checks, contention tests, and bus sniffing.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "diag/p7501_diag_electrical.h"
#include "hal/p7501_hw.h"
#include "pico/stdlib.h"
#include "hardware/gpio.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>

static void configure_busdiag_hiz_and_sw_uart(bool sw_uart_low) {
    p7501_hw_isolate_bus_hiz();
    p7501_hw_sw_uart_set_low(sw_uart_low);
}

void p7501_diag_sample_transitions(
    uint32_t duration_ms,
    p7501_bus_transitions_t *out_counts,
    p7501_bus_levels_t *out_final_levels
) {
    p7501_bus_transitions_t counts = { 0U, 0U, 0U, 0U };
    uint32_t all = gpio_get_all();
    bool last_rxd    = ((all & (1UL << P7501_PIN_RXD)) != 0U);
    bool last_nsw5v  = ((all & (1UL << P7501_PIN_NSW5V)) != 0U);
    bool last_txd    = ((all & (1UL << P7501_PIN_TXD)) != 0U);
    bool last_xreset = ((all & (1UL << P7501_PIN_XRESET)) != 0U);

    const absolute_time_t end_time = make_timeout_time_ms(duration_ms);
    while (get_absolute_time() < end_time) {
        all = gpio_get_all();
        const bool cur_rxd    = ((all & (1UL << P7501_PIN_RXD)) != 0U);
        const bool cur_nsw5v  = ((all & (1UL << P7501_PIN_NSW5V)) != 0U);
        const bool cur_txd    = ((all & (1UL << P7501_PIN_TXD)) != 0U);
        const bool cur_xreset = ((all & (1UL << P7501_PIN_XRESET)) != 0U);

        if (cur_rxd    != last_rxd)    { counts.rxd++;    last_rxd    = cur_rxd;    }
        if (cur_nsw5v  != last_nsw5v)  { counts.nsw5v++;  last_nsw5v  = cur_nsw5v;  }
        if (cur_txd    != last_txd)    { counts.txd++;    last_txd    = cur_txd;    }
        if (cur_xreset != last_xreset) { counts.xreset++; last_xreset = cur_xreset; }
    }

    if (out_counts != NULL) {
        *out_counts = counts;
    }
    if (out_final_levels != NULL) {
        out_final_levels->rxd    = last_rxd;
        out_final_levels->nsw5v  = last_nsw5v;
        out_final_levels->txd    = last_txd;
        out_final_levels->xreset = last_xreset;
    }
}

void p7501_diag_cmd_sniff(const char *arg) {
    uint32_t duration_ms = 250U;
    if ((arg != NULL) && (*arg != '\0')) {
        duration_ms = (uint32_t)atoi(arg);
        if (duration_ms == 0U) { duration_ms = 250U; }
        if (duration_ms > 10000U) { duration_ms = 10000U; }
    }

    printf("[SNIFF] Passively monitoring P7501 GP2/RXD, GP3/NSW5V, GP4/TXD, GP5/XRESET for %" PRIu32 " ms...\n",
           duration_ms);
    p7501_hw_isolate_bus_hiz();
    sleep_us(200);

    p7501_bus_transitions_t counts;
    p7501_bus_levels_t final_lvl;
    p7501_diag_sample_transitions(duration_ms, &counts, &final_lvl);
    p7501_hw_isolate_bus_hiz();

    printf("SNIFF RESULT (%" PRIu32 " ms):\n", duration_ms);
    printf("  UARTP2M (GP2) : Final=%d, Transitions=%" PRIu32 "\n", final_lvl.rxd ? 1 : 0, counts.rxd);
    printf("  NSW5V   (GP3) : Final=%d, Transitions=%" PRIu32 "\n", final_lvl.nsw5v ? 1 : 0, counts.nsw5v);
    printf("  UARTM2P (GP4) : Final=%d, Transitions=%" PRIu32 "\n", final_lvl.txd ? 1 : 0, counts.txd);
    printf("  XRESET  (GP5) : Final=%d, Transitions=%" PRIu32 "\n", final_lvl.xreset ? 1 : 0, counts.xreset);
    if ((counts.rxd == 0U) && (counts.nsw5v == 0U) && (counts.txd == 0U) && (counts.xreset == 0U)) {
        if (final_lvl.nsw5v && final_lvl.txd) {
            printf("  [STATUS] Lines idle HIGH (3.3V). Target is powered and P7501 pull-ups are intact.\n");
        } else if (!final_lvl.nsw5v && !final_lvl.txd) {
            printf("  [STATUS] Lines steady LOW (0V). Target may be unpowered or lines shorted to GND.\n");
        } else {
            printf("  [STATUS] Lines static: GP2=%d, GP3=%d, GP4=%d, GP5=%d.\n",
                   final_lvl.rxd ? 1 : 0, final_lvl.nsw5v ? 1 : 0,
                   final_lvl.txd ? 1 : 0, final_lvl.xreset ? 1 : 0);
        }
    } else {
        printf("  [STATUS] ACTIVE P7501 BUS ACTIVITY DETECTED! (GP2=%" PRIu32 ", GP3=%" PRIu32 ", GP4=%" PRIu32 ", GP5=%" PRIu32 ")\n",
               counts.rxd, counts.nsw5v, counts.txd, counts.xreset);
    }
    printf("SNIFF: OK\n");
}

void p7501_diag_cmd_pincheck(void) {
    static const uint8_t pins[4] = {
        (uint8_t)P7501_PIN_RXD,
        (uint8_t)P7501_PIN_NSW5V,
        (uint8_t)P7501_PIN_TXD,
        (uint8_t)P7501_PIN_XRESET
    };
    static const char *names[4] = {
        "GP2 (Pin6 RXD   )",
        "GP3 (Pin5 NSW5V )",
        "GP4 (Pin3 TXD   )",
        "GP5 (Pin2 XRESET)"
    };

    p7501_hw_isolate_bus_hiz();
    sleep_us(500);

    printf("\n=== P7501 6-PIN ELECTRICAL MATRIX (PINCHECK) ===\n");
    printf("Pin               | Hi-Z | Pull-Down | Pull-Up | Electrical State\n");
    printf("------------------+------+-----------+---------+----------------------------------------\n");

    for (size_t i = 0U; i < 4U; ++i) {
        const uint8_t p = pins[i];
        /* Keep GP3 (NSW5V) strictly Hi-Z with pulls disabled to guard against ~5.15V upstream rail */
        gpio_disable_pulls(p);
        sleep_us(300);
        const int hiz = gpio_get(p) ? 1 : 0;

        int pd = hiz;
        int pu = hiz;
        const char *state = "NSW5V HI-Z SENSE (Pulls Disabled Safe Rail)";
        if (p != (uint8_t)P7501_PIN_NSW5V) {
            gpio_pull_down(p);
            sleep_us(300);
            pd = gpio_get(p) ? 1 : 0;

            gpio_pull_up(p);
            sleep_us(300);
            pu = gpio_get(p) ? 1 : 0;

            gpio_disable_pulls(p);
            state = "UNKNOWN";
            if ((pd == 1) && (pu == 1)) {
                state = "STRONG HIGH (External Pull-Up / 3.3V Active)";
            } else if ((pd == 0) && (pu == 0)) {
                state = "STRONG LOW  (Grounded / Supervisor Held 0V)";
            } else if ((pd == 0) && (pu == 1)) {
                state = "HI-Z / CMOS INPUT (No External Pull or Open)";
            }
        }
        printf("%-17s |  %d   |     %d     |    %d    | %s\n", names[i], hiz, pd, pu, state);
    }

    const float vdd_hiz = p7501_hw_sample_sw_uart_volts(false, false);
    const float vdd_pd  = p7501_hw_sample_sw_uart_volts(true, false);
    const float vdd_pu  = p7501_hw_sample_sw_uart_volts(false, true);

    printf("GP26(Pin1 SWUART) | %.2fV|   %.2fV   |  %.2fV  | P7501 SW_UART / Target Sense\n",
           vdd_hiz, vdd_pd, vdd_pu);

    p7501_hw_reset_assert();
    sleep_ms(15);
    const int rst_low_gp2 = gpio_get(P7501_PIN_RXD) ? 1 : 0;
    const int rst_low_gp3 = gpio_get(P7501_PIN_NSW5V) ? 1 : 0;
    const int rst_low_gp4 = gpio_get(P7501_PIN_TXD) ? 1 : 0;
    const int rst_low_gp5 = gpio_get(P7501_PIN_XRESET) ? 1 : 0;
    p7501_hw_reset_release();

    uint32_t rise_us = 0U;
    bool rose = false;
    for (rise_us = 0U; rise_us <= 50000U; rise_us += 10U) {
        if (gpio_get(P7501_PIN_XRESET)) {
            rose = true;
            break;
        }
        sleep_us(10);
    }

    p7501_bus_transitions_t post_counts;
    p7501_diag_sample_transitions(250U, &post_counts, NULL);

    printf("------------------------------------------------------------------------\n");
    printf("While GP5 (XRESET) asserted LOW (0V): GP2=%d, GP3=%d, GP4=%d, GP5=%d\n",
           rst_low_gp2, rst_low_gp3, rst_low_gp4, rst_low_gp5);
    if (rose) {
        printf("XRESET Release Rise Time (Hi-Z)     : %" PRIu32 " us\n", rise_us);
    } else {
        printf("XRESET Release Rise Time (Hi-Z)     : TIMEOUT (>50000 us, remained 0)\n");
    }
    printf("Post-Reset 250ms Transitions        : GP2=%" PRIu32 ", GP3=%" PRIu32 ", GP4=%" PRIu32 ", GP5=%" PRIu32 "\n",
           post_counts.rxd, post_counts.nsw5v, post_counts.txd, post_counts.xreset);
    printf("PINCHECK: OK\n");

    p7501_hw_isolate_bus_hiz();
}

void p7501_diag_cmd_busdiag(void) {
    printf("\n=== P7501 ELECTRICAL CONTENTION & DUAL-CHANNEL DIAGNOSTIC (BUSDIAG) ===\n");
    static const enum gpio_drive_strength ds_list[4] = {
        GPIO_DRIVE_STRENGTH_2MA,
        GPIO_DRIVE_STRENGTH_4MA,
        GPIO_DRIVE_STRENGTH_8MA,
        GPIO_DRIVE_STRENGTH_12MA
    };
    static const char *ds_names[4] = { "2mA", "4mA", "8mA", "12mA" };

    for (uint32_t st = 0U; st < 3U; ++st) {
        configure_busdiag_hiz_and_sw_uart(st != 0U);
        if (st == 2U) {
            p7501_hw_reset_assert();
        } else {
            p7501_hw_reset_release();
        }
        sleep_ms(5);

        printf("State %" PRIu32 " (%s):\n", st,
               (st == 0U) ? "SW_UART=HiZ, XRESET=HiZ" :
               (st == 1U) ? "SW_UART=LOW, XRESET=HiZ" : "SW_UART=LOW, XRESET=LOW");

        for (uint32_t d = 0U; d < 4U; ++d) {
            gpio_init(P7501_PIN_TXD);
            gpio_disable_pulls(P7501_PIN_TXD);
            gpio_set_drive_strength(P7501_PIN_TXD, ds_list[d]);
            gpio_put(P7501_PIN_TXD, false);
            gpio_set_dir(P7501_PIN_TXD, GPIO_OUT);
            sleep_us(25);
            const int pad_while_low = gpio_get(P7501_PIN_TXD) ? 1 : 0;

            uint32_t ticks = 0U;
            gpio_set_dir(P7501_PIN_TXD, GPIO_IN);
            while (!gpio_get(P7501_PIN_TXD) && (ticks < 50000U)) {
                ticks++;
            }
            printf("  GP4 (MAX RXD) @ %-4s -> pad_low=%d, release_rise_ticks=%" PRIu32 "\n",
                   ds_names[d], pad_while_low, ticks);
            sleep_us(100);
        }
    }

    for (uint32_t mode = 0U; mode < 2U; ++mode) {
        configure_busdiag_hiz_and_sw_uart(mode != 0U);
        sleep_ms(2);
        p7501_hw_reset_assert();
        sleep_ms(15);
        p7501_hw_reset_release();

        p7501_bus_transitions_t phase_a;
        p7501_bus_transitions_t phase_b;
        p7501_diag_sample_transitions(1300U, &phase_a, NULL);
        p7501_diag_sample_transitions(2200U, &phase_b, NULL);

        printf("Boot Transitions (%s):\n", (mode == 0U) ? "NORMAL SW_UART=HiZ" : "JIG SW_UART=LOW");
        printf("  Phase A (0..1300ms)   : GP2=%" PRIu32 " GP3=%" PRIu32 " GP4=%" PRIu32 " GP5=%" PRIu32 "\n",
               phase_a.rxd, phase_a.nsw5v, phase_a.txd, phase_a.xreset);
        printf("  Phase B (1300..3500ms): GP2=%" PRIu32 " GP3=%" PRIu32 " GP4=%" PRIu32 " GP5=%" PRIu32 "\n",
               phase_b.rxd, phase_b.nsw5v, phase_b.txd, phase_b.xreset);
    }

    p7501_hw_isolate_bus_hiz();
    gpio_pull_up(P7501_PIN_RXD);
    printf("BUSDIAG: OK\n");
}
