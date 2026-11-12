/**
 * @file gdb_rsp_memory.c
 * @brief GDB RSP target memory aperture, sector caching, XML map, and IPL air-gap guard.
 *
 * Enforces OPUS-07 read-only isolation over protected Panasonic Bootloader IPL
 * ranges (`0x80000000 .. nvram_start - 1`), rejecting illegal writes with `E05`.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "gdb/gdb_rsp_internal.h"
#include "hal/dwire32a_hw.h"
#include <stdio.h>
#include <string.h>

static uint8_t  s_sector_cache[GDB_SECTOR_CACHE_SIZE];
static uint32_t s_cache_base_addr = 0xFFFFFFFFUL;
static bool     s_cache_valid = false;
static uint32_t s_flash_size_bytes = 0x00800000UL;

bool gdb_rsp_set_flash_size(uint32_t flash_size_bytes) {
    if ((flash_size_bytes != 0x00200000UL) &&
        (flash_size_bytes != 0x00400000UL) &&
        (flash_size_bytes != 0x00800000UL) &&
        (flash_size_bytes != 0x01000000UL)) {
        return false;
    }
    s_flash_size_bytes = flash_size_bytes;
    gdb_rsp_invalidate_cache();
    return true;
}

uint32_t gdb_rsp_get_flash_size(void) {
    return s_flash_size_bytes;
}

void gdb_rsp_invalidate_cache(void) {
    s_cache_valid = false;
    s_cache_base_addr = 0xFFFFFFFFUL;
}

static void populate_cache(uint32_t aligned_addr) {
    s_cache_base_addr = aligned_addr;
    const uint32_t words_to_read = GDB_SECTOR_CACHE_SIZE / 4U;

    for (uint32_t i = 0U; i < words_to_read; i++) {
        const uint32_t word = dwire_hw_read_mem32(aligned_addr + (i * 4U));
        s_sector_cache[(i * 4U) + 0U] = (uint8_t)(word & 0xFFU);
        s_sector_cache[(i * 4U) + 1U] = (uint8_t)((word >> 8) & 0xFFU);
        s_sector_cache[(i * 4U) + 2U] = (uint8_t)((word >> 16) & 0xFFU);
        s_sector_cache[(i * 4U) + 3U] = (uint8_t)((word >> 24) & 0xFFU);
    }
    s_cache_valid = true;
}

static uint8_t memory_read_byte(uint32_t addr) {
    if (s_cache_valid && (addr >= s_cache_base_addr) && (addr < (s_cache_base_addr + GDB_SECTOR_CACHE_SIZE))) {
        return s_sector_cache[addr - s_cache_base_addr];
    }

    if ((addr >= 0x80000000UL) && (addr < (0x80000000UL + s_flash_size_bytes))) {
        const uint32_t aligned = addr & ~(GDB_SECTOR_CACHE_SIZE - 1U);
        populate_cache(aligned);
        return s_sector_cache[addr - aligned];
    }

    const uint32_t aligned_addr = addr & ~0x03UL;
    const uint32_t word = dwire_hw_read_mem32(aligned_addr);
    const uint32_t byte_idx = addr & 0x03UL;
    return (uint8_t)((word >> (byte_idx * 8U)) & 0xFFU);
}

static bool is_valid_write_range(uint32_t addr, uint32_t length) {
    if (length == 0U) {
        return false;
    }
    const uint32_t end_addr = addr + (length - 1U);
    if (end_addr < addr) {
        return false;
    }

    const uint32_t nvram_start = 0x80000000UL + (s_flash_size_bytes - 0x00010000UL);
    const uint32_t ipl_end     = nvram_start - 1UL;
    const uint32_t flash_end   = 0x80000000UL + (s_flash_size_bytes - 1UL);

    /* OPUS-07: Strictly reject any overlap with protected Bootloader IPL ROM */
    if ((addr <= ipl_end) && (end_addr >= 0x80000000UL)) {
        return false;
    }

    if (end_addr <= 0x0003FFFFUL) { return true; }                        /* Internal SRAM */
    if ((addr >= 0x10000000UL) && (end_addr <= 0x1000FFFFUL)) { return true; } /* Core MMIO */
    if ((addr >= 0x40000000UL) && (end_addr <= 0x43FFFFFFUL)) { return true; } /* System SDRAM */
    if ((addr >= nvram_start)  && (end_addr <= flash_end))    { return true; } /* Top 64KB NVRAM */
    if ((addr >= 0xC0000000UL) && (end_addr <= 0xC000FFFFUL)) { return true; } /* OCD MMIO */
    if ((addr >= 0xD4000000UL) && (end_addr <= 0xD4000FFFUL)) { return true; } /* PATA / IDE MMIO */

    return false;
}

