/**
 * @file p7501_flash.c
 * @brief Parallel NOR Flash discovery and AM34 SRAM loader injection implementation.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "flash/p7501_flash.h"
#include "flash/ram_loader_payload.h"
#include "hal/p7501_hw.h"
#include "gdb/gdb_rsp.h"
#include "protocol/p7501_bus_protocol.h"
#include "pico/stdlib.h"
#include <stdio.h>

static bool p7501_fingerprint_matches(uint32_t base_a, uint32_t base_b) {
    for (uint32_t i = 0U; i < 4U; ++i) {
        const uint32_t wa = p7501_hw_read_mem32(base_a + (i * 4U));
        const uint32_t wb = p7501_hw_read_mem32(base_b + (i * 4U));
        if (wa != wb) {
            return false;
        }
    }
    return true;
}

uint32_t p7501_flash_normalize_size(uint32_t flash_size_bytes) {
    if ((flash_size_bytes == 0x00200000UL) ||
        (flash_size_bytes == 0x00400000UL) ||
        (flash_size_bytes == 0x00800000UL) ||
        (flash_size_bytes == 0x01000000UL)) {
        return flash_size_bytes;
    }
    return 0x00800000UL;
}

p7501_flash_geometry_t p7501_flash_get_geometry(uint32_t flash_size_bytes) {
    const uint32_t eff_size = p7501_flash_normalize_size(flash_size_bytes);
    const uint32_t nvram_size = (eff_size == 0x01000000UL) ? 0x00020000UL : 0x00010000UL;

    p7501_flash_geometry_t geo;
    geo.bytes = eff_size;
    geo.mb = eff_size / (1024UL * 1024UL);
    geo.nvram_size = nvram_size;
    geo.nvram_base = SPANSION_FLASH_UNCACHED_BASE + (eff_size - nvram_size);
    geo.nvram_end = SPANSION_FLASH_UNCACHED_BASE + eff_size;
    geo.sector_count = nvram_size / SPANSION_MICROSECTOR_SIZE_BYTES;
    geo.base_sector = (eff_size == 0x01000000UL) ? 254U :
                      (eff_size == 0x00400000UL) ? 70U :
                      (eff_size == 0x00200000UL) ? 38U : 134U;
    return geo;
}

uint32_t p7501_hw_autodetect_flash_size(void) {
    const uint32_t w0 = p7501_hw_read_mem32(SPANSION_FLASH_UNCACHED_BASE);
    const uint32_t w1 = p7501_hw_read_mem32(SPANSION_FLASH_UNCACHED_BASE + 4UL);
    if (w0 == P7501_BUS_RESP_OPEN_HIGH && w1 == P7501_BUS_RESP_OPEN_HIGH &&
        !p7501_proto_is_monitor_online()) {
        return 0U;
    }

    if (((w0 == P7501_BUS_RESP_OPEN_HIGH) && (w1 == P7501_BUS_RESP_OPEN_HIGH)) ||
        ((w0 == P7501_BUS_RESP_OPEN_LOW)  && (w1 == P7501_BUS_RESP_OPEN_LOW))) {
        return 0x00800000UL;
    }

    if (p7501_fingerprint_matches(SPANSION_FLASH_UNCACHED_BASE, 0x80200000UL)) {
        return 0x00200000UL;
    }

    if (p7501_fingerprint_matches(SPANSION_FLASH_UNCACHED_BASE, 0x80400000UL)) {
        return 0x00400000UL;
    }

    const uint32_t u0 = p7501_hw_read_mem32(0x80800000UL);
    const uint32_t u1 = p7501_hw_read_mem32(0x80800004UL);
    if (p7501_fingerprint_matches(SPANSION_FLASH_UNCACHED_BASE, 0x80800000UL) ||
        ((u0 == P7501_BUS_RESP_OPEN_HIGH) && (u1 == P7501_BUS_RESP_OPEN_HIGH)) ||
        ((u0 == P7501_BUS_RESP_OPEN_LOW)  && (u1 == P7501_BUS_RESP_OPEN_LOW))) {
        return 0x00800000UL;
    }

    return 0x01000000UL;
}

bool p7501_hw_query_cfi(uint32_t *out_size_bytes) {
    p7501_hw_write_mem32(0x800000AAUL, 0x00980098UL);
    sleep_us(10);

    const uint32_t qry0 = p7501_hw_read_mem32(0x80000020UL);
    const uint32_t qry1 = p7501_hw_read_mem32(0x80000024UL);

    const uint8_t q = (uint8_t)(qry0 & 0xFFU);
    const uint8_t r = (uint8_t)((qry0 >> 16) & 0xFFU);
    const uint8_t y = (uint8_t)(qry1 & 0xFFU);

    const bool valid = ((q == 'Q') && (r == 'R') && (y == 'Y'));
    uint32_t size = 0U;

    if (valid) {
        const uint32_t sz_word = p7501_hw_read_mem32(0x8000004CUL);
        const uint8_t exponent = (uint8_t)((sz_word >> 16) & 0xFFU);
        if ((exponent >= 20U) && (exponent <= 26U)) {
            size = 1UL << exponent;
        }
    }

    p7501_hw_write_mem32(SPANSION_FLASH_UNCACHED_BASE, 0x00F000F0UL);
    sleep_us(10);

    if (out_size_bytes != NULL) {
        *out_size_bytes = size;
    }
    return valid;
}

p7501_flash_geometry_t p7501_probe_flash_geometry(void) {
    const uint32_t detected = p7501_hw_autodetect_flash_size();
    if (detected == 0U) {
        p7501_flash_geometry_t none = {0U, 0U, 0U, 0U, 0U, 0U, 0U};
        return none;
    }
    (void)gdb_rsp_set_flash_size(detected);
    return p7501_flash_get_geometry(detected);
}
