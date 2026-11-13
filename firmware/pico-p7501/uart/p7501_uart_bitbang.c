/**
 * @file p7501_uart_bitbang.c
 * @brief Shared 115200 8N1 bit-banged UART primitives for connector P7501.
 *
 * Pin-safe invariants:
 *   - GP2 (P7501 Pin 6 UARTP2M_T): 115200 8N1 RX input with weak pull-up.
 *   - GP3 (P7501 Pin 5 NSW5V): Always High-Z input with pulls disabled.
 *   - GP4 (P7501 Pin 3 UARTM2P_T): Driven TX only when `config_txd_out` is set.
 *   - GP5 (P7501 Pin 2 XRESET): Strictly Open-Drain via `p7501_hw_reset_*()`.
 *   - GP26 (P7501 Pin 1 SW_UART): Restored cleanly via `p7501_hw_restore_sw_uart_adc0()`
 *     so SIO OE is cleared before returning to ADC0 Hi-Z.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_uart_bitbang.h"
#include "hal/p7501_hw.h"
#include "pico/stdlib.h"
#include "hardware/gpio.h"
#include <stdio.h>

static const uint32_t P7501_RX_BIT_US[8] = {
    13U, 22U, 30U, 39U, 48U, 56U, 65U, 74U
};

void p7501_uart_configure_pins(
    bool sw_uart_out_low,
    bool config_txd_out,
    bool txd_initial_high,
    bool use_12ma_drive
) {
    p7501_hw_isolate_bus_hiz();
    gpio_pull_up(P7501_PIN_RXD);          /* GP2 = UARTP2M_T (Target TX) */
    gpio_disable_pulls(P7501_PIN_NSW5V);  /* GP3 = NSW5V (Keep Hi-Z!) */

    if (sw_uart_out_low) {
        if (use_12ma_drive) {
            gpio_set_drive_strength(P7501_PIN_SW_UART_ADC, GPIO_DRIVE_STRENGTH_12MA);
        }
        p7501_hw_sw_uart_set_low(true);
    } else {
        p7501_hw_restore_sw_uart_adc0();
    }

    if (config_txd_out) {
        gpio_init(P7501_PIN_TXD);
        if (use_12ma_drive) {
            gpio_set_drive_strength(P7501_PIN_TXD, GPIO_DRIVE_STRENGTH_12MA);
        }
        gpio_put(P7501_PIN_TXD, txd_initial_high);
        gpio_set_dir(P7501_PIN_TXD, GPIO_OUT);
    }

    sleep_us(500);
}

void p7501_uart_restore_default_state(void) {
    p7501_hw_restore_sw_uart_adc0();
    p7501_hw_isolate_bus_hiz();
    gpio_pull_up(P7501_PIN_RXD);
}

void p7501_uart_pulse_reset_and_settle(void) {
    p7501_hw_reset_assert();
    sleep_ms(15);
    p7501_hw_reset_release();

    const uint32_t settle_t = time_us_32();
    while (!gpio_get(P7501_PIN_RXD) && (time_us_32() - settle_t) < 5000UL) {}
}

void p7501_uart_tx_byte(uint8_t byte_val) {
    uint32_t bt = time_us_32();
    gpio_put(P7501_PIN_TXD, false); /* Start bit */
    for (uint32_t b = 0U; b < 8U; ++b) {
        bt += 9UL;
        while ((int32_t)(time_us_32() - bt) < 0) {}
        gpio_put(P7501_PIN_TXD, ((byte_val >> b) & 1U) != 0U);
    }
    bt += 9UL;
    while ((int32_t)(time_us_32() - bt) < 0) {}
    gpio_put(P7501_PIN_TXD, true); /* Stop bit */
}

void p7501_uart_tx_bytes(const uint8_t *data, size_t len, uint32_t inter_byte_delay_us) {
    for (size_t i = 0U; i < len; ++i) {
        p7501_uart_tx_byte(data[i]);
        if (inter_byte_delay_us > 0U) {
            sleep_us(inter_byte_delay_us);
        }
    }
}

bool p7501_uart_try_sample_byte(bool *prev_high, uint8_t *out_byte, uint32_t *out_t0_us) {
    const bool cur = gpio_get(P7501_PIN_RXD);
    if (!(*prev_high && !cur)) {
        *prev_high = cur;
        return false;
    }

    const uint32_t t0 = time_us_32();
    /* Verify Start bit is still LOW at +4 us (noise rejection) */
    while ((time_us_32() - t0) < 4UL) {}
    if (gpio_get(P7501_PIN_RXD)) {
        *prev_high = true;
        return false;
    }

    uint8_t byte_val = 0U;
    for (uint32_t b = 0U; b < 8U; ++b) {
        while ((time_us_32() - t0) < P7501_RX_BIT_US[b]) {}
        if (gpio_get(P7501_PIN_RXD)) {
            byte_val |= (uint8_t)(1U << b);
        }
    }
    /* Advance into Stop bit (+82 us) */
    while ((time_us_32() - t0) < 82UL) {}

    if (out_byte != NULL) {
        *out_byte = byte_val;
    }
    if (out_t0_us != NULL) {
        *out_t0_us = t0;
    }
    *prev_high = gpio_get(P7501_PIN_RXD);
    return true;
}

void p7501_uart_print_ascii_stream(const uint8_t *buf, uint32_t len) {
    for (uint32_t i = 0U; i < len; ++i) {
        const uint8_t c = buf[i];
        if (c == '\r') {
            continue;
        }
        if (c == '\n' || (c >= 0x20U && c <= 0x7EU)) {
            putchar((int)c);
        } else {
            printf("<%02X>", (unsigned)c);
        }
    }
}
