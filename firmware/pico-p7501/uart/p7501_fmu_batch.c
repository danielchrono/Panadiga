/**
 * @file p7501_fmu_batch.c
 * @brief Batch single-byte FMU response sweep (`FMUBATCH`) over P7501 SC0.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_fmu_probe.h"
#include "uart/p7501_fmu_internal.h"
#include "uart/p7501_uart_capture.h"
#include "uart/p7501_uart_bitbang.h"

#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include "pico/stdlib.h"

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
        p7501_fmu_phase_state_t st;
        p7501_fmu_start_phase_session(true, &prev_high, &st);

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
                        p7501_fmu_buf_contains_tag_at_eol(rx_buf, rx_len, 20U, "seri", 4U)) {
                        p7501_uart_tx_byte(bval);
                        sent = true;
                        tx_us = time_us_32();
                    } else if (!st.wait1_seen) {
                        p7501_fmu_handle_phase_triggers(&st, rx_buf, 0U, rx_len, now_us);
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
