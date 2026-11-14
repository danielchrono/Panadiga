/**
 * @file p7501_pana_rsp.c
 * @brief Active `PANARSP` probe against EH55 MAIN `_dev_SERIAL` RSP stub over PIO1.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_pana_rsp.h"
#include "uart/p7501_uart_pio_engine.h"
#include "common/p7501_hex.h"
#include "pico/stdlib.h"

#include <inttypes.h>
#include <stdio.h>
#include <string.h>

#define PANARSP_RX_RAM_CAP 1536u

typedef struct panarsp_rx_cap {
    uint8_t buf[PANARSP_RX_RAM_CAP];
    size_t len;
    bool overflow;
} panarsp_rx_cap_t;

static void panarsp_push_rx(panarsp_rx_cap_t *cap, uint8_t b) {
    if (cap->len < PANARSP_RX_RAM_CAP) {
        cap->buf[cap->len++] = b;
    } else {
        cap->overflow = true;
    }
}

static bool panarsp_drain_for_ms(panarsp_rx_cap_t *cap, uint32_t wait_ms, bool stop_on_rsp_frame) {
    const uint64_t deadline_us = time_us_64() + ((uint64_t)wait_ms * 1000ULL);
    uint64_t frame_grace_us = 0ULL;

    while (time_us_64() < deadline_us) {
        p7501_pio_poll_sw_uart_release();
        uint8_t b = 0u;
        bool pushed = false;
        while (p7501_pio_pop_rx_byte(&b)) {
            panarsp_push_rx(cap, b);
            pushed = true;
        }
        if (stop_on_rsp_frame && pushed && frame_grace_us == 0ULL) {
            p7501_rsp_frame_info_t info;
            if (p7501_rsp_inspect_frame(cap->buf, cap->len, &info)) {
                frame_grace_us = time_us_64() + 40000ULL;
            }
        }
        if (frame_grace_us != 0ULL && time_us_64() >= frame_grace_us) {
            return true;
        }
        tight_loop_contents();
    }
    return false;
}

static uint32_t panarsp_tx_frame(const char *payload) {
    char frame[P7501_RSP_MAX_FRAME];
    const size_t flen = p7501_rsp_build_frame(payload, frame, sizeof(frame));
    if (flen > 0u) {
        p7501_pio_tx_bytes((const uint8_t *)frame, flen, false);
    }
    return (uint32_t)flen;
}

static void panarsp_print_report(const p7501_pana_rsp_spec_t *spec, const panarsp_rx_cap_t *cap, bool triggered) {
    p7501_rsp_frame_info_t info;
    const bool has_frame = p7501_rsp_inspect_frame(cap->buf, cap->len, &info);

    printf("[PANARSP] RX (%u bytes%s):\n",
           (unsigned)cap->len,
           cap->overflow ? ", TRUNCATED" : "");
    for (size_t off = 0u; off < cap->len; off += 16u) {
        const size_t chunk = (cap->len - off > 16u) ? 16u : (cap->len - off);
        printf("  %04X: ", (unsigned)off);
        for (size_t i = 0u; i < 16u; ++i) {
            if (i < chunk) {
                printf("%02X ", cap->buf[off + i]);
            } else {
                printf("   ");
            }
        }
        printf(" |");
        for (size_t i = 0u; i < chunk; ++i) {
            const uint8_t ch = cap->buf[off + i];
            putchar((ch >= 0x20u && ch <= 0x7Eu) ? (int)ch : '.');
        }
        printf("|\n");
    }

    if (has_frame) {
        char ptext[96];
        const size_t copy_len = (info.payload_len < (sizeof(ptext) - 1u))
                                    ? info.payload_len
                                    : (sizeof(ptext) - 1u);
        memcpy(ptext, &cap->buf[info.payload_offset], copy_len);
        ptext[copy_len] = '\0';
        printf("[PANARSP] Frame: ack=%s nak=%s nak0x15=%s | payload=\"%s\" | csum=0x%02X/0x%02X (%s)\n",
               info.ack_plus ? "+" : "-",
               info.nak_minus ? "-" : "no",
               info.raw_nak_0x15 ? "yes" : "no",
               ptext,
               info.actual_csum,
               info.expected_csum,
               info.checksum_ok ? "VALID" : "MISMATCH");
    } else {
        printf("[PANARSP] Frame: none (ack=%s nak=%s nak0x15=%s)\n",
               info.ack_plus ? "+" : "no",
               info.nak_minus ? "-" : "no",
               info.raw_nak_0x15 ? "yes" : "no");
    }

    if (spec->wait_trigger_enabled && !triggered) {
        printf("PANARSP: OK (triggered=NO, rx=%u bytes, rsp=%s)\n",
               (unsigned)cap->len,
               has_frame ? (info.checksum_ok ? "VALID" : "BAD_CSUM") : "NO_REPLY");
        return;
    }
    printf("PANARSP: OK (rx=%u bytes, rsp=%s)\n",
           (unsigned)cap->len,
           has_frame ? (info.checksum_ok ? "VALID" : "BAD_CSUM") : "NO_REPLY");
}

void p7501_cmd_pana_rsp(const char *arg) {
    static panarsp_rx_cap_t s_cap;
    p7501_pana_rsp_spec_t spec;

    if (!p7501_panarsp_parse_args(arg, &spec)) {
        printf("PANARSP: ERROR (Usage: PANARSP [ms] [NORMAL|JIG[:ms]|NOPULL] [MAGIC] [NONAK] [WAIT:<trig>] [PROBESEQ|G|Z0|Z1|MEM:<addr>,<len>|NAK|RAW:<txt>|HEX:<hex>])\n");
        return;
    }

    memset(&s_cap, 0, sizeof(s_cap));
    p7501_pio_enforce_nsw5v_hiz();
    printf("=== PANARSP PIO1 (baud=%" PRIu32 " ms=%" PRIu32 " boot=%d hold=%" PRIu32 " action=%d) ===\n",
           p7501_pio_get_baud(),
           spec.timeout_ms,
           (int)spec.boot_action,
           spec.jig_hold_ms,
           (int)spec.action);

    p7501_pio_begin_rx_session_ex(spec.boot_action, spec.jig_hold_ms);

    bool triggered = !spec.wait_trigger_enabled;
    if (spec.wait_trigger_enabled) {
        uint8_t ring[P7501_RSP_MAX_TRIG + 1u];
        size_t ring_len = 0u;
        const uint64_t wait_deadline = time_us_64() + ((uint64_t)spec.timeout_ms * 1000ULL);
        while (!triggered && time_us_64() < wait_deadline) {
            p7501_pio_poll_sw_uart_release();
            uint8_t b = 0u;
            while (p7501_pio_pop_rx_byte(&b)) {
                panarsp_push_rx(&s_cap, b);
                if (p7501_rolling_match_push(ring, &ring_len, sizeof(ring), b, spec.trigger)) {
                    triggered = true;
                    break;
                }
            }
            tight_loop_contents();
        }
    }

    if (triggered) {
        if (spec.send_magic_prefix || spec.action == PANA_RSP_ACT_MAGIC) {
            static const uint8_t k_magic[3] = { 0x0Eu, 0x04u, 0x02u };
            p7501_pio_tx_bytes(k_magic, sizeof(k_magic), false);
            panarsp_drain_for_ms(&s_cap, 30u, false);
        }
        if (spec.send_nak_prefix && spec.action != PANA_RSP_ACT_MAGIC) {
            static const uint8_t k_nak = 0x15u;
            p7501_pio_tx_bytes(&k_nak, 1u, false);
            panarsp_drain_for_ms(&s_cap, (spec.action == PANA_RSP_ACT_NAK) ? 350u : 80u, true);
        }
        switch (spec.action) {
            case PANA_RSP_ACT_SEQ:
                (void)panarsp_tx_frame("g");
                panarsp_drain_for_ms(&s_cap, 300u, true);
                (void)panarsp_tx_frame("Z0");
                panarsp_drain_for_ms(&s_cap, 250u, true);
                break;
            case PANA_RSP_ACT_G:
            case PANA_RSP_ACT_Z0:
            case PANA_RSP_ACT_Z1:
            case PANA_RSP_ACT_MEM:
            case PANA_RSP_ACT_RAW:
                (void)panarsp_tx_frame(spec.payload);
                panarsp_drain_for_ms(&s_cap, (spec.timeout_ms < 700u) ? spec.timeout_ms : 700u, true);
                break;
            case PANA_RSP_ACT_HEX:
                p7501_pio_tx_bytes(spec.raw_bytes, spec.raw_len, false);
                panarsp_drain_for_ms(&s_cap, 500u, true);
                break;
            case PANA_RSP_ACT_NAK:
            case PANA_RSP_ACT_MAGIC:
            default:
                break;
        }
    }

    p7501_pio_end_session();
    panarsp_print_report(&spec, &s_cap, triggered);
}
