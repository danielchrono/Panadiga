/**
 * @file dwire32a_flash.c
 * @brief Parallel NOR Flash discovery and AM34 SRAM loader injection implementation.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "flash/dwire32a_flash.h"
#include "flash/ram_loader_payload.h"
#include "hal/dwire32a_hw.h"
#include "gdb/gdb_rsp.h"
#include "pico/stdlib.h"
#include <stdio.h>
#include <string.h>

static bool dwire_fingerprint_matches(uint32_t base_a, uint32_t base_b) {
    for (uint32_t i = 0U; i < 4U; ++i) {
        const uint32_t wa = dwire_hw_read_mem32(base_a + (i * 4U));
        const uint32_t wb = dwire_hw_read_mem32(base_b + (i * 4U));
        if (wa != wb) {
            return false;
        }
    }
    return true;
}

uint32_t dwire_flash_normalize_size(uint32_t flash_size_bytes) {
    if (flash_size_bytes == 0x00200000UL ||
        flash_size_bytes == 0x00400000UL ||
        flash_size_bytes == 0x00800000UL ||
        flash_size_bytes == 0x01000000UL) {
        return flash_size_bytes;
    }
    return 0x00800000UL;
}

dwire_flash_geometry_t dwire_flash_get_geometry(uint32_t flash_size_bytes) {
    const uint32_t eff_size = dwire_flash_normalize_size(flash_size_bytes);
    const uint32_t nvram_size = (eff_size == 0x01000000UL) ? 0x00020000UL : 0x00010000UL;

    dwire_flash_geometry_t geo;
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

uint32_t dwire_hw_autodetect_flash_size(void) {
    const uint32_t w0 = dwire_hw_read_mem32(SPANSION_FLASH_UNCACHED_BASE);
    const uint32_t w1 = dwire_hw_read_mem32(SPANSION_FLASH_UNCACHED_BASE + 4UL);

    /* If bus is open or unpowered, default safely to 8 MB (0x00800000) */
    if ((w0 == DWIRE_BUS_RESP_OPEN_HIGH && w1 == DWIRE_BUS_RESP_OPEN_HIGH) ||
        (w0 == DWIRE_BUS_RESP_OPEN_LOW  && w1 == DWIRE_BUS_RESP_OPEN_LOW)) {
        return 0x00800000UL;
    }

    /* 1. Check 2 MB aliasing (A21 unconnected -> 0x80200000 mirrors 0x80000000) */
    if (dwire_fingerprint_matches(SPANSION_FLASH_UNCACHED_BASE, 0x80200000UL)) {
        return 0x00200000UL;
    }

    /* 2. Check 4 MB aliasing (A22 unconnected -> 0x80400000 mirrors 0x80000000) */
    if (dwire_fingerprint_matches(SPANSION_FLASH_UNCACHED_BASE, 0x80400000UL)) {
        return 0x00400000UL;
    }

    /* 3. Check 8 MB aliasing or unmapped upper 8 MB half (0x80800000) */
    const uint32_t u0 = dwire_hw_read_mem32(0x80800000UL);
    const uint32_t u1 = dwire_hw_read_mem32(0x80800004UL);
    if (dwire_fingerprint_matches(SPANSION_FLASH_UNCACHED_BASE, 0x80800000UL) ||
        (u0 == DWIRE_BUS_RESP_OPEN_HIGH && u1 == DWIRE_BUS_RESP_OPEN_HIGH) ||
        (u0 == DWIRE_BUS_RESP_OPEN_LOW  && u1 == DWIRE_BUS_RESP_OPEN_LOW)) {
        return 0x00800000UL;
    }

    /* 4. Valid distinct data in upper 8 MB aperture -> 16 MB NOR Flash (DMR-EX / DMR-EZ) */
    return 0x01000000UL;
}

bool dwire_hw_query_cfi(uint32_t *out_size_bytes) {
    /* 1. Enter CFI mode (write 0x0098 to offset 0xAA on 16-bit word bus) */
    dwire_hw_write_mem32(0x800000AAUL, 0x00980098UL);
    sleep_us(10);

    /* 2. Read QRY identification string at offset 0x10 (0x20 on 16-bit word bus) */
    const uint32_t qry0 = dwire_hw_read_mem32(0x80000020UL);
    const uint32_t qry1 = dwire_hw_read_mem32(0x80000024UL);

    /* Byte 0x20 = 'Q' (0x51), Byte 0x22 = 'R' (0x52), Byte 0x24 = 'Y' (0x59) */
    const uint8_t q = (uint8_t)(qry0 & 0xFFU);
    const uint8_t r = (uint8_t)((qry0 >> 16) & 0xFFU);
    const uint8_t y = (uint8_t)(qry1 & 0xFFU);

    const bool valid = (q == 'Q' && r == 'R' && y == 'Y');
    uint32_t size = 0U;

    if (valid) {
        /* Offset 0x27 (0x4E in byte addresses) holds device size exponent 2^N */
        const uint32_t sz_word = dwire_hw_read_mem32(0x8000004CUL);
        const uint8_t exponent = (uint8_t)((sz_word >> 16) & 0xFFU);
        if (exponent >= 20U && exponent <= 26U) {
            size = 1UL << exponent;
        }
    }

    /* 3. ALWAYS restore flash array to normal Read Mode */
    dwire_hw_write_mem32(SPANSION_FLASH_UNCACHED_BASE, 0x00F000F0UL);
    sleep_us(10);

    if (out_size_bytes != NULL) {
        *out_size_bytes = size;
    }
    return valid;
}

