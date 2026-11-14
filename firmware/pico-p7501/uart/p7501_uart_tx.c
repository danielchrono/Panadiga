/**
 * @file p7501_uart_tx.c
 * @brief Active PIO1 TX (`UARTTXHEX`), pattern-triggered inject (`WAITTX`), and `BRIDGE`.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_uart_tx.h"

#include "common/p7501_hex.h"
#include "hal/p7501_led.h"
#include "p7501_types.h"
#include "uart/p7501_uart_codec.h"
#include "uart/p7501_uart_pio_engine.h"

#include "pico/stdlib.h"
#include "pico/time.h"

#include <inttypes.h>
#include <stdio.h>
#include <string.h>

static void emit_printable_byte(uint8_t b) {
    if (b == '\r') {
        return;
    }
    if (b == '\n' || (b >= 0x20u && b <= 0x7Eu)) {
        putchar((int)b);
    } else {
        printf("[%02X]", (unsigned)b);
    }
}

void p7501_cmd_uart_tx_hex(const char *arg) {
    char tok1[(P7501_UART_HEX_MAX_BYTES * 2u) + 12u] = {0};
    char tok2[24] = {0};
    const char *cur = p7501_next_token(arg, tok1, sizeof(tok1));
    if (cur == NULL || tok1[0] == '\0') {
        printf("[UARTTXHEX] ERROR: Usage UARTTXHEX [LOW:]<hex|BRK> [rx_ms]\n");
        return;
    }

    bool force_low = false;
    const char *payload_tok = tok1;
    if ((tok1[0] == 'L' || tok1[0] == 'l') && (tok1[1] == 'O' || tok1[1] == 'o') &&
        (tok1[2] == 'W' || tok1[2] == 'w') && tok1[3] == ':') {
        force_low = true;
        payload_tok = tok1 + 4;
    }

    uint32_t wait_ms = 1500u;
    if (p7501_next_token(cur, tok2, sizeof(tok2)) != NULL) {
        wait_ms = p7501_parse_uint32_clamped(tok2, 1500u, 100u, 20000u);
    }

    uint8_t tx_buf[P7501_UART_HEX_MAX_BYTES];
    size_t tx_len = 0u;
    const bool is_break = p7501_ascii_ieq(payload_tok, "BRK") || p7501_ascii_ieq(payload_tok, "BREAK");
    if (!is_break && !p7501_decode_hex_bytes(payload_tok, tx_buf, sizeof(tx_buf), &tx_len)) {
        printf("[UARTTXHEX] ERROR: Invalid hex payload '%s'.\n", payload_tok);
        return;
    }

    p7501_led_set_auto_mode(LED_MODE_BUS_ACTIVE);
    p7501_pio_begin_rx_session(P7501_BOOT_ACTION_PASSIVE);
    if (is_break) {
        p7501_pio_tx_break_ms(120u, force_low);
    } else {
        p7501_pio_tx_bytes(tx_buf, tx_len, force_low);
    }

    const absolute_time_t end_t = make_timeout_time_ms(wait_ms);
    uint32_t rx_count = 0u;
    uint8_t byte = 0u;
    while (!time_reached(end_t)) {
        while (p7501_pio_pop_rx_byte(&byte)) {
            ++rx_count;
            emit_printable_byte(byte);
        }
    }

    p7501_pio_end_session();
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);
    if (rx_count > 0u) {
        putchar('\n');
    }
    printf("UARTTXHEX: OK (tx=%s, sw_uart=%s, rx=%" PRIu32 " bytes)\n",
           is_break ? "BREAK_120MS" : "HEX", force_low ? "LOW" : "HIZ_Q7514_ON", rx_count);
}

void p7501_cmd_waittx(const char *arg) {
    p7501_waittx_spec_t spec;
    if (!p7501_parse_waittx_args(arg, &spec)) {
        printf("[WAITTX] ERROR: Usage WAITTX <timeout_ms> <NORMAL|JIG|JIG:ms|NOPULL> \"<trigger>\" <hex|BRK> [post_ms]\n");
        return;
    }

    printf("=== WAITTX: baud=%" PRIu32 " timeout=%" PRIu32 "ms hold=%" PRIu32 "ms trigger=\"%s\" post=%" PRIu32 "ms ===\n",
           p7501_pio_get_baud(), spec.timeout_ms, spec.jig_hold_ms, spec.trigger, spec.post_rx_ms);
    fflush(stdout);

    p7501_led_set_auto_mode(LED_MODE_BUS_ACTIVE);
    p7501_pio_begin_rx_session_ex(spec.boot_action, spec.jig_hold_ms);

    uint8_t ring[P7501_WAITTX_MAX_TRIGGER + 1u];
    size_t ring_len = 0u;
    bool triggered = false;
    uint32_t rx_count = 0u;
    uint8_t byte = 0u;
    const absolute_time_t wait_end = make_timeout_time_ms(spec.timeout_ms);

    while (!time_reached(wait_end)) {
        if (p7501_pio_poll_sw_uart_release()) {
            printf("\n[SW_UART] Released to Hi-Z -> Q7514 active\n");
            fflush(stdout);
        }
        while (p7501_pio_pop_rx_byte(&byte)) {
            ++rx_count;
            emit_printable_byte(byte);
            if (!triggered && p7501_rolling_match_push(
                                  ring, &ring_len, sizeof(ring), byte, spec.trigger
                              )) {
                triggered = true;
                break;
            }
        }
        if (triggered) {
            break;
        }
    }

    if (triggered) {
        printf("\n[WAITTX] Trigger matched! Injecting payload via Q7514 (SW_UART=Hi-Z)...\n");
        fflush(stdout);
        if (spec.tx_break) {
            p7501_pio_tx_break_ms(120u, false);
        } else {
            p7501_pio_tx_bytes(spec.tx_bytes, spec.tx_len, false);
        }

        const absolute_time_t post_end = make_timeout_time_ms(spec.post_rx_ms);
        while (!time_reached(post_end)) {
            while (p7501_pio_pop_rx_byte(&byte)) {
                ++rx_count;
                emit_printable_byte(byte);
            }
        }
    }

    p7501_pio_end_session();
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);
    printf("\nWAITTX: OK (triggered=%s, rx=%" PRIu32 " bytes)\n",
           triggered ? "YES" : "NO", rx_count);
}

void p7501_cmd_bridge(const char *arg) {
    char tok1[24] = {0};
    char tok2[24] = {0};
    const uint32_t orig_baud = p7501_pio_get_baud();
    p7501_boot_action_t action = P7501_BOOT_ACTION_PASSIVE;
    uint32_t jig_hold_ms = 0u;

    const char *cur = p7501_next_token(arg, tok1, sizeof(tok1));
    if (cur != NULL && tok1[0] != '\0') {
        const uint32_t req = p7501_parse_uint32_clamped(tok1, orig_baud, P7501_UART_MIN_BAUD, P7501_UART_MAX_BAUD);
        (void)p7501_pio_set_baud(req);
        if (p7501_next_token(cur, tok2, sizeof(tok2)) != NULL) {
            action = p7501_parse_boot_spec(tok2, &jig_hold_ms);
        }
    }

    printf("[BRIDGE] USB CDC <-> P7501 SC0 %" PRIu32 " 8N1 active (exit: send '~~~').\n",
           p7501_pio_get_baud());
    fflush(stdout);

    p7501_led_set_auto_mode(LED_MODE_GDB_RSP);
    p7501_pio_begin_rx_session_ex(action, jig_hold_ms);

    uint8_t tilde_run = 0u;
    uint8_t rx_byte = 0u;
    while (true) {
        p7501_led_tick();
        (void)p7501_pio_poll_sw_uart_release();
        while (p7501_pio_pop_rx_byte(&rx_byte)) {
            putchar_raw((int)rx_byte);
        }
        fflush(stdout);

        const int ch = getchar_timeout_us(500u);
        if (ch == PICO_ERROR_TIMEOUT) {
            continue;
        }
        if (ch == '~') {
            ++tilde_run;
            if (tilde_run >= 3u) {
                break;
            }
            continue;
        }
        while (tilde_run > 0u) {
            const uint8_t t = (uint8_t)'~';
            p7501_pio_tx_bytes(&t, 1u, false);
            --tilde_run;
        }
        const uint8_t tx_b = (uint8_t)ch;
        p7501_pio_tx_bytes(&tx_b, 1u, false);
    }

    p7501_pio_end_session();
    (void)p7501_pio_set_baud(orig_baud);
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);
    printf("\nBRIDGE: OK\n");
}
