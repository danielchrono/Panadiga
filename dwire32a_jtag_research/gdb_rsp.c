/**
 * @file gdb_rsp.c
 * @brief Implementation of GDB Remote Serial Protocol (RSP) Server for Panasonic AM34 / MN103S.
 *
 * Implements bidirectional GDB remote protocol framing, register caching,
 * zero-copy memory windowing, and DWire32A hardware orchestration.
 *
 * Target: Panasonic MN2DS0016 (Matsushita AM34 / MN103S core)
 * PCB:    Panasonic DMR-EH55 Main Digital Board VEP79138
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "gdb_rsp.h"
#include "dwire32a_hw.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

/* =========================================================================
 * Internal State Machine & Caching Structures
 * ========================================================================= */

typedef enum gdb_parser_state {
    GDB_STATE_IDLE,         /**< Waiting for '$', '+', '-', or Ctrl-C */
    GDB_STATE_DATA,         /**< Collecting packet payload until '#' */
    GDB_STATE_CHKSUM_MSB,   /**< Reading high hex digit of checksum */
    GDB_STATE_CHKSUM_LSB    /**< Reading low hex digit of checksum */
} gdb_parser_state_t;

static bool               s_gdb_active = false;
static gdb_parser_state_t s_parser_state = GDB_STATE_IDLE;
static char               s_rx_buffer[GDB_RSP_MAX_PACKET_LEN];
static size_t             s_rx_len = 0U;
static uint8_t            s_calc_checksum = 0U;
static uint8_t            s_received_checksum = 0U;
static char               s_last_tx_packet[GDB_RSP_MAX_PACKET_LEN];

/* Shadow register file */
static am34_reg_file_t    s_reg_file;

/* Sector memory cache */
static uint8_t            s_sector_cache[GDB_SECTOR_CACHE_SIZE];
static uint32_t           s_cache_base_addr = 0xFFFFFFFFUL;
static bool               s_cache_valid = false;

/* XML Memory Map representation for GDB qXfer:memory-map */
static const char * const GDB_XML_MEMORY_MAP =
    "l<memory-map>"
    "<memory type=\"ram\" start=\"0x00000000\" length=\"0x00040000\"/>"
    "<memory type=\"rom\" start=\"0x80000000\" length=\"0x00800000\"/>"
    "<memory type=\"flash\" start=\"0x807F0000\" length=\"0x00010000\">"
    "<property name=\"blocksize\">0x2000</property>"
    "</memory>"
    "</memory-map>";

/* =========================================================================
 * Helper Utility Functions
 * ========================================================================= */

static int hex_char_to_nibble(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}

static char nibble_to_hex_char(uint8_t n) {
    static const char hex_digits[] = "0123456789abcdef";
    return hex_digits[n & 0x0FU];
}

static uint8_t compute_checksum(const char *payload) {
    uint8_t sum = 0U;
    while (*payload != '\0') {
        sum = (uint8_t)(sum + (uint8_t)(*payload));
        payload++;
    }
    return sum;
}

/**
 * @brief Parse a hexadecimal address/length from string up to delimiter.
 */
static const char *parse_hex_u32(const char *str, uint32_t *out_val, char delim) {
    uint32_t val = 0U;
    if (str == NULL || out_val == NULL) return NULL;

    while (*str != '\0' && *str != delim) {
        int nibble = hex_char_to_nibble(*str);
        if (nibble < 0) return NULL;
        val = (val << 4) | (uint32_t)nibble;
        str++;
    }
    *out_val = val;
    if (*str == delim) {
        str++; // Advance past delimiter
    }
    return str;
}

/* =========================================================================
 * Memory Subsystem & Caching Engine
 * ========================================================================= */

void gdb_rsp_invalidate_cache(void) {
    s_cache_valid = false;
    s_cache_base_addr = 0xFFFFFFFFUL;
}

/**
 * @brief Populate the sector cache from target flash or SRAM.
 */
static void populate_cache(uint32_t aligned_addr) {
    s_cache_base_addr = aligned_addr;
    const uint32_t words_to_read = GDB_SECTOR_CACHE_SIZE / 4U;

    for (uint32_t i = 0U; i < words_to_read; i++) {
        uint32_t word = dwire_hw_read_mem32(aligned_addr + (i * 4U));
        // Store little-endian
        s_sector_cache[(i * 4U) + 0U] = (uint8_t)(word & 0xFFU);
        s_sector_cache[(i * 4U) + 1U] = (uint8_t)((word >> 8) & 0xFFU);
        s_sector_cache[(i * 4U) + 2U] = (uint8_t)((word >> 16) & 0xFFU);
        s_sector_cache[(i * 4U) + 3U] = (uint8_t)((word >> 24) & 0xFFU);
    }
    s_cache_valid = true;
}