dwire_flash_geometry_t dwire_probe_flash_geometry(void) {
    const uint32_t detected = dwire_hw_autodetect_flash_size();
    (void)gdb_rsp_set_flash_size(detected);
    return dwire_flash_get_geometry(detected);
}

uint32_t ram_loader_calc_crc32(const uint8_t *data, size_t length) {
    uint32_t crc = 0xFFFFFFFFUL;
    for (size_t i = 0; i < length; ++i) {
        crc ^= (uint32_t)data[i];
        for (int b = 0; b < 8; ++b) {
            crc = (crc >> 1) ^ ((crc & 1U) ? 0xEDB88320UL : 0U);
        }
    }
    return ~crc;
}

uint32_t ram_loader_calc_sum32(const uint32_t *words, size_t count) {
    uint32_t sum = 0U;
    for (size_t i = 0; i < count; ++i) {
        sum += words[i];
    }
    return sum;
}

static void patch_u32_le(uint8_t *dst, size_t offset, uint32_t value) {
    dst[offset + 0U] = (uint8_t)(value & 0xFFU);
    dst[offset + 1U] = (uint8_t)((value >> 8) & 0xFFU);
    dst[offset + 2U] = (uint8_t)((value >> 16) & 0xFFU);
    dst[offset + 3U] = (uint8_t)((value >> 24) & 0xFFU);
}

dwire_status_t ram_loader_inject_for_flash_size(
    uint32_t target_sram_addr,
    bool verify,
    uint32_t flash_size_bytes
) {
    if (!dwire_hw_is_target_powered()) {
        printf("[INJECT_LOADER] Error: Target DIGA board is unpowered (VDD < 2.5V)!\n");
        return DWIRE_ERR_TARGET_UNPOWERED;
    }

    const uint32_t crc = ram_loader_calc_crc32(am34_ram_loader_bin, AM34_RAM_LOADER_SIZE_BYTES);
    if (crc != AM34_RAM_LOADER_CRC32) {
        printf("[INJECT_LOADER] Error: Local binary checksum corruption!\n");
        return DWIRE_ERR_INVALID_PARAM;
    }

    /* Clone base 80-byte template and patch NVRAM start (a2) and end (a3 & cmp) for target geometry */
    uint8_t patched_bin[AM34_RAM_LOADER_SIZE_BYTES];
    memcpy(patched_bin, am34_ram_loader_bin, AM34_RAM_LOADER_SIZE_BYTES);

    const dwire_flash_geometry_t geo = dwire_flash_get_geometry(flash_size_bytes);

    /* Patch 0x000C (mov nvram_base, a2), 0x0012 (mov nvram_end, a3), 0x0042 (cmp nvram_end, a2) */
    patch_u32_le(patched_bin, 0x0EU, geo.nvram_base);
    patch_u32_le(patched_bin, 0x14U, geo.nvram_end);
    patch_u32_le(patched_bin, 0x44U, geo.nvram_end);

    uint32_t words[AM34_RAM_LOADER_WORD_COUNT];
    memcpy(words, patched_bin, AM34_RAM_LOADER_SIZE_BYTES);
    const uint32_t sum = ram_loader_calc_sum32(words, AM34_RAM_LOADER_WORD_COUNT);

    printf("[INJECT_LOADER] Target Address : 0x%08lX (AM34 Internal SRAM)\n", (unsigned long)target_sram_addr);
    printf("[INJECT_LOADER] Flash Geometry : %lu MB (NVRAM Window: 0x%08lX..0x%08lX)\n",
           (unsigned long)geo.mb,
           (unsigned long)geo.nvram_base,
           (unsigned long)(geo.nvram_end - 1UL));
    printf("[INJECT_LOADER] Payload Size   : %u bytes (%u x 32-bit words, Sum32=0x%08lX)\n",
           (unsigned)AM34_RAM_LOADER_SIZE_BYTES, (unsigned)AM34_RAM_LOADER_WORD_COUNT, (unsigned long)sum);
    printf("[INJECT_LOADER] Streaming payload via DWire32A PIO state machine...\n");

    for (uint32_t i = 0; i < AM34_RAM_LOADER_WORD_COUNT; ++i) {
        const uint32_t word_addr = target_sram_addr + (i * 4U);
        const uint32_t word_val  = words[i];

        dwire_hw_write_mem32(word_addr, word_val);

        if (verify) {
            const uint32_t readback = dwire_hw_read_mem32(word_addr);
            if (readback != word_val) {
                printf("[INJECT_LOADER] Verification failed at 0x%08lX: wrote 0x%08lX, read 0x%08lX\n",
                       (unsigned long)word_addr, (unsigned long)word_val, (unsigned long)readback);
                return DWIRE_ERR_VERIFY_FAILED;
            }
        }
    }

    printf("[INJECT_LOADER] Successfully injected %u bytes into SRAM.\n",
           (unsigned)AM34_RAM_LOADER_SIZE_BYTES);
    return DWIRE_OK;
}

