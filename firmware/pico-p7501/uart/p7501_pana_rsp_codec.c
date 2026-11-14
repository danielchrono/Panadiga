/**
 * @file p7501_pana_rsp_codec.c
 * @brief Pure ISO C11 RSP client packet builder, verifier, and `PANARSP` CLI parser.
 *
 * Compiled identically on Pico ARM/RISC-V and host C11 test targets (`PICO_HOST_TEST`).
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_pana_rsp.h"
#include "common/p7501_hex.h"

#include <stdio.h>
#include <string.h>

static bool ascii_iprefix(const char *str, const char *prefix) {
    if (str == NULL || prefix == NULL) {
        return false;
    }
    while (*prefix != '\0') {
        char a = *str++;
        char b = *prefix++;
        if (a >= 'a' && a <= 'z') a = (char)(a - ('a' - 'A'));
        if (b >= 'a' && b <= 'z') b = (char)(b - ('a' - 'A'));
        if (a != b) {
            return false;
        }
    }
    return true;
}

static bool is_digits_only(const char *tok) {
    if (tok == NULL || *tok == '\0') {
        return false;
    }
    for (const char *p = tok; *p != '\0'; ++p) {
        if (*p < '0' || *p > '9') {
            return false;
        }
    }
    return true;
}

static bool parse_mem_spec(const char *spec_text, char *out_payload, size_t cap) {
    const char *comma = strchr(spec_text, ',');
    if (comma == NULL || comma == spec_text || comma[1] == '\0') {
        return false;
    }
    const char *addr_start = spec_text;
    if (addr_start[0] == '0' && (addr_start[1] == 'x' || addr_start[1] == 'X')) {
        addr_start += 2;
    }
    size_t addr_len = (size_t)(comma - addr_start);
    if (addr_len == 0u || addr_len > 8u) {
        return false;
    }
    for (size_t i = 0u; i < addr_len; ++i) {
        if (p7501_hex_char_to_nibble(addr_start[i]) < 0) {
            return false;
        }
    }

    const char *len_start = comma + 1;
    if (len_start[0] == '0' && (len_start[1] == 'x' || len_start[1] == 'X')) {
        len_start += 2;
    }
    size_t len_chars = strlen(len_start);
    if (len_chars == 0u || len_chars > 4u) {
        return false;
    }
    for (size_t i = 0u; i < len_chars; ++i) {
        if (p7501_hex_char_to_nibble(len_start[i]) < 0) {
            return false;
        }
    }

    char addr_buf[12];
    memcpy(addr_buf, addr_start, addr_len);
    addr_buf[addr_len] = '\0';
    int n = snprintf(out_payload, cap, "m%s,%s", addr_buf, len_start);
    return n > 0 && (size_t)n < cap;
}

size_t p7501_rsp_build_frame(const char *payload, char *out, size_t out_cap) {
    if (payload == NULL || out == NULL) {
        return 0u;
    }
    const size_t plen = strlen(payload);
    if ((plen + 5u) > out_cap) {
        return 0u;
    }
    const uint8_t csum = p7501_rsp_checksum_str(payload);
    out[0] = '$';
    if (plen > 0u) {
        memcpy(&out[1], payload, plen);
    }
    out[1u + plen] = '#';
    out[2u + plen] = p7501_nibble_to_hex_char((uint8_t)(csum >> 4));
    out[3u + plen] = p7501_nibble_to_hex_char((uint8_t)(csum & 0x0Fu));
    out[4u + plen] = '\0';
    return 4u + plen;
}

bool p7501_rsp_inspect_frame(const uint8_t *buf, size_t len, p7501_rsp_frame_info_t *out) {
    if (buf == NULL || out == NULL) {
        return false;
    }
    memset(out, 0, sizeof(*out));
    for (size_t i = 0u; i < len; ++i) {
        if (buf[i] == 0x15u) {
            out->raw_nak_0x15 = true;
        } else if (buf[i] == (uint8_t)'+') {
            out->ack_plus = true;
        } else if (buf[i] == (uint8_t)'-') {
            out->nak_minus = true;
        }
        if (buf[i] != (uint8_t)'$') {
            continue;
        }
        const size_t start = i + 1u;
        for (size_t j = start; (j + 2u) < len; ++j) {
            if (buf[j] != (uint8_t)'#') {
                continue;
            }
            const int hi = p7501_hex_char_to_nibble((char)buf[j + 1u]);
            const int lo = p7501_hex_char_to_nibble((char)buf[j + 2u]);
            if (hi < 0 || lo < 0) {
                continue;
            }
            out->found = true;
            out->payload_offset = start;
            out->payload_len = j - start;
            out->expected_csum = p7501_rsp_checksum(&buf[start], out->payload_len);
            out->actual_csum = (uint8_t)((hi << 4) | lo);
            out->checksum_ok = (out->expected_csum == out->actual_csum);
            return true;
        }
    }
    return false;
}

static bool apply_panarsp_action_token(const char *tok, p7501_pana_rsp_spec_t *out, bool *saw_action) {
    if (p7501_ascii_ieq(tok, "PROBESEQ") || p7501_ascii_ieq(tok, "PROBE") ||
        p7501_ascii_ieq(tok, "PING_STUB")) {
        out->action = PANA_RSP_ACT_SEQ;
        *saw_action = true;
        return true;
    }
    if (p7501_ascii_ieq(tok, "G")) {
        out->action = PANA_RSP_ACT_G;
        snprintf(out->payload, sizeof(out->payload), "g");
        *saw_action = true;
        return true;
    }
    if (p7501_ascii_ieq(tok, "Z0")) {
        out->action = PANA_RSP_ACT_Z0;
        snprintf(out->payload, sizeof(out->payload), "Z0");
        *saw_action = true;
        return true;
    }
    if (p7501_ascii_ieq(tok, "Z1")) {
        out->action = PANA_RSP_ACT_Z1;
        snprintf(out->payload, sizeof(out->payload), "Z1");
        *saw_action = true;
        return true;
    }
    if (ascii_iprefix(tok, "MEM:") && parse_mem_spec(tok + 4, out->payload, sizeof(out->payload))) {
        out->action = PANA_RSP_ACT_MEM;
        *saw_action = true;
        return true;
    }
    if (ascii_iprefix(tok, "RAW:") && tok[4] != '\0') {
        out->action = PANA_RSP_ACT_RAW;
        snprintf(out->payload, sizeof(out->payload), "%s", tok + 4);
        *saw_action = true;
        return true;
    }
    if (ascii_iprefix(tok, "HEX:") && tok[4] != '\0') {
        if (!p7501_decode_hex_bytes(tok + 4, out->raw_bytes, sizeof(out->raw_bytes), &out->raw_len)) {
            return false;
        }
        out->action = PANA_RSP_ACT_HEX;
        *saw_action = true;
        return true;
    }
    return false;
}

bool p7501_panarsp_parse_args(const char *arg, p7501_pana_rsp_spec_t *out) {
    if (out == NULL) {
        return false;
    }
    memset(out, 0, sizeof(*out));
    out->timeout_ms = 4000u;
    out->boot_action = P7501_BOOT_ACTION_PASSIVE;
    out->send_nak_prefix = true;
    out->action = PANA_RSP_ACT_SEQ;

    if (arg == NULL || arg[0] == '\0') {
        return true;
    }

    const char *cur = arg;
    char tok[144];
    bool saw_boot = false;
    bool saw_action = false;
    bool explicit_nak_only = false;
    bool explicit_magic_only = false;

    while ((cur = p7501_next_token(cur, tok, sizeof(tok))) != NULL) {
        if (tok[0] == '\0') {
            continue;
        }
        if (is_digits_only(tok)) {
            out->timeout_ms = p7501_parse_uint32_clamped(tok, 4000u, 150u, 30000u);
        } else if (p7501_ascii_ieq(tok, "MAGIC")) {
            out->send_magic_prefix = true;
            explicit_magic_only = true;
        } else if (p7501_ascii_ieq(tok, "NONAK")) {
            out->send_nak_prefix = false;
        } else if (p7501_ascii_ieq(tok, "NAK")) {
            out->send_nak_prefix = true;
            explicit_nak_only = true;
        } else if (ascii_iprefix(tok, "WAIT:") && tok[5] != '\0') {
            out->wait_trigger_enabled = true;
            snprintf(out->trigger, sizeof(out->trigger), "%s", tok + 5);
        } else if (!saw_boot && (p7501_ascii_ieq(tok, "NORMAL") || p7501_ascii_ieq(tok, "BOOT") ||
                                 p7501_ascii_ieq(tok, "NOPULL") || p7501_ascii_ieq(tok, "PASSIVE") ||
                                 p7501_ascii_ieq(tok, "JIG") || ascii_iprefix(tok, "JIG:"))) {
            out->boot_action = p7501_parse_boot_spec(tok, &out->jig_hold_ms);
            saw_boot = true;
        } else if (!apply_panarsp_action_token(tok, out, &saw_action)) {
            return false;
        }
    }

    if (!saw_action) {
        if (explicit_nak_only) {
            out->action = PANA_RSP_ACT_NAK;
        } else if (explicit_magic_only && !out->send_nak_prefix) {
            out->action = PANA_RSP_ACT_MAGIC;
        }
    }
    return true;
}