static bool memory_write_byte(uint32_t addr, uint8_t byte_val) {
    if (!is_valid_write_range(addr, 1U)) {
        return false;
    }

    const uint32_t aligned_addr = addr & ~0x03UL;
    const uint32_t byte_idx = addr & 0x03UL;
    const uint32_t current_word = dwire_hw_read_mem32(aligned_addr);

    const uint32_t mask = 0xFFUL << (byte_idx * 8U);
    const uint32_t new_word = (current_word & ~mask) | (((uint32_t)byte_val) << (byte_idx * 8U));

    dwire_hw_write_mem32(aligned_addr, new_word);

    if (s_cache_valid && (addr >= s_cache_base_addr) && (addr < (s_cache_base_addr + GDB_SECTOR_CACHE_SIZE))) {
        s_sector_cache[addr - s_cache_base_addr] = byte_val;
    }
    return true;
}

void gdb_mem_send_memory_map_xml(char *resp_buf, size_t resp_cap) {
    const uint32_t rom_len = s_flash_size_bytes - 0x00010000UL;
    const uint32_t nvram_base = 0x80000000UL + rom_len;
    snprintf(resp_buf, resp_cap,
        "l<memory-map>"
        "<memory type=\"ram\" start=\"0x00000000\" length=\"0x00040000\"/>"
        "<memory type=\"ram\" start=\"0x10000000\" length=\"0x00010000\"/>"
        "<memory type=\"ram\" start=\"0x40000000\" length=\"0x04000000\"/>"
        "<memory type=\"rom\" start=\"0x80000000\" length=\"0x%08lX\"/>"
        "<memory type=\"flash\" start=\"0x%08lX\" length=\"0x00010000\">"
        "<property name=\"blocksize\">0x2000</property>"
        "</memory>"
        "</memory-map>",
        (unsigned long)rom_len,
        (unsigned long)nvram_base);
    gdb_rsp_send_packet(resp_buf);
}

void gdb_mem_handle_read_cmd(const char *arg, char *resp_buf) {
    uint32_t addr = 0U;
    uint32_t length = 0U;
    if (gdb_parse_addr_and_len(arg, &addr, &length, '\0') == NULL) {
        gdb_rsp_send_packet("E02");
        return;
    }

    char *out = resp_buf;
    for (uint32_t i = 0U; i < length; i++) {
        const uint8_t b = memory_read_byte(addr + i);
        *out++ = gdb_nibble_to_hex_char((uint8_t)(b >> 4));
        *out++ = gdb_nibble_to_hex_char((uint8_t)(b & 0x0FU));
    }
    *out = '\0';
    gdb_rsp_send_packet(resp_buf);
}

void gdb_mem_handle_write_cmd(const char *arg) {
    uint32_t addr = 0U;
    uint32_t length = 0U;
    const char *p = gdb_parse_addr_and_len(arg, &addr, &length, ':');
    if (p == NULL) {
        gdb_rsp_send_packet("E03");
        return;
    }

    if (!is_valid_write_range(addr, length)) {
        gdb_rsp_send_packet("E05");
        return;
    }

    if (strlen(p) < (length * 2U)) {
        gdb_rsp_send_packet("E04");
        return;
    }

    bool write_ok = true;
    for (uint32_t i = 0U; i < length; i++) {
        const int h = gdb_hex_char_to_nibble(*p++);
        const int l = gdb_hex_char_to_nibble(*p++);
        if ((h >= 0) && (l >= 0)) {
            const uint8_t byte = (uint8_t)((h << 4) | l);
            if (!memory_write_byte(addr + i, byte)) {
                write_ok = false;
                break;
            }
        } else {
            gdb_rsp_send_packet("E04");
            return;
        }
    }

    gdb_rsp_send_packet(write_ok ? "OK" : "E05");
}