/**
 * @brief Read single byte from target memory or local cache.
 */
static uint8_t memory_read_byte(uint32_t addr) {
    // Check if within cache
    if (s_cache_valid && addr >= s_cache_base_addr && addr < (s_cache_base_addr + GDB_SECTOR_CACHE_SIZE)) {
        return s_sector_cache[addr - s_cache_base_addr];
    }

    // Flash or NVRAM range: populate sector cache for burst speed
    if ((addr >= 0x80000000UL && addr < 0x80800000UL) ||
        (addr >= 0x007F0000UL && addr < 0x00800000UL)) {
        uint32_t aligned = addr & ~(GDB_SECTOR_CACHE_SIZE - 1U);
        populate_cache(aligned);
        return s_sector_cache[addr - aligned];
    }

    // Direct word read
    uint32_t aligned_addr = addr & ~0x03UL;
    uint32_t word = dwire_hw_read_mem32(aligned_addr);
    uint32_t byte_idx = addr & 0x03UL;
    return (uint8_t)((word >> (byte_idx * 8U)) & 0xFFU);
}

/**
 * @brief Write single byte to target memory.
 */
static void memory_write_byte(uint32_t addr, uint8_t byte_val) {
    uint32_t aligned_addr = addr & ~0x03UL;
    uint32_t byte_idx = addr & 0x03UL;
    uint32_t current_word = dwire_hw_read_mem32(aligned_addr);

    uint32_t mask = 0xFFUL << (byte_idx * 8U);
    uint32_t new_word = (current_word & ~mask) | (((uint32_t)byte_val) << (byte_idx * 8U));

    dwire_hw_write_mem32(aligned_addr, new_word);

    // Invalidate or update cache
    if (s_cache_valid && addr >= s_cache_base_addr && addr < (s_cache_base_addr + GDB_SECTOR_CACHE_SIZE)) {
        s_sector_cache[addr - s_cache_base_addr] = byte_val;
    }
}

/* =========================================================================
 * Register Shadow & CPU Control
 * ========================================================================= */

static void sync_shadow_registers(void) {
    s_reg_file.is_halted = dwire_hw_is_cpu_halted();
    // Default boot values if not read from hardware registers
    if (s_reg_file.pc == 0U) {
        s_reg_file.pc = 0x80000000UL;
    }
    if (s_reg_file.sp == 0U) {
        s_reg_file.sp = 0x0003FFF0UL;
    }
}

/* =========================================================================
 * GDB RSP Protocol Handler
 * ========================================================================= */

void gdb_rsp_init(void) {
    s_gdb_active = false;
    s_parser_state = GDB_STATE_IDLE;
    s_rx_len = 0U;
    s_calc_checksum = 0U;
    s_received_checksum = 0U;
    memset(s_last_tx_packet, 0, sizeof(s_last_tx_packet));
    memset(&s_reg_file, 0, sizeof(s_reg_file));
    s_reg_file.pc = 0x80000000UL;
    s_reg_file.sp = 0x0003FFF0UL;
    gdb_rsp_invalidate_cache();
}

bool gdb_rsp_is_active(void) {
    return s_gdb_active;
}

void gdb_rsp_set_active(bool active) {
    s_gdb_active = active;
    if (active) {
        s_parser_state = GDB_STATE_IDLE;
        s_rx_len = 0U;
        gdb_rsp_invalidate_cache();
        sync_shadow_registers();
    }
}

const char *gdb_rsp_get_status_info(void) {
    return GDB_TARGET_CHIP_LABEL;
}

void gdb_rsp_send_packet(const char *payload) {
    if (payload == NULL) return;

    uint8_t csum = compute_checksum(payload);

    // Save for retransmission on NACK
    snprintf(s_last_tx_packet, sizeof(s_last_tx_packet), "%s", payload);

    putchar('$');
    const char *p = payload;
    while (*p != '\0') {
        putchar(*p++);
    }
    putchar('#');
    putchar(nibble_to_hex_char((uint8_t)(csum >> 4)));
    putchar(nibble_to_hex_char((uint8_t)(csum & 0x0FU)));
}

/**
 * @brief Dispatch parsed command packet to handler.
 */
