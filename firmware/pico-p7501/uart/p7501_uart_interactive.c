/**
 * @file p7501_uart_interactive.c
 * @brief Interactive P7501 TX/RX and BREAK probes (`UARTSEND`, `UARTPROBE`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_uart_interactive.h"
#include "uart/p7501_uart_capture.h"
#include "uart/p7501_uart_bitbang.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "pico/stdlib.h"
#include "hardware/gpio.h"

void p7501_cmd_uart_send(const char *arg) {
    /* Default to SW_UART=Hi-Z so Q7510 releases XFWE and Q7514 conducts TX toward AM34.
     * Prefix payload with "LOW:" to intentionally probe Bootrom while holding SW_UART=LOW. */
    bool sw_uart_out_low = false;
    const char *payload = (arg != NULL) ? arg : "";
    if ((payload[0] == 'L') && (payload[1] == 'O') && (payload[2] == 'W') && (payload[3] == ':')) {
        sw_uart_out_low = true;
        payload += 4;
    }

    char tx_str[128];
    if (*payload != '\0') {
        snprintf(tx_str, sizeof(tx_str), "%s\r\n", payload);
    } else {
        snprintf(tx_str, sizeof(tx_str), "\r\n");
    }

    p7501_uart_configure_pins(sw_uart_out_low, true, true, false);
    sleep_us(200U);

    const size_t slen = strlen(tx_str);
    p7501_uart_tx_bytes((const uint8_t *)tx_str, slen, 20U);

    uint8_t *rx_buf = p7501_uart_rx_buffer(NULL);
    const uint32_t rx_len = p7501_uart_capture_window(2500000UL, 4096U);
    p7501_uart_restore_default_state();

    printf(
        "\n=== UARTSEND REPLY (%" PRIu32 " bytes, SW_UART=%s) ===\n",
        rx_len,
        sw_uart_out_low ? "LOW" : "Hi-Z"
    );
    p7501_uart_print_ascii_stream(rx_buf, rx_len);
    printf("\nUARTSEND: OK\n");
}

void p7501_cmd_uart_probe(const char *arg) {
    uint32_t pmode = 5U;
    if ((arg != NULL) && (*arg != '\0')) {
        pmode = (uint32_t)atoi(arg);
    }

    const bool sw_uart_low = ((pmode == 5U) || (pmode == 6U) || (pmode == 7U));
    const bool start_sdata_low = ((pmode == 7U) || (pmode == 8U) || (pmode == 2U));
    p7501_uart_configure_pins(sw_uart_low, true, !start_sdata_low, false);
    p7501_uart_pulse_reset_and_settle();

    uint8_t *rx_buf = p7501_uart_rx_buffer(NULL);
    uint32_t rx_len = 0U;
    const uint32_t t0_us = time_us_32();
    bool prev_high = gpio_get(P7501_PIN_RXD);
    bool break_released = false;
    uint32_t last_probe_us = 0U;
    uint8_t probe_idx = 0U;
    static const uint8_t probe_chars[8] = { '\r', '\n', ' ', '?', 0x1BU, 0x03U, 'd', '\r' };

    while ((time_us_32() - t0_us) < 4500000UL) {
        const uint32_t now_us = time_us_32();
        if (((pmode == 7U) || (pmode == 8U)) && (!break_released) && ((now_us - t0_us) >= 550000UL)) {
            gpio_put(P7501_PIN_TXD, true);
            break_released = true;
        }
        if ((pmode == 6U) && ((now_us - t0_us) >= 150000UL) && ((now_us - last_probe_us) >= 80000UL)) {
            if (prev_high) {
                p7501_uart_tx_byte(probe_chars[probe_idx % 8U]);
                probe_idx++;
                last_probe_us = time_us_32();
            }
        }

        if (!p7501_uart_poll_capture_byte(&prev_high, &rx_len, 8192U)) {
            tight_loop_contents();
        }
    }

    p7501_uart_restore_default_state();

    printf("\n=== UARTPROBE MODE %" PRIu32 " CAPTURE (%" PRIu32 " bytes) ===\n", pmode, rx_len);
    p7501_uart_print_ascii_stream(rx_buf, rx_len);
    printf("\nUARTPROBE: OK\n");
}
