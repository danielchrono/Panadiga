/**
 * @file test_uart_codec.c
 * @brief Host C11 assertions for baud math, pulse snapping, hex decode, and WAITTX triggers.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "tests/test_uart_codec.h"
#include "tests/test_pana_logic.h"
#include "uart/p7501_uart_codec.h"
#include "common/p7501_hex.h"
#include "hal/p7501_led_logic.h"

#include <assert.h>
#include <stdio.h>
#include <string.h>

static void test_timeout_and_tokens(void) {
    char tok[32];
    const char *cur = "  \"Hit Any Key\"  0D0A 4500 ";

    assert(p7501_parse_uint32_clamped(NULL, 5000u, 100u, 30000u) == 5000u);
    assert(p7501_parse_uint32_clamped("50", 5000u, 100u, 30000u) == 100u);
    assert(p7501_parse_uint32_clamped("999999", 5000u, 100u, 30000u) == 30000u);
    assert(p7501_parse_uint32_clamped("  7500 ", 5000u, 100u, 30000u) == 7500u);

    cur = p7501_next_token(cur, tok, sizeof(tok));
    assert(cur != NULL);
    assert(strcmp(tok, "Hit Any Key") == 0);
    cur = p7501_next_token(cur, tok, sizeof(tok));
    assert(cur != NULL);
    assert(strcmp(tok, "0D0A") == 0);
    cur = p7501_next_token(cur, tok, sizeof(tok));
    assert(cur != NULL);
    assert(strcmp(tok, "4500") == 0);
    assert(p7501_next_token(cur, tok, sizeof(tok)) == NULL);
}

static void test_hex_and_baud_math(void) {
    uint8_t buf[16];
    size_t len = 0u;
    uint16_t div_i = 0u;
    uint8_t div_f = 0u;
    uint32_t raw_bps = 0u;

    assert(p7501_decode_hex_bytes("1B 5B:48-0D0A", buf, sizeof(buf), &len));
    assert(len == 5u);
    assert(buf[0] == 0x1Bu && buf[3] == 0x0Du && buf[4] == 0x0Au);
    assert(!p7501_decode_hex_bytes("0D0", buf, sizeof(buf), &len));
    assert(!p7501_decode_hex_bytes("ZZ", buf, sizeof(buf), &len));

    assert(p7501_baud_is_valid(115200u));
    assert(!p7501_baud_is_valid(300u));
    assert(p7501_baud_to_bit_ns(115200u) == 8680u);

    p7501_calc_pio_clkdiv_fp16(125000000u, 115200u, &div_i, &div_f);
    assert(div_i == 135u && div_f == 162u);

    p7501_calc_pio_clkdiv_fp16(150000000u, 115200u, &div_i, &div_f);
    assert(div_i == 162u && div_f == 195u);

    assert(p7501_snap_standard_baud(9u, &raw_bps) == 115200u);
    assert(raw_bps == 111111u);
    assert(p7501_snap_standard_baud(17u, &raw_bps) == 57600u);
    assert(p7501_snap_standard_baud(32u, &raw_bps) == 31250u);
    assert(p7501_snap_standard_baud(104u, &raw_bps) == 9600u);
}

static void test_trigger_and_waittx_parser(void) {
    uint8_t ring[16];
    size_t ring_len = 0u;
    const char *feed = "Loader ver0.61 -- Hit Any Key";
    bool hit = false;
    p7501_waittx_spec_t spec;

    for (size_t i = 0u; feed[i] != '\0'; ++i) {
        if (p7501_rolling_match_push(ring, &ring_len, sizeof(ring), (uint8_t)feed[i], "Hit Any Key")) {
            hit = true;
        }
    }
    assert(hit);

    assert(p7501_parse_waittx_args("20000 JIG \"Hit Any Key\" 1B0D 6000", &spec));
    assert(spec.timeout_ms == 20000u);
    assert(spec.boot_action == P7501_BOOT_ACTION_JIG);
    assert(strcmp(spec.trigger, "Hit Any Key") == 0);
    assert(spec.tx_len == 2u && spec.tx_bytes[0] == 0x1Bu && spec.tx_bytes[1] == 0x0Du);
    assert(spec.post_rx_ms == 6000u);
    assert(!spec.tx_break);

    assert(p7501_parse_waittx_args("5000 NORMAL wait BRK", &spec));
    assert(spec.boot_action == P7501_BOOT_ACTION_NORMAL);
    assert(spec.tx_break);
    assert(spec.post_rx_ms == 4000u);

    assert(!p7501_parse_waittx_args("5000 JIG trigger", &spec));

    assert(p7501_parse_waittx_args("8000 JIG:350 \"wait 1\" 0D 4000", &spec));
    assert(spec.boot_action == P7501_BOOT_ACTION_JIG);
    assert(spec.jig_hold_ms == 350u);
    assert(strcmp(spec.trigger, "wait 1") == 0);
    assert(spec.tx_len == 1u && spec.tx_bytes[0] == 0x0Du);
}

static void test_hex_window_and_led_logic(void) {
    uint8_t comma_buf[4] = { 0 };
    size_t comma_len = 0u;
    assert(p7501_decode_hex_bytes("1B,0D,0A", comma_buf, sizeof(comma_buf), &comma_len));
    assert(comma_len == 3u && comma_buf[0] == 0x1Bu && comma_buf[1] == 0x0Du && comma_buf[2] == 0x0Au);
    assert(p7501_ascii_ieq("ConFirm", "CONFIRM"));

    char upper[16];
    p7501_ascii_upper_copy(upper, sizeof(upper), "jig:350");
    assert(strcmp(upper, "JIG:350") == 0);

    p7501_window_boot_spec_t win;
    p7501_parse_window_boot_args("8000 JIG:350", 5000u, 200u, 30000u, &win);
    assert(win.window_ms == 8000u && win.boot_action == P7501_BOOT_ACTION_JIG && win.jig_hold_ms == 350u);
    p7501_parse_window_boot_args("4000 NOPULL", 5000u, 200u, 30000u, &win);
    assert(win.window_ms == 4000u && win.boot_action == P7501_BOOT_ACTION_PASSIVE && win.jig_hold_ms == 0u);

    p7501_led_state_t led;
    p7501_led_state_init(&led);
    assert(!led.manual_lock && led.mode == LED_MODE_HEARTBEAT_IDLE);
    assert(p7501_led_pattern_output(LED_MODE_HEARTBEAT_IDLE, 0u));
    assert(!p7501_led_pattern_output(LED_MODE_HEARTBEAT_IDLE, 3u));
    assert(p7501_led_pattern_output(LED_MODE_BUS_FAULT, 4u));
    assert(!p7501_led_pattern_output(LED_MODE_OFF, 0u));

    p7501_led_state_set_manual(&led, LED_MODE_GDB_RSP);
    assert(led.manual_lock && led.mode == LED_MODE_GDB_RSP);
    assert(!p7501_led_state_set_auto(&led, LED_MODE_HEARTBEAT_IDLE));
    assert(led.mode == LED_MODE_GDB_RSP && led.manual_lock);

    const bool next_on = p7501_led_state_step_10hz(&led);
    assert(!next_on && led.phase == 1u);

    p7501_led_state_unlock_auto(&led, LED_MODE_HEARTBEAT_IDLE);
    assert(!led.manual_lock && led.mode == LED_MODE_HEARTBEAT_IDLE);
    assert(p7501_led_state_set_auto(&led, LED_MODE_BUS_ACTIVE));
    assert(led.mode == LED_MODE_BUS_ACTIVE);

    pico_led_mode_t parsed_mode = LED_MODE_OFF;
    bool is_auto = false;
    bool is_query = false;
    assert(p7501_led_parse_arg("off", &parsed_mode, &is_auto, &is_query));
    assert(parsed_mode == LED_MODE_OFF && !is_auto && !is_query);
    assert(p7501_led_parse_arg("AUTO", &parsed_mode, &is_auto, &is_query));
    assert(is_auto && !is_query);
    assert(p7501_led_parse_arg("status", &parsed_mode, &is_auto, &is_query));
    assert(is_query);
    assert(!p7501_led_parse_arg("bogus", &parsed_mode, &is_auto, &is_query));
}

int p7501_run_uart_codec_tests(void) {
    test_timeout_and_tokens();
    test_hex_and_baud_math();
    test_trigger_and_waittx_parser();
    test_hex_window_and_led_logic();
    printf("[HOST-TEST] test_uart_codec + LED logic + hex passed all assertions.\n");
    return p7501_run_pana_logic_tests();
}