static void handle_gdb_command(const char *cmd) {
    char resp_buf[GDB_RSP_MAX_PACKET_LEN];
    resp_buf[0] = '\0';

    switch (cmd[0]) {
        /* Query Halt / Stop Reason */
        case '?': {
            // Signal 5 = SIGTRAP (target halted at breakpoint or break-in)
            gdb_rsp_send_packet("S05");
            break;
        }

        /* General Capabilities Query */
        case 'q': {
            if (strncmp(cmd, "qSupported", 10) == 0) {
                // Advertise packet size and memory map XML capability
                gdb_rsp_send_packet("PacketSize=800;qXfer:memory-map:read+;qAttached+");
            }
            else if (strncmp(cmd, "qXfer:memory-map:read::", 23) == 0) {
                // Return XML memory map
                gdb_rsp_send_packet(GDB_XML_MEMORY_MAP);
            }
            else if (strcmp(cmd, "qAttached") == 0) {
                gdb_rsp_send_packet("1"); // Attached to existing hardware target
            }
            else if (strcmp(cmd, "qOffsets") == 0) {
                gdb_rsp_send_packet("Text=0;Data=0;Bss=0");
            }
            else {
                // Unsupported query packet
                gdb_rsp_send_packet("");
            }
            break;
        }

        /* Read General Registers (D0..D3, A0..A3, MDR, PSW, PC, SP) */
        case 'g': {
            sync_shadow_registers();
            char *ptr = resp_buf;
            uint32_t *regs = (uint32_t *)&s_reg_file;
            for (size_t i = 0U; i < AM34_REG_COUNT; i++) {
                uint32_t val = regs[i];
                // Output in little-endian byte order
                for (int b = 0; b < 4; b++) {
                    uint8_t byte = (uint8_t)((val >> (b * 8)) & 0xFFU);
                    *ptr++ = nibble_to_hex_char((uint8_t)(byte >> 4));
                    *ptr++ = nibble_to_hex_char((uint8_t)(byte & 0x0FU));
                }
            }
            *ptr = '\0';
            gdb_rsp_send_packet(resp_buf);
            break;
        }

        /* Write General Registers */
        case 'G': {
            const char *hex_str = &cmd[1];
            size_t hex_len = strlen(hex_str);
            if (hex_len >= (AM34_REG_COUNT * 8U)) {
                uint32_t *regs = (uint32_t *)&s_reg_file;
                for (size_t i = 0U; i < AM34_REG_COUNT; i++) {
                    uint32_t word = 0U;
                    for (int b = 0; b < 4; b++) {
                        int h = hex_char_to_nibble(*hex_str++);
                        int l = hex_char_to_nibble(*hex_str++);
                        if (h >= 0 && l >= 0) {
                            uint8_t byte = (uint8_t)((h << 4) | l);
                            word |= ((uint32_t)byte) << (b * 8);
                        }
                    }
                    regs[i] = word;
                }
                dwire_hw_set_pc(s_reg_file.pc);
                gdb_rsp_send_packet("OK");
            } else {
                gdb_rsp_send_packet("E01");
            }
            break;
        }

        /* Read Memory: 'm<addr>,<length>' */
        case 'm': {
            uint32_t addr = 0U;
            uint32_t length = 0U;
            const char *p = parse_hex_u32(&cmd[1], &addr, ',');
            if (p != NULL) {
                parse_hex_u32(p, &length, '\0');
            }

            if (length == 0U || length > 512U) {
                gdb_rsp_send_packet("E02");
                break;
            }

            char *out = resp_buf;
            for (uint32_t i = 0U; i < length; i++) {
                uint8_t b = memory_read_byte(addr + i);
                *out++ = nibble_to_hex_char((uint8_t)(b >> 4));
                *out++ = nibble_to_hex_char((uint8_t)(b & 0x0FU));
            }
            *out = '\0';
            gdb_rsp_send_packet(resp_buf);
            break;
        }

        /* Write Memory: 'M<addr>,<length>:<data>' */
        case 'M': {
            uint32_t addr = 0U;
            uint32_t length = 0U;
            const char *p = parse_hex_u32(&cmd[1], &addr, ',');
            if (p != NULL) {
                p = parse_hex_u32(p, &length, ':');
            }

            if (p == NULL || length == 0U || length > 512U) {
                gdb_rsp_send_packet("E03");
                break;
            }

            for (uint32_t i = 0U; i < length; i++) {
                int h = hex_char_to_nibble(*p++);
                int l = hex_char_to_nibble(*p++);
                if (h >= 0 && l >= 0) {
                    uint8_t byte = (uint8_t)((h << 4) | l);
                    memory_write_byte(addr + i, byte);
                } else {
                    gdb_rsp_send_packet("E04");
                    return;
                }
            }
            gdb_rsp_send_packet("OK");
            break;
        }

        /* Continue Execution: 'c' [addr] */
        case 'c': {
            if (cmd[1] != '\0') {
                uint32_t cont_addr = 0U;
                parse_hex_u32(&cmd[1], &cont_addr, '\0');
                s_reg_file.pc = cont_addr;
                dwire_hw_set_pc(cont_addr);
            }
            dwire_hw_resume_cpu();
            s_reg_file.is_halted = false;
            // Send stop reason when paused
            gdb_rsp_send_packet("S05");
            break;
        }

        /* Single-Step: 's' [addr] */
        case 's': {
            if (cmd[1] != '\0') {
                uint32_t step_addr = 0U;
                parse_hex_u32(&cmd[1], &step_addr, '\0');
                s_reg_file.pc = step_addr;
                dwire_hw_set_pc(step_addr);
            }
            // Step CPU
            s_reg_file.pc += 2U; // Approximate instruction advance
            dwire_hw_set_pc(s_reg_file.pc);
            gdb_rsp_send_packet("S05");
            break;
        }

        /* Kill / Reset: 'k' */
        case 'k': {
            dwire_hw_reset_assert();
            sleep_ms(DWIRE_RESET_ASSERT_MS);
            dwire_hw_reset_release();
            gdb_rsp_invalidate_cache();
            gdb_rsp_send_packet("OK");
            break;
        }

        /* Detach: 'D' */
        case 'D': {
            gdb_rsp_send_packet("OK");
            s_gdb_active = false; // Exit GDB mode and return to terminal CLI
            break;
        }

        /* Reset and Break-in restart: 'R' */
        case 'R': {
            dwire_hw_cold_break_in(DWIRE_BREAKIN_MODE_EXTRG_HIGH_PRE_ONLY);
            gdb_rsp_invalidate_cache();
            s_reg_file.pc = 0x80000000UL;
            gdb_rsp_send_packet("OK");
            break;
        }

        default: {
            // Empty response indicates command is not supported by this stub
            gdb_rsp_send_packet("");
            break;
        }
    }
}

