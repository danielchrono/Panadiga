/**
 * @file p7501_fmu_probe.c
 * @brief Panasonic Bootrom FMU handshake probes (`FMUSCAN`, `FMUTEST`, `FMUBATCH`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_fmu_probe.h"
#include "uart/p7501_uart_capture.h"
#include "uart/p7501_uart_bitbang.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "pico/stdlib.h"
#include "hardware/gpio.h"

typedef struct fmu_phase_state {
    bool sw_uart_flipped;
    bool sent_after_serial_msg;
    bool wait1_seen;
    uint32_t wait1_us;
} fmu_phase_state_t;

static bool fmu_buf_contains_tag_at_eol(
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
    return fmu_buf_contains_tag_at_eol(buf, len, scan_start, "DDR CHECK", 9U);
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

static void fmu_handle_phase_triggers(
    fmu_phase_state_t *st,
    const uint8_t *rx_buf,
    uint32_t fmode,
    uint32_t rx_len,
    uint32_t now_us
) {
    if ((!st->sw_uart_flipped) && (fmode == 2U) &&
        fmu_buf_contains_tag_at_eol(rx_buf, rx_len, 20U, "seri", 4U)) {
        gpio_put(DWIRE_PIN_VDD_ADC, true);
        st->sw_uart_flipped = true;
    }
    if ((!st->sw_uart_flipped) && (fmode == 3U) &&
        fmu_buf_is_ddr_ok_eol(rx_buf, rx_len)) {
        gpio_put(DWIRE_PIN_VDD_ADC, true);
        st->sw_uart_flipped = true;
    }
    if ((!st->wait1_seen) && fmu_buf_is_wait1_eol(rx_buf, rx_len)) {
        st->wait1_seen = true;
        st->wait1_us = now_us;
    }
}

static uint8_t fmu_hex_nibble(char c) {
    if ((c >= '0') && (c <= '9')) { return (uint8_t)(c - '0'); }
    if ((c >= 'a') && (c <= 'f')) { return (uint8_t)(c - 'a' + 10); }
    if ((c >= 'A') && (c <= 'F')) { return (uint8_t)(c - 'A' + 10); }
    return 0xFFU;
}

static size_t fmu_parse_hex_payload(const char *hex_str, uint8_t *out, size_t max_len) {
    size_t out_len = 0U;
    if ((hex_str == NULL) || (*hex_str == '\0')) {
        out[0] = 0x0DU;
        return 1U;
    }

    const char *p = hex_str;
    while ((*p != '\0') && (out_len < max_len)) {
        while ((*p == ' ') || (*p == ',') || (*p == '\t')) {
            p++;
        }
        if (*p == '\0') {
            break;
        }
        const uint8_t hi = fmu_hex_nibble(*p++);
        if (hi == 0xFFU) {
            continue;
        }
        const uint8_t lo = (*p != '\0') ? fmu_hex_nibble(*p) : 0xFFU;
        if (lo != 0xFFU) {
            p++;
            out[out_len++] = (uint8_t)((hi << 4U) | lo);
        } else {
            out[out_len++] = hi;
        }
    }

    if (out_len == 0U) {
        out[0] = 0x0DU;
        out_len = 1U;
    }
    return out_len;
}

static void fmu_start_phase_session(
    bool use_12ma,
    bool *out_prev_high,
    fmu_phase_state_t *out_st
) {
    p7501_uart_configure_pins(true, true, true, use_12ma);
    p7501_uart_pulse_reset_and_settle();
    *out_prev_high = gpio_get(DWIRE_PIN_EXTRG);
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
    fmu_phase_state_t st;
    fmu_start_phase_session(false, &prev_high, &st);

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
                fmu_handle_phase_triggers(&st, rx_buf, fmode, rx_len, now_us);
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
                gpio_put(DWIRE_PIN_VDD_ADC, true);
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
    const size_t tx_len = fmu_parse_hex_payload(hex_str, tx_bytes, sizeof(tx_bytes));

    uint8_t *rx_buf = p7501_uart_rx_buffer(NULL);
    bool prev_high = true;
    fmu_phase_state_t st;
    fmu_start_phase_session(true, &prev_high, &st);

    uint32_t rx_len = 0U;
    uint32_t next_tx_us = 0U;
    uint32_t tx_count = 0U;
    const uint32_t t0_us = time_us_32();

    while ((time_us_32() - t0_us) < 4000000UL) {
        const uint32_t now_us = time_us_32();
        if (p7501_uart_poll_capture_byte(&prev_high, &rx_len, 8192U) && (!st.wait1_seen)) {
            fmu_handle_phase_triggers(&st, rx_buf, fmode, rx_len, now_us);
            if ((fmode == 4U) && (!st.sent_after_serial_msg) &&
                fmu_buf_contains_tag_at_eol(rx_buf, rx_len, 20U, "seri", 4U)) {
                p7501_uart_tx_bytes(tx_bytes, tx_len, 20U);
                st.sent_after_serial_msg = true;
                tx_count++;
            }
            if (st.wait1_seen) {
                if ((fmode == 1U) && (!st.sw_uart_flipped)) {
                    gpio_put(DWIRE_PIN_VDD_ADC, true);
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

void p7501_cmd_fmu_batch(const char *arg) {
    uint32_t start_b = 0U;
    uint32_t count_b = 16U;
    uint32_t offset_ms = 20U;
    if ((arg != NULL) && (*arg != '\0')) {
        unsigned int s = 0U;
        unsigned int c = 16U;
        unsigned int o = 20U;
        const int n = sscanf(arg, "%u %u %u", &s, &c, &o);
        if (n >= 1) { start_b = (uint32_t)s; }
        if (n >= 2) { count_b = (uint32_t)c; }
        if (n >= 3) { offset_ms = (uint32_t)o; }
    }
    if (start_b > 255U) { start_b = 255U; }
    if ((start_b + count_b) > 256U) { count_b = 256U - start_b; }

    printf("\n=== FMUBATCH (0x%02" PRIX32 "..0x%02" PRIX32 ", offset=%" PRIu32 " ms after wait 1 second) ===\n",
           start_b, start_b + count_b - 1U, offset_ms);

    uint8_t *rx_buf = p7501_uart_rx_buffer(NULL);
    for (uint32_t b = start_b; b < (start_b + count_b); b++) {
        const uint8_t bval = (uint8_t)b;
        bool prev_high = true;
        fmu_phase_state_t st;
        fmu_start_phase_session(true, &prev_high, &st);

        uint32_t rx_len = 0U;
        bool sent = false;
        uint8_t post_byte = 0U;
        bool got_post = false;
        uint32_t post_dt = 0U;
        uint32_t tx_us = 0U;
        uint32_t target_tx_us = 0U;
        const uint32_t t0_us = time_us_32();

        while ((time_us_32() - t0_us) < 2000000UL) {
            const uint32_t now_us = time_us_32();
            if (p7501_uart_poll_capture_byte(&prev_high, &rx_len, 512U)) {
                if (!sent) {
                    if ((offset_ms == 8888U) &&
                        fmu_buf_contains_tag_at_eol(rx_buf, rx_len, 20U, "seri", 4U)) {
                        p7501_uart_tx_byte(bval);
                        sent = true;
                        tx_us = time_us_32();
                    } else if (!st.wait1_seen) {
                        fmu_handle_phase_triggers(&st, rx_buf, 0U, rx_len, now_us);
                        if (st.wait1_seen) {
                            target_tx_us = now_us + (offset_ms * 1000UL);
                        }
                    }
                } else if (!got_post) {
                    post_byte = rx_buf[rx_len - 1U];
                    post_dt = time_us_32() - tx_us;
                    got_post = true;
                    break;
                }
            }

            if (st.wait1_seen && (!sent) && (offset_ms != 8888U) && (now_us >= target_tx_us)) {
                p7501_uart_tx_byte(bval);
                sent = true;
                tx_us = time_us_32();
            }
            tight_loop_contents();
        }

        const char printable = ((post_byte >= 0x20U) && (post_byte <= 0x7EU)) ? (char)post_byte : '.';
        printf("  0x%02X: trig=%d delay=%" PRIu32 " us first=0x%02X ('%c')\n",
               bval, sent ? 1 : 0, post_dt, post_byte, printable);
        if (got_post && ((post_dt < 980000UL) || (post_dt == 0U))) {
            printf("  [!!! HIT ON BYTE 0x%02X !!!]\n", bval);
            break;
        }
    }

    p7501_uart_restore_default_state();
    printf("FMUBATCH: OK\n");
}
