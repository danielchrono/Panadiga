/**
 * @file p7501_uart_stream.c
 * @brief Real-time PIO1 UART byte streamer (`UARTSTREAM`) and delta-us hex frame capture (`HEXCAP`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_uart_stream.h"

#include "hal/p7501_led.h"
#include "p7501_types.h"
#include "uart/p7501_uart_codec.h"
#include "uart/p7501_uart_pio_engine.h"

#include "pico/stdlib.h"
#include "pico/time.h"

#include <inttypes.h>
#include <stdio.h>
#include <string.h>

static void print_stream_byte(uint8_t b, uint32_t *col) {
    if (b == '\r') {
        return;
    }
    if (b == '\n') {
        putchar('\n');
        *col = 0u;
        return;
    }
    if (b >= 0x20u && b <= 0x7Eu) {
        putchar((int)b);
    } else {
        printf("[%02X]", (unsigned)b);
    }
    (*col)++;
    if (*col >= 96u) {
        *col = 0u;
    }
}

static absolute_time_t begin_pio_stream_window(
    const char *label,
    const p7501_window_boot_spec_t *spec
) {
    printf("=== %s: PIO1 %" PRIu32 " 8N1 (window=%" PRIu32 " ms, hold=%" PRIu32 " ms) ===\n",
           label, p7501_pio_get_baud(), spec->window_ms, spec->jig_hold_ms);
    fflush(stdout);
    p7501_led_set_auto_mode(LED_MODE_BUS_ACTIVE);
    p7501_pio_begin_rx_session_ex(spec->boot_action, spec->jig_hold_ms);
    return make_timeout_time_ms(spec->window_ms);
}

void p7501_cmd_uart_stream(const char *arg) {
    p7501_window_boot_spec_t spec;
    p7501_parse_window_boot_args(arg, 10000u, 200u, 120000u, &spec);

    const absolute_time_t end_t = begin_pio_stream_window("UARTSTREAM", &spec);
    uint32_t rx_count = 0u;
    uint32_t ferr_count = 0u;
    uint32_t col = 0u;
    uint8_t byte = 0u;

    while (!time_reached(end_t)) {
        if (p7501_pio_poll_sw_uart_release()) {
            printf("\n[SW_UART] Timed release -> Hi-Z ADC0 (Q7514 TX ready)\n");
            fflush(stdout);
        }
        if (p7501_pio_poll_and_clear_framing_err()) {
            ++ferr_count;
        }
        while (p7501_pio_pop_rx_byte(&byte)) {
            ++rx_count;
            print_stream_byte(byte, &col);
        }
    }

    const bool ovf = p7501_pio_had_rx_overflow();
    p7501_pio_end_session();
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);

    if (col > 0u) {
        putchar('\n');
    }
    printf("UARTSTREAM: OK (%" PRIu32 " bytes, framing_err=%" PRIu32 ", overflow=%s)\n",
           rx_count, ferr_count, ovf ? "YES" : "NO");
}

static void flush_hexcap_row(const uint8_t *row, size_t len, uint32_t delta_us) {
    if (len == 0u) {
        return;
    }
    printf("[+%7" PRIu32 "us] ", delta_us);
    for (size_t i = 0u; i < 16u; ++i) {
        if (i < len) {
            printf("%02X ", (unsigned)row[i]);
        } else {
            printf("   ");
        }
    }
    printf("| ");
    for (size_t i = 0u; i < len; ++i) {
        const char ch = (row[i] >= 0x20u && row[i] <= 0x7Eu) ? (char)row[i] : '.';
        putchar(ch);
    }
    putchar('\n');
}

#define HEXCAP_RAM_BUF_CAP 2048u

static void flush_hexcap_buffer(
    const uint8_t *bytes,
    const uint32_t *gaps_us,
    size_t count
) {
    uint8_t row[16];
    size_t row_len = 0u;
    uint32_t row_delta_us = 0u;

    for (size_t i = 0u; i < count; ++i) {
        if (row_len > 0u && gaps_us[i] >= 5000u) {
            flush_hexcap_row(row, row_len, row_delta_us);
            row_len = 0u;
        }
        if (row_len == 0u) {
            row_delta_us = gaps_us[i];
        }
        row[row_len++] = bytes[i];
        if (row_len >= sizeof(row)) {
            flush_hexcap_row(row, row_len, row_delta_us);
            row_len = 0u;
        }
    }
    flush_hexcap_row(row, row_len, row_delta_us);
}

void p7501_cmd_hexcap(const char *arg) {
    static uint8_t hexcap_bytes[HEXCAP_RAM_BUF_CAP];
    static uint32_t hexcap_gaps_us[HEXCAP_RAM_BUF_CAP];

    p7501_window_boot_spec_t spec;
    p7501_parse_window_boot_args(arg, 8000u, 200u, 120000u, &spec);

    const absolute_time_t end_t = begin_pio_stream_window("HEXCAP", &spec);
    uint32_t rx_count = 0u;
    uint32_t ferr_count = 0u;
    uint32_t last_byte_us = time_us_32();
    size_t buf_len = 0u;
    uint8_t byte = 0u;

    while (!time_reached(end_t)) {
        (void)p7501_pio_poll_sw_uart_release();
        if (p7501_pio_poll_and_clear_framing_err()) {
            ++ferr_count;
        }
        while (p7501_pio_pop_rx_byte(&byte)) {
            const uint32_t now_us = time_us_32();
            const uint32_t gap_us = now_us - last_byte_us;
            last_byte_us = now_us;
            if (buf_len >= HEXCAP_RAM_BUF_CAP) {
                flush_hexcap_buffer(hexcap_bytes, hexcap_gaps_us, buf_len);
                buf_len = 0u;
            }
            hexcap_bytes[buf_len] = byte;
            hexcap_gaps_us[buf_len] = gap_us;
            ++buf_len;
            ++rx_count;
        }
    }

    const bool ovf = p7501_pio_had_rx_overflow();
    p7501_pio_end_session();
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);

    flush_hexcap_buffer(hexcap_bytes, hexcap_gaps_us, buf_len);
    fflush(stdout);

    printf("HEXCAP: OK (%" PRIu32 " bytes, framing_err=%" PRIu32 ", overflow=%s)\n",
           rx_count, ferr_count, ovf ? "YES" : "NO");
}