/* =========================================================================
 * Stream Processing Entry Point
 * ========================================================================= */

void gdb_rsp_process_char(char ch) {
    // Check for GDB Interrupt (Ctrl-C / 0x03)
    if (ch == 0x03) {
        dwire_hw_extrg_set(true); // Force OCD Halt via EXTRG
        s_reg_file.is_halted = true;
        gdb_rsp_send_packet("S05"); // Report SIGTRAP
        return;
    }

    switch (s_parser_state) {
        case GDB_STATE_IDLE: {
            if (ch == '$') {
                s_parser_state = GDB_STATE_DATA;
                s_rx_len = 0U;
                s_calc_checksum = 0U;
            } else if (ch == '-') {
                // Retransmit last packet on NACK
                if (s_last_tx_packet[0] != '\0') {
                    gdb_rsp_send_packet(s_last_tx_packet);
                }
            } else if (ch == '+') {
                // ACK received, do nothing
            }
            break;
        }

        case GDB_STATE_DATA: {
            if (ch == '#') {
                s_parser_state = GDB_STATE_CHKSUM_MSB;
            } else {
                if (s_rx_len < (GDB_RSP_MAX_PACKET_LEN - 1U)) {
                    s_rx_buffer[s_rx_len++] = ch;
                    s_calc_checksum = (uint8_t)(s_calc_checksum + (uint8_t)ch);
                }
            }
            break;
        }

        case GDB_STATE_CHKSUM_MSB: {
            int nibble = hex_char_to_nibble(ch);
            if (nibble >= 0) {
                s_received_checksum = (uint8_t)(nibble << 4);
                s_parser_state = GDB_STATE_CHKSUM_LSB;
            } else {
                putchar('-'); // NACK malformed checksum
                s_parser_state = GDB_STATE_IDLE;
            }
            break;
        }

        case GDB_STATE_CHKSUM_LSB: {
            int nibble = hex_char_to_nibble(ch);
            if (nibble >= 0) {
                s_received_checksum = (uint8_t)(s_received_checksum | (uint8_t)nibble);
                if (s_received_checksum == s_calc_checksum) {
                    putchar('+'); // Valid packet ACK
                    s_rx_buffer[s_rx_len] = '\0';
                    handle_gdb_command(s_rx_buffer);
                } else {
                    putchar('-'); // Bad checksum NACK
                }
            } else {
                putchar('-');
            }
            s_parser_state = GDB_STATE_IDLE;
            break;
        }
    }
}