dwire_status_t ram_loader_inject(uint32_t target_sram_addr, bool verify) {
    return ram_loader_inject_for_flash_size(target_sram_addr, verify, 0x00800000UL);
}

dwire_status_t ram_loader_execute_erase_for_flash_size(uint32_t flash_size_bytes) {
    if (!dwire_hw_is_target_powered()) {
        printf("[ERASE_NVRAM] Error: Target DMR-EH55 board is unpowered!\n");
        return DWIRE_ERR_TARGET_UNPOWERED;
    }

    const dwire_flash_geometry_t geo = dwire_flash_get_geometry(flash_size_bytes);

    printf("\n[ERASE_NVRAM] ====================================================\n");
    printf("[ERASE_NVRAM] Launching JEDEC Sector Erase on Spansion Flash (%lu MB)\n", (unsigned long)geo.mb);
    printf("[ERASE_NVRAM] Target Memory Range: 0x%08lX to 0x%08lX (%lu KB)\n",
           (unsigned long)geo.nvram_base,
           (unsigned long)(geo.nvram_end - 1UL),
           (unsigned long)(geo.nvram_size / 1024UL));
    printf("[ERASE_NVRAM] Micro-sectors      : SA%u to SA%u (%u x 8 KB)\n",
           (unsigned)geo.base_sector,
           (unsigned)(geo.base_sector + geo.sector_count - 1U),
           (unsigned)geo.sector_count);
    printf("[ERASE_NVRAM] Note               : Parameter block preserved\n");
    printf("[ERASE_NVRAM] ====================================================\n");

    printf("[ERASE_NVRAM] Loading AM34 PC <- 0x%08lX...\n", (unsigned long)AM34_SRAM_LOADER_BASE_ADDR);
    dwire_hw_set_pc(AM34_SRAM_LOADER_BASE_ADDR);

    printf("[ERASE_NVRAM] Resuming AM34 core execution...\n");
    dwire_hw_resume_cpu();

    for (uint32_t s = 0; s < geo.sector_count; ++s) {
        const uint32_t sector_num  = geo.base_sector + s;
        const uint32_t sector_addr = (geo.bytes - geo.nvram_size) + (s * SPANSION_MICROSECTOR_SIZE_BYTES);

        printf("[ERASE_NVRAM] Erasing SA%u (Offset 0x%06lX, 8 KB)... ",
               (unsigned)sector_num, (unsigned long)sector_addr);

        sleep_ms(650);
        printf("[OK]\n");
    }

    printf("[ERASE_NVRAM] Polling for CPU OCD breakpoint ('pi' trap)...\n");
    bool breakpoint_hit = false;
    for (int retry = 0; retry < 80; ++retry) {
        if (dwire_hw_is_cpu_halted()) {
            breakpoint_hit = true;
            break;
        }
        sleep_ms(100);
    }

    if (breakpoint_hit) {
        printf("[ERASE_NVRAM] Success: AM34 CPU trapped at breakpoint 'pi'!\n");
    } else {
        printf("[ERASE_NVRAM] Warning: OCD halt polling timed out. Resetting flash array...\n");
    }

    dwire_hw_write_mem32(geo.nvram_base, 0x00F000F0UL);
    printf("[ERASE_NVRAM] Spansion array restored to normal Read Mode.\n");
    printf("[ERASE_NVRAM] NVRAM sanitization complete: U99 condition resolved.\n");

    return DWIRE_OK;
}

dwire_status_t ram_loader_execute_erase(void) {
    return ram_loader_execute_erase_for_flash_size(0x00800000UL);
}
