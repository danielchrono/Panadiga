/**
 * @file p7501_fmu_probe.c
 * @brief Panasonic Bootrom FMU handshake probes (`FMUSCAN`, `FMUTEST`, `FMUBATCH`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_fmu_probe.h"
#include "uart/p7501_fmu_internal.h"
#include "uart/p7501_uart_capture.h"
#include "uart/p7501_uart_bitbang.h"
#include "common/p7501_hex.h"
#include "hal/p7501_hw.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "pico/stdlib.h"
#include "hardware/gpio.h"

bool p7501_fmu_buf_contains_tag_at_eol(
    const uint8_t *buf,
    uint32_t len,
    uint32_t start_min,
    const char *tag,
    size_t tag_len
) {
    if ((len < (start_min + (uint32_t)tag_len + 1U)) || (buf[len - 1U] != '\n')) {
        return false;
    }
    for (uint32_t k = start_min; (k + (uint32_t)tag_len) <= len; k++) {
        if (memcmp(&buf[k], tag, tag_len) == 0) {
            return true;
        }
    }
    return false;
}

static bool fmu_buf_is_ddr_ok_eol(const uint8_t *buf, uint32_t len) {
    if ((len < 14U) || (buf[len - 1U] != '\n')) {
        return false;
    }
    const uint32_t scan_start = (len >= 18U) ? (len - 18U) : 0U;
    return p7501_fmu_buf_contains_tag_at_eol(buf, len, scan_start, "DDR CHECK", 9U);
}

static bool fmu_buf_is_wait1_eol(const uint8_t *buf, uint32_t len) {
    if ((len < 20U) || (buf[len - 1U] != '\n')) {
        return false;
    }
    if ((buf[len - 2U] != '.') && (buf[len - 3U] != '.')) {
        return false;
    }
    for (uint32_t k = len - 18U; k < len; k++) {
        if (buf[k] == '1') {
            return true;
        }
    }
    return false;
}

void p7501_fmu_handle_phase_triggers(
    p7501_fmu_phase_state_t *st,
    const uint8_t *rx_buf,
    uint32_t fmode,
    uint32_t rx_len,
    uint32_t now_us
) {
    if ((!st->sw_uart_flipped) && (fmode == 2U) &&
        p7501_fmu_buf_contains_tag_at_eol(rx_buf, rx_len, 20U, "seri", 4U)) {
        p7501_hw_sw_uart_set_low(false);
        st->sw_uart_flipped = true;
    }
    if ((!st->sw_uart_flipped) && (fmode == 3U) &&
        fmu_buf_is_ddr_ok_eol(rx_buf, rx_len)) {
        p7501_hw_sw_uart_set_low(false);
        st->sw_uart_flipped = true;
    }
    if ((!st->wait1_seen) && fmu_buf_is_wait1_eol(rx_buf, rx_len)) {
        st->wait1_seen = true;
        st->wait1_us = now_us;
    }
}

static size_t fmu_decode_or_default_cr(const char *hex_str, uint8_t *out, size_t max_len) {
    size_t out_len = 0U;
    if ((hex_str != NULL) && (*hex_str != '\0') &&
        p7501_decode_hex_bytes(hex_str, out, max_len, &out_len) && (out_len > 0U)) {
        return out_len;
    }
    out[0] = 0x0DU;
    return 1U;
}

void p7501_fmu_start_phase_session(
    bool use_12ma,
    bool *out_prev_high,
    p7501_fmu_phase_state_t *out_st
) {
    p7501_uart_configure_pins(true, true, true, use_12ma);
    p7501_uart_pulse_reset_and_settle();
    *out_prev_high = gpio_get(P7501_PIN_RXD);
    out_st->sw_uart_flipped = false;
    out_st->sent_after_serial_msg = false;
    out_st->wait1_seen = false;
    out_st->wait1_us = 0U;
}

void p7501_cmd_fmu_scan(const char *arg) {
    uint32_t fmode = 0U;
    if ((arg != NULL) && (*arg != '\0')) {
        fmode = (uint32_t)atoi(arg);
    }

    uint8_t *rx_buf = p7501_uart_rx_buffer(NULL);
    bool prev_high = true;
    p7501_fmu_phase_state_t st;
    p7501_fmu_start_phase_session(false, &prev_high, &st);

    uint32_t rx_len = 0U;
    const uint32_t t0_us = time_us_32();
    bool sweep_done = false;
    int hit_byte = -1;
    uint16_t bval = 0U;
    uint32_t next_tx_us = 0U;
    bool listening_after_tx = false;

    while ((time_us_32() - t0_us) < 4000000UL) {
        const uint32_t now_us = time_us_32();
        if (p7501_uart_poll_capture_byte(&prev_high, &rx_len, 8192U)) {
            if (!listening_after_tx) {
                const bool was_wait1 = st.wait1_seen;
                p7501_fmu_handle_phase_triggers(&st, rx_buf, fmode, rx_len, now_us);
                if ((!was_wait1) && st.wait1_seen) {
                    next_tx_us = now_us + 2000UL;
                }
            } else if ((hit_byte < 0) && (bval > 0U)) {
                hit_byte = (int)(bval - 1U);
                sweep_done = true;
            }
        }

        if (st.wait1_seen && (!sweep_done) && (now_us >= next_tx_us) && ((int32_t)(now_us - st.wait1_us) < 1100000L)) {
            if ((bval == 0U) && (fmode == 1U) && (!st.sw_uart_flipped)) {
                p7501_hw_sw_uart_set_low(false);
                st.sw_uart_flipped = true;
                sleep_us(50U);
            }
            if (bval < 256U) {
                p7501_uart_tx_byte((uint8_t)bval);
                bval++;
                listening_after_tx = true;
                next_tx_us = time_us_32() + 2200UL;
            } else {
                sweep_done = true;
            }
        }
        tight_loop_contents();
    }

    p7501_uart_restore_default_state();

    printf("\n=== FMUSCAN MODE %" PRIu32 " RESULT (hit=0x%02X/%d, bytes=%" PRIu32 ") ===\n",
           fmode, hit_byte >= 0 ? hit_byte : 0, hit_byte, rx_len);
    p7501_uart_print_ascii_stream(rx_buf, rx_len);
    printf("\nFMUSCAN: OK\n");
}

void p7501_cmd_fmu_test(const char *arg) {
    uint32_t raw_mode = 2U;
    const char *hex_str = "0D";
    if ((arg != NULL) && (*arg != '\0')) {
        char *endp = NULL;
        raw_mode = (uint32_t)strtoul(arg, &endp, 10);
        if (endp != NULL) {
            while (*endp == ' ') {
                endp++;
            }
            if (*endp != '\0') {
                hex_str = endp;
            }
        }
    }
    const uint32_t fmode = raw_mode % 10U;
    const bool repeat_25ms = (raw_mode >= 10U);

    uint8_t tx_bytes[64];
    const size_t tx_len = fmu_decode_or_default_cr(hex_str, tx_bytes, sizeof(tx_bytes));

    uint8_t *rx_buf = p7501_uart_rx_buffer(NULL);
    bool prev_high = true;
    p7501_fmu_phase_state_t st;
    p7501_fmu_start_phase_session(true, &prev_high, &st);

    uint32_t rx_len = 0U;
    uint32_t next_tx_us = 0U;
    uint32_t tx_count = 0U;
    const uint32_t t0_us = time_us_32();

    while ((time_us_32() - t0_us) < 4000000UL) {
        const uint32_t now_us = time_us_32();
        if (p7501_uart_poll_capture_byte(&prev_high, &rx_len, 8192U) && (!st.wait1_seen)) {
            p7501_fmu_handle_phase_triggers(&st, rx_buf, fmode, rx_len, now_us);
            if ((fmode == 4U) && (!st.sent_after_serial_msg) &&
                p7501_fmu_buf_contains_tag_at_eol(rx_buf, rx_len, 20U, "seri", 4U)) {
                p7501_uart_tx_bytes(tx_bytes, tx_len, 20U);
                st.sent_after_serial_msg = true;
                tx_count++;
            }
            if (st.wait1_seen) {
                if ((fmode == 1U) && (!st.sw_uart_flipped)) {
                    p7501_hw_sw_uart_set_low(false);
                    st.sw_uart_flipped = true;
                    sleep_us(50U);
                }
                next_tx_us = now_us + 2000UL;
            }
        }

        if (st.wait1_seen && ((tx_count == 0U) || (repeat_25ms && prev_high)) &&
            (now_us >= next_tx_us) && ((uint32_t)(now_us - st.wait1_us) < 1050000UL)) {
            p7501_uart_tx_bytes(tx_bytes, tx_len, 20U);
            tx_count++;
            next_tx_us = time_us_32() + 25000UL;
        }
        tight_loop_contents();
    }

    p7501_uart_restore_default_state();

    printf("\n=== FMUTEST MODE %" PRIu32 " (%u tx bytes x%" PRIu32 ") CAPTURE (%" PRIu32 " bytes) ===\n",
           raw_mode, (unsigned)tx_len, tx_count, rx_len);
    p7501_uart_print_ascii_stream(rx_buf, rx_len);
    printf("\nFMUTEST: OK\n");
}

