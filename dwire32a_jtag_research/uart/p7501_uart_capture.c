/**
 * @file p7501_uart_capture.c
 * @brief Shared 16 KB P7501 RX buffer and `BOOTCAP`/`UARTMON`/`JIGBOOT` implementation.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_uart_capture.h"
#include "uart/p7501_uart_bitbang.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include "pico/stdlib.h"
#include "hardware/gpio.h"

/** Single shared RX capture buffer (16 KB) reused across all UART/FMU probes. */
static uint8_t s_p7501_rx_buf[16384];

uint8_t *p7501_uart_rx_buffer(size_t *out_capacity) {
    if (out_capacity != NULL) {
        *out_capacity = sizeof(s_p7501_rx_buf);
    }
    return s_p7501_rx_buf;
}

bool p7501_uart_poll_capture_byte(bool *prev_high, uint32_t *rx_len, uint32_t max_cap) {
    uint8_t rx_byte = 0U;
    if (!p7501_uart_try_sample_byte(prev_high, &rx_byte, NULL)) {
        return false;
    }
    if (*rx_len < max_cap) {
        s_p7501_rx_buf[(*rx_len)++] = rx_byte;
    }
    return true;
}

uint32_t p7501_uart_capture_window(uint32_t dur_us, uint32_t max_cap) {
    uint32_t rx_len = 0U;
    const uint32_t start_us = time_us_32();
    bool prev_high = gpio_get(DWIRE_PIN_EXTRG);
    while ((time_us_32() - start_us) < dur_us) {
        if (!p7501_uart_poll_capture_byte(&prev_high, &rx_len, max_cap)) {
            tight_loop_contents();
        }
    }
    return rx_len;
}

void p7501_cmd_boot_capture(const char *arg, bool do_reset, bool is_jig, const char *cmd_name) {
    uint32_t duration_ms = 7500U;
    if ((arg != NULL) && (*arg != '\0')) {
        const uint32_t parsed = (uint32_t)strtoul(arg, NULL, 10);
        if ((parsed >= 100U) && (parsed <= 30000U)) {
            duration_ms = parsed;
        }
    }

    p7501_uart_configure_pins(is_jig, false, false, false);
    if (do_reset) {
        p7501_uart_pulse_reset_and_settle();
    } else {
        sleep_ms(5U);
    }

    const uint32_t rx_len = p7501_uart_capture_window(
        duration_ms * 1000UL,
        (uint32_t)(sizeof(s_p7501_rx_buf) - 1U)
    );

    /* Restore SCLK/SDATA/SW_UART/nRESET into safe Hi-Z so target continues undisturbed */
    gpio_set_dir(DWIRE_PIN_SCLK, GPIO_IN);
    gpio_set_dir(DWIRE_PIN_SDATA, GPIO_IN);
    gpio_pull_up(DWIRE_PIN_SDATA);
    gpio_set_dir(DWIRE_PIN_VDD_ADC, GPIO_IN);
    gpio_disable_pulls(DWIRE_PIN_VDD_ADC);

    printf(
        "\n=== %s: PANASONIC 115200 8N1 UART CAPTURE ON GP2 (%" PRIu32 " bytes, %" PRIu32 " ms) ===\n",
        cmd_name,
        rx_len,
        duration_ms
    );
    p7501_uart_print_ascii_stream(s_p7501_rx_buf, rx_len);
    printf("\n%s: OK (Bus kept in Hi-Z with pull-up so target continues running)\n", cmd_name);
}
