/**
 * @file gdb_rsp_codec.c
 * @brief GDB RSP packet framing state machine, hex codecs, checksums, and host test hooks.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "gdb/gdb_rsp_internal.h"
#include "hal/dwire32a_hw.h"
#include <stdio.h>
#include <string.h>

typedef enum gdb_parser_state {
    GDB_STATE_IDLE,
    GDB_STATE_DATA,
    GDB_STATE_CHKSUM_MSB,
    GDB_STATE_CHKSUM_LSB
} gdb_parser_state_t;

static bool               s_gdb_active = false;
static gdb_parser_state_t s_parser_state = GDB_STATE_IDLE;
static char               s_rx_buffer[GDB_RSP_MAX_PACKET_LEN];
static size_t             s_rx_len = 0U;
static uint8_t            s_calc_checksum = 0U;
static uint8_t            s_received_checksum = 0U;
static char               s_last_tx_packet[GDB_RSP_MAX_PACKET_LEN];

#if defined(PICO_HOST_TEST) && (PICO_HOST_TEST != 0)
static char s_test_last_resp[GDB_RSP_MAX_PACKET_LEN];

static void gdb_rsp_test_record_response(const char *payload) {
    if (payload != NULL) {
        snprintf(s_test_last_resp, sizeof(s_test_last_resp), "%s", payload);
    }
}

void gdb_rsp_test_get_last_response(char *out, size_t max_len) {
    if ((out != NULL) && (max_len > 0U)) {
        snprintf(out, max_len, "%s", s_test_last_resp);
    }
}

void gdb_rsp_test_inject_packet(const char *payload) {
    if (payload == NULL) {
        return;
    }
    gdb_rsp_process_char('$');
    const char *p = payload;
    while (*p != '\0') {
        gdb_rsp_process_char(*p++);
    }
    gdb_rsp_process_char('#');
    const uint8_t csum = gdb_compute_checksum(payload);
    gdb_rsp_process_char(gdb_nibble_to_hex_char((uint8_t)(csum >> 4)));
    gdb_rsp_process_char(gdb_nibble_to_hex_char((uint8_t)(csum & 0x0FU)));
}
#endif

int gdb_hex_char_to_nibble(char c) {
    if ((c >= '0') && (c <= '9')) { return c - '0'; }
    if ((c >= 'a') && (c <= 'f')) { return c - 'a' + 10; }
    if ((c >= 'A') && (c <= 'F')) { return c - 'A' + 10; }
    return -1;
}

char gdb_nibble_to_hex_char(uint8_t n) {
    static const char hex_digits[] = "0123456789abcdef";
    return hex_digits[n & 0x0FU];
}

uint8_t gdb_compute_checksum(const char *payload) {
    uint8_t sum = 0U;
    while (*payload != '\0') {
        sum = (uint8_t)(sum + (uint8_t)(*payload));
        payload++;
    }
    return sum;
}

const char *gdb_parse_hex_u32(const char *str, uint32_t *out_val, char delim) {
    uint32_t val = 0U;
    size_t digits = 0U;
    if ((str == NULL) || (out_val == NULL)) {
        return NULL;
    }

    while ((*str != '\0') && (*str != delim)) {
        const int nibble = gdb_hex_char_to_nibble(*str);
        if (nibble < 0) {
            return NULL;
        }
        val = (val << 4) | (uint32_t)nibble;
        str++;
        digits++;
    }
    if ((digits == 0U) || (*str != delim)) {
        return NULL;
    }
    *out_val = val;
    if ((*str == delim) && (delim != '\0')) {
        str++;
    }
    return str;
}

const char *gdb_parse_addr_and_len(
    const char *str,
    uint32_t *out_addr,
    uint32_t *out_len,
    char end_delim
) {
    const char *p = gdb_parse_hex_u32(str, out_addr, ',');
    if (p == NULL) {
        return NULL;
    }
    p = gdb_parse_hex_u32(p, out_len, end_delim);
    if ((p == NULL) || (*out_len == 0U) || (*out_len > 512U)) {
        return NULL;
    }
    return p;
}

void gdb_rsp_init(void) {
    s_gdb_active = false;
    s_parser_state = GDB_STATE_IDLE;
    s_rx_len = 0U;
    s_calc_checksum = 0U;
    s_received_checksum = 0U;
    memset(s_last_tx_packet, 0, sizeof(s_last_tx_packet));
    gdb_cpu_init_registers();
    gdb_rsp_invalidate_cache();
}

bool gdb_rsp_is_active(void) {
    return s_gdb_active;
}

void gdb_rsp_set_active_flag(bool active) {
    s_gdb_active = active;
}

void gdb_rsp_set_active(bool active) {
    s_gdb_active = active;
    if (active) {
        s_parser_state = GDB_STATE_IDLE;
        s_rx_len = 0U;
        gdb_rsp_invalidate_cache();
        gdb_cpu_sync_registers();
    }
}

const char *gdb_rsp_get_status_info(void) {
    return GDB_TARGET_CHIP_LABEL;
}

void gdb_rsp_send_packet(const char *payload) {
    if (payload == NULL) {
        return;
    }

#if defined(PICO_HOST_TEST) && (PICO_HOST_TEST != 0)
    gdb_rsp_test_record_response(payload);
#endif

    const uint8_t csum = gdb_compute_checksum(payload);
    snprintf(s_last_tx_packet, sizeof(s_last_tx_packet), "%s", payload);

    putchar('$');
    const char *p = payload;
    while (*p != '\0') {
        putchar(*p++);
    }
    putchar('#');
    putchar(gdb_nibble_to_hex_char((uint8_t)(csum >> 4)));
    putchar(gdb_nibble_to_hex_char((uint8_t)(csum & 0x0FU)));
}

void gdb_rsp_process_char(char ch) {
    if (ch == 0x03) {
        gdb_cpu_handle_ctrl_c();
        return;
    }

    switch (s_parser_state) {
        case GDB_STATE_IDLE:
            if (ch == '$') {
                s_parser_state = GDB_STATE_DATA;
                s_rx_len = 0U;
                s_calc_checksum = 0U;
            } else if (ch == '-') {
                if (s_last_tx_packet[0] != '\0') {
                    gdb_rsp_send_packet(s_last_tx_packet);
                }
            } else if (ch == '+') {
                /* ACK received */
            }
            break;

        case GDB_STATE_DATA:
            if (ch == '#') {
                s_parser_state = GDB_STATE_CHKSUM_MSB;
            } else if (s_rx_len < (GDB_RSP_MAX_PACKET_LEN - 1U)) {
                s_rx_buffer[s_rx_len++] = ch;
                s_calc_checksum = (uint8_t)(s_calc_checksum + (uint8_t)ch);
            }
            break;

        case GDB_STATE_CHKSUM_MSB: {
            const int nibble = gdb_hex_char_to_nibble(ch);
            if (nibble >= 0) {
                s_received_checksum = (uint8_t)(nibble << 4);
                s_parser_state = GDB_STATE_CHKSUM_LSB;
            } else {
                putchar('-');
                s_parser_state = GDB_STATE_IDLE;
            }
            break;
        }

        case GDB_STATE_CHKSUM_LSB: {
            const int nibble = gdb_hex_char_to_nibble(ch);
            if (nibble >= 0) {
                s_received_checksum = (uint8_t)(s_received_checksum | (uint8_t)nibble);
                if (s_received_checksum == s_calc_checksum) {
                    putchar('+');
                    s_rx_buffer[s_rx_len] = '\0';
                    gdb_cpu_dispatch_command(s_rx_buffer);
                } else {
                    putchar('-');
                }
            } else {
                putchar('-');
            }
            s_parser_state = GDB_STATE_IDLE;
            break;
        }
    }
}
