/**
 * @file p7501_hex.c
 * @brief Shared pure C11 hexadecimal and ASCII string helpers across CLI, UART, FMU, and GDB RSP.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "common/p7501_hex.h"

static char to_ascii_upper(char c) {
    return (c >= 'a' && c <= 'z') ? (char)(c - ('a' - 'A')) : c;
}

int p7501_hex_char_to_nibble(char c) {
    if (c >= '0' && c <= '9') {
        return c - '0';
    }
    if (c >= 'a' && c <= 'f') {
        return (c - 'a') + 10;
    }
    if (c >= 'A' && c <= 'F') {
        return (c - 'A') + 10;
    }
    return -1;
}

char p7501_nibble_to_hex_char(uint8_t nibble) {
    static const char k_hex[] = "0123456789abcdef";
    return k_hex[nibble & 0x0Fu];
}

bool p7501_ascii_ieq(const char *a, const char *b) {
    if (a == NULL || b == NULL) {
        return false;
    }
    while (*a != '\0' && *b != '\0') {
        if (to_ascii_upper(*a) != to_ascii_upper(*b)) {
            return false;
        }
        ++a;
        ++b;
    }
    return *a == '\0' && *b == '\0';
}

void p7501_ascii_upper_copy(char *dst, size_t dst_cap, const char *src) {
    if (dst == NULL || dst_cap == 0u) {
        return;
    }
    if (src == NULL) {
        dst[0] = '\0';
        return;
    }
    size_t i = 0u;
    while (src[i] != '\0' && (i + 1u) < dst_cap) {
        dst[i] = to_ascii_upper(src[i]);
        ++i;
    }
    dst[i] = '\0';
}

bool p7501_decode_hex_bytes(
    const char *hex,
    uint8_t *out_bytes,
    size_t max_bytes,
    size_t *out_len
) {
    if (hex == NULL || out_bytes == NULL || out_len == NULL || max_bytes == 0u) {
        return false;
    }

    size_t count = 0u;
    int pending_hi = -1;

    for (const char *p = hex; *p != '\0'; ++p) {
        const char ch = *p;
        if (ch == ' ' || ch == '\t' || ch == ',' || ch == ':' || ch == '-' || ch == '_') {
            if (pending_hi >= 0) {
                return false;
            }
            continue;
        }
        const int nib = p7501_hex_char_to_nibble(ch);
        if (nib < 0) {
            return false;
        }
        if (pending_hi < 0) {
            pending_hi = nib;
        } else {
            if (count >= max_bytes) {
                return false;
            }
            out_bytes[count++] = (uint8_t)((pending_hi << 4) | nib);
            pending_hi = -1;
        }
    }

    if (pending_hi >= 0 || count == 0u) {
        return false;
    }
    *out_len = count;
    return true;
}
