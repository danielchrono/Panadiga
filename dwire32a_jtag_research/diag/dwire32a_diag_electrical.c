/**
 * @file dwire32a_diag_electrical.c
 * @brief Implementation of P7501 electrical matrix checks, contention tests, and bus sniffing.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "diag/dwire32a_diag_electrical.h"
#include "hal/dwire32a_hw.h"
#include "pico/stdlib.h"
#include "hardware/gpio.h"
#include "hardware/adc.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>

static void configure_busdiag_bus_hiz_and_sw_uart(bool sw_uart_low) {
    dwire_hw_isolate_bus_hiz();
    gpio_disable_pulls(DWIRE_PIN_EXTRG);
    gpio_disable_pulls(DWIRE_PIN_SCLK);
    gpio_disable_pulls(DWIRE_PIN_SDATA);
    gpio_disable_pulls(DWIRE_PIN_NRESET);

    if (sw_uart_low) {
        gpio_init(DWIRE_PIN_VDD_ADC);
        gpio_set_drive_strength(DWIRE_PIN_VDD_ADC, GPIO_DRIVE_STRENGTH_12MA);
        gpio_put(DWIRE_PIN_VDD_ADC, false);
        gpio_set_dir(DWIRE_PIN_VDD_ADC, GPIO_OUT);
    } else {
        adc_gpio_init(DWIRE_PIN_VDD_ADC);
        gpio_disable_pulls(DWIRE_PIN_VDD_ADC);
    }
}

void dwire_diag_sample_transitions(
    uint32_t duration_ms,
    dwire_bus_transitions_t *out_counts,
    dwire_bus_levels_t *out_final_levels
) {
    dwire_bus_transitions_t counts = { 0U, 0U, 0U, 0U };
    uint32_t all = gpio_get_all();
    bool last_extrg  = ((all & (1UL << DWIRE_PIN_EXTRG)) != 0U);
    bool last_sclk   = ((all & (1UL << DWIRE_PIN_SCLK)) != 0U);
    bool last_sdata  = ((all & (1UL << DWIRE_PIN_SDATA)) != 0U);
    bool last_nreset = ((all & (1UL << DWIRE_PIN_NRESET)) != 0U);

    const absolute_time_t end_time = make_timeout_time_ms(duration_ms);
    while (get_absolute_time() < end_time) {
        all = gpio_get_all();
        const bool cur_extrg  = ((all & (1UL << DWIRE_PIN_EXTRG)) != 0U);
        const bool cur_sclk   = ((all & (1UL << DWIRE_PIN_SCLK)) != 0U);
        const bool cur_sdata  = ((all & (1UL << DWIRE_PIN_SDATA)) != 0U);
        const bool cur_nreset = ((all & (1UL << DWIRE_PIN_NRESET)) != 0U);

        if (cur_extrg  != last_extrg)  { counts.extrg++;  last_extrg  = cur_extrg;  }
        if (cur_sclk   != last_sclk)   { counts.sclk++;   last_sclk   = cur_sclk;   }
        if (cur_sdata  != last_sdata)  { counts.sdata++;  last_sdata  = cur_sdata;  }
        if (cur_nreset != last_nreset) { counts.nreset++; last_nreset = cur_nreset; }
    }

    if (out_counts != NULL) {
        *out_counts = counts;
    }
    if (out_final_levels != NULL) {
        out_final_levels->extrg  = last_extrg;
        out_final_levels->sclk   = last_sclk;
        out_final_levels->sdata  = last_sdata;
        out_final_levels->nreset = last_nreset;
    }
}

void dwire_diag_cmd_sniff(const char *arg) {
    uint32_t duration_ms = 250U;
    if (arg != NULL && *arg != '\0') {
        duration_ms = (uint32_t)atoi(arg);
        if (duration_ms == 0U) { duration_ms = 250U; }
        if (duration_ms > 10000U) { duration_ms = 10000U; }
    }

    printf("[SNIFF] Passively monitoring GP2/EXTRG, GP3/SCLK, GP4/SDATA, GP5/nRESET for %" PRIu32 " ms...\n",
           duration_ms);
    dwire_hw_isolate_bus_hiz();
    sleep_us(200);

    dwire_bus_transitions_t counts;
    dwire_bus_levels_t final_lvl;
    dwire_diag_sample_transitions(duration_ms, &counts, &final_lvl);

    dwire_hw_engage_bus_active();

    printf("SNIFF RESULT (%" PRIu32 " ms):\n", duration_ms);
    printf("  EXTRG  (GP2) : Final=%d, Transitions=%" PRIu32 "\n", final_lvl.extrg ? 1 : 0, counts.extrg);
    printf("  SCLK   (GP3) : Final=%d, Transitions=%" PRIu32 "\n", final_lvl.sclk ? 1 : 0, counts.sclk);
    printf("  SDATA  (GP4) : Final=%d, Transitions=%" PRIu32 "\n", final_lvl.sdata ? 1 : 0, counts.sdata);
    printf("  nRESET (GP5) : Final=%d, Transitions=%" PRIu32 "\n", final_lvl.nreset ? 1 : 0, counts.nreset);
    if (counts.extrg == 0U && counts.sclk == 0U && counts.sdata == 0U && counts.nreset == 0U) {
        if (final_lvl.sclk && final_lvl.sdata) {
            printf("  [STATUS] Lines idle HIGH (3.3V). Target is powered and pull-ups are intact.\n");
        } else if (!final_lvl.sclk && !final_lvl.sdata) {
            printf("  [STATUS] Lines steady LOW (0V). Target may be unpowered or lines shorted to GND.\n");
        } else {
            printf("  [STATUS] Lines static: GP2=%d, GP3=%d, GP4=%d, GP5=%d.\n",
                   final_lvl.extrg ? 1 : 0, final_lvl.sclk ? 1 : 0,
                   final_lvl.sdata ? 1 : 0, final_lvl.nreset ? 1 : 0);
        }
    } else {
        printf("  [STATUS] ACTIVE BUS ACTIVITY DETECTED! (GP2=%" PRIu32 ", GP3=%" PRIu32 ", GP4=%" PRIu32 ", GP5=%" PRIu32 ")\n",
               counts.extrg, counts.sclk, counts.sdata, counts.nreset);
    }
    printf("SNIFF: OK\n");
}

void dwire_diag_cmd_pincheck(void) {
    static const uint8_t pins[4] = {
        (uint8_t)DWIRE_PIN_EXTRG,
        (uint8_t)DWIRE_PIN_SCLK,
        (uint8_t)DWIRE_PIN_SDATA,
        (uint8_t)DWIRE_PIN_NRESET
    };
    static const char *names[4] = {
        "GP2 (Pin1 EXTRG )",
        "GP3 (Pin2 SCLK  )",
        "GP4 (Pin4 SDATA )",
        "GP5 (Pin5 nRESET)"
    };

    dwire_hw_isolate_bus_hiz();
    sleep_us(500);

    printf("\n=== DWIRE32A 6-PIN ELECTRICAL MATRIX (PINCHECK) ===\n");
    printf("Pin               | Hi-Z | Pull-Down | Pull-Up | Electrical State\n");
    printf("------------------+------+-----------+---------+----------------------------------------\n");

    for (size_t i = 0U; i < 4U; ++i) {
        const uint8_t p = pins[i];
        gpio_disable_pulls(p);
        sleep_us(300);
        const int hiz = gpio_get(p) ? 1 : 0;

        gpio_pull_down(p);
        sleep_us(300);
        const int pd = gpio_get(p) ? 1 : 0;

        gpio_pull_up(p);
        sleep_us(300);
        const int pu = gpio_get(p) ? 1 : 0;

        gpio_disable_pulls(p);

        const char *state = "UNKNOWN";
        if (pd == 1 && pu == 1) {
            state = "STRONG HIGH (External Pull-Up / 3.3V Active)";
        } else if (pd == 0 && pu == 0) {
            state = "STRONG LOW  (Grounded / Supervisor Held 0V)";
        } else if (pd == 0 && pu == 1) {
            state = "HI-Z / CMOS INPUT (No External Pull or Open)";
        }
        printf("%-17s |  %d   |     %d     |    %d    | %s\n", names[i], hiz, pd, pu, state);
    }

    adc_select_input(DWIRE_ADC_CH_VDD);
    gpio_disable_pulls(DWIRE_PIN_VDD_ADC);
    sleep_us(300);
    const float vdd_hiz = ((float)adc_read() * DWIRE_ADC_VREF) / DWIRE_ADC_RESOLUTION;

    gpio_pull_down(DWIRE_PIN_VDD_ADC);
    sleep_us(300);
    const float vdd_pd = ((float)adc_read() * DWIRE_ADC_VREF) / DWIRE_ADC_RESOLUTION;

    gpio_pull_up(DWIRE_PIN_VDD_ADC);
    sleep_us(300);
    const float vdd_pu = ((float)adc_read() * DWIRE_ADC_VREF) / DWIRE_ADC_RESOLUTION;
    gpio_disable_pulls(DWIRE_PIN_VDD_ADC);

    printf("GP26(Pin6 VDD  ) | %.2fV|   %.2fV   |  %.2fV  | Target VDD Rail Sense\n",
           vdd_hiz, vdd_pd, vdd_pu);

    dwire_hw_reset_assert();
    sleep_ms(15);
    const int rst_low_gp2 = gpio_get(DWIRE_PIN_EXTRG) ? 1 : 0;
    const int rst_low_gp3 = gpio_get(DWIRE_PIN_SCLK) ? 1 : 0;
    const int rst_low_gp4 = gpio_get(DWIRE_PIN_SDATA) ? 1 : 0;
    const int rst_low_gp5 = gpio_get(DWIRE_PIN_NRESET) ? 1 : 0;
    dwire_hw_reset_release();

    uint32_t rise_us = 0U;
    bool rose = false;
    for (rise_us = 0U; rise_us <= 50000U; rise_us += 10U) {
        if (gpio_get(DWIRE_PIN_NRESET)) {
            rose = true;
            break;
        }
        sleep_us(10);
    }

    dwire_bus_transitions_t post_counts;
    dwire_diag_sample_transitions(250U, &post_counts, NULL);

    printf("------------------------------------------------------------------------\n");
    printf("While GP5 (nRESET) asserted LOW (0V): GP2=%d, GP3=%d, GP4=%d, GP5=%d\n",
           rst_low_gp2, rst_low_gp3, rst_low_gp4, rst_low_gp5);
    if (rose) {
        printf("nRESET Release Rise Time (Hi-Z)     : %" PRIu32 " us\n", rise_us);
    } else {
        printf("nRESET Release Rise Time (Hi-Z)     : TIMEOUT (>50000 us, remained 0)\n");
    }
    printf("Post-Reset 250ms Transitions        : GP2=%" PRIu32 ", GP3=%" PRIu32 ", GP4=%" PRIu32 ", GP5=%" PRIu32 "\n",
           post_counts.extrg, post_counts.sclk, post_counts.sdata, post_counts.nreset);
    printf("PINCHECK: OK\n");

    dwire_hw_engage_bus_active();
}

void dwire_diag_cmd_busdiag(void) {
    printf("\n=== P7501 ELECTRICAL CONTENTION & DUAL-CHANNEL DIAGNOSTIC (BUSDIAG) ===\n");
    static const enum gpio_drive_strength ds_list[4] = {
        GPIO_DRIVE_STRENGTH_2MA,
        GPIO_DRIVE_STRENGTH_4MA,
        GPIO_DRIVE_STRENGTH_8MA,
        GPIO_DRIVE_STRENGTH_12MA
    };
    static const char *ds_names[4] = { "2mA", "4mA", "8mA", "12mA" };

    for (uint32_t st = 0U; st < 3U; ++st) {
        configure_busdiag_bus_hiz_and_sw_uart(st != 0U);
        if (st == 2U) {
            dwire_hw_reset_assert();
        } else {
            dwire_hw_reset_release();
        }
        sleep_ms(5);

        printf("State %" PRIu32 " (%s):\n", st,
               (st == 0U) ? "SW_UART=HiZ, XRESET=HiZ" :
               (st == 1U) ? "SW_UART=LOW, XRESET=HiZ" : "SW_UART=LOW, XRESET=LOW");

        for (uint32_t d = 0U; d < 4U; ++d) {
            gpio_init(DWIRE_PIN_SDATA);
            gpio_disable_pulls(DWIRE_PIN_SDATA);
            gpio_set_drive_strength(DWIRE_PIN_SDATA, ds_list[d]);
            gpio_put(DWIRE_PIN_SDATA, false);
            gpio_set_dir(DWIRE_PIN_SDATA, GPIO_OUT);
            sleep_us(25);
            const int pad_while_low = gpio_get(DWIRE_PIN_SDATA) ? 1 : 0;

            uint32_t ticks = 0U;
            gpio_set_dir(DWIRE_PIN_SDATA, GPIO_IN);
            while (!gpio_get(DWIRE_PIN_SDATA) && ticks < 50000U) {
                ticks++;
            }
            printf("  GP4 (MAX RXD) @ %-4s -> pad_low=%d, release_rise_ticks=%" PRIu32 "\n",
                   ds_names[d], pad_while_low, ticks);
            sleep_us(100);
        }
    }

    for (uint32_t mode = 0U; mode < 2U; ++mode) {
        configure_busdiag_bus_hiz_and_sw_uart(mode != 0U);
        sleep_ms(2);
        dwire_hw_reset_assert();
        sleep_ms(15);
        dwire_hw_reset_release();

        dwire_bus_transitions_t phase_a;
        dwire_bus_transitions_t phase_b;
        dwire_diag_sample_transitions(1300U, &phase_a, NULL);
        dwire_diag_sample_transitions(2200U, &phase_b, NULL);

        printf("Boot Transitions (%s):\n", (mode == 0U) ? "NORMAL SW_UART=HiZ" : "JIG SW_UART=LOW");
        printf("  Phase A (0..1300ms)   : GP2=%" PRIu32 " GP3=%" PRIu32 " GP4=%" PRIu32 " GP5=%" PRIu32 "\n",
               phase_a.extrg, phase_a.sclk, phase_a.sdata, phase_a.nreset);
        printf("  Phase B (1300..3500ms): GP2=%" PRIu32 " GP3=%" PRIu32 " GP4=%" PRIu32 " GP5=%" PRIu32 "\n",
               phase_b.extrg, phase_b.sclk, phase_b.sdata, phase_b.nreset);
    }

    adc_gpio_init(DWIRE_PIN_VDD_ADC);
    dwire_hw_isolate_bus_hiz();
    gpio_pull_up(DWIRE_PIN_EXTRG);
    printf("BUSDIAG: OK\n");
}
