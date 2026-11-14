/**
 * @file p7501_ram_loader.c
 * @brief Guarded AM34 SRAM loader injection and Spansion EEMUL NVRAM erase executor.
 *
 * Strictly verifies an active synchronous monitor (`DCSR != 0xFFFFFFFF`) and
 * target VDD stability before streaming payloads or reporting NVRAM sanitization.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "flash/p7501_flash.h"

#include "flash/ram_loader_payload.h"
#include "hal/p7501_hw.h"
#include "hal/p7501_led.h"
#include "protocol/p7501_bus_protocol.h"

#include <stdio.h>
#include <string.h>

#if defined(PICO_HOST_TEST) && (PICO_HOST_TEST != 0)
void sleep_ms(uint32_t ms);
#else
#include "pico/stdlib.h"
#endif

uint32_t ram_loader_calc_crc32(const uint8_t *data, size_t length) {
    uint32_t crc = 0xFFFFFFFFUL;
    for (size_t i = 0u; i < length; ++i) {
        crc ^= (uint32_t)data[i];
        for (int b = 0; b < 8; ++b) {
            crc = (crc >> 1) ^ ((crc & 1U) ? 0xEDB88320UL : 0U);
        }
    }
    return ~crc;
}

uint32_t ram_loader_calc_sum32(const uint32_t *words, size_t count) {
    uint32_t sum = 0U;
    for (size_t i = 0u; i < count; ++i) {
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

p7501_status_t ram_loader_inject_for_flash_size(
    uint32_t target_sram_addr,
    bool verify,
    uint32_t flash_size_bytes
) {
    if (!p7501_proto_is_monitor_online()) {
        printf("[INJECT_LOADER] ERROR: No synchronous AM34 monitor online (DCSR=0xFFFFFFFF). Aborting injection.\n");
        return P7501_ERR_TIMEOUT;
    }
    if (!p7501_hw_is_target_powered()) {
        printf("[INJECT_LOADER] Error: Target DIGA board is unpowered (SW_UART/VDD < 2.5V)!\n");
        return P7501_ERR_TARGET_UNPOWERED;
    }

    const uint32_t crc = ram_loader_calc_crc32(am34_ram_loader_bin, AM34_RAM_LOADER_SIZE_BYTES);
    if (crc != AM34_RAM_LOADER_CRC32) {
        printf("[INJECT_LOADER] Error: Local binary checksum corruption!\n");
        return P7501_ERR_INVALID_PARAM;
    }

    uint8_t patched_bin[AM34_RAM_LOADER_SIZE_BYTES];
    memcpy(patched_bin, am34_ram_loader_bin, AM34_RAM_LOADER_SIZE_BYTES);

    const p7501_flash_geometry_t geo = p7501_flash_get_geometry(flash_size_bytes);
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

    for (uint32_t i = 0u; i < AM34_RAM_LOADER_WORD_COUNT; ++i) {
        const uint32_t word_addr = target_sram_addr + (i * 4U);
        const uint32_t word_val  = words[i];
        p7501_hw_write_mem32(word_addr, word_val);
        if (verify) {
            const uint32_t readback = p7501_hw_read_mem32(word_addr);
            if (readback != word_val) {
                printf("[INJECT_LOADER] Verification failed at 0x%08lX: wrote 0x%08lX, read 0x%08lX\n",
                       (unsigned long)word_addr, (unsigned long)word_val, (unsigned long)readback);
                return P7501_ERR_VERIFY_FAILED;
            }
        }
    }

    printf("[INJECT_LOADER] Successfully injected %u bytes into SRAM.\n",
           (unsigned)AM34_RAM_LOADER_SIZE_BYTES);
    return P7501_OK;
}

p7501_status_t ram_loader_inject(uint32_t target_sram_addr, bool verify) {
    return ram_loader_inject_for_flash_size(target_sram_addr, verify, 0x00800000UL);
}

p7501_status_t ram_loader_execute_erase_for_flash_size(uint32_t flash_size_bytes) {
    if (!p7501_proto_is_monitor_online()) {
        printf("[ERASE_NVRAM] ERROR: No synchronous AM34 monitor online (DCSR=0xFFFFFFFF). Zero sectors erased.\n");
        return P7501_ERR_TIMEOUT;
    }
    if (!p7501_hw_is_target_powered()) {
        printf("[ERASE_NVRAM] Error: Target DMR-EH55 board is unpowered!\n");
        return P7501_ERR_TARGET_UNPOWERED;
    }

    const p7501_flash_geometry_t geo = p7501_flash_get_geometry(flash_size_bytes);
    printf("\n[ERASE_NVRAM] Launching JEDEC Sector Erase (%lu MB | 0x%08lX..0x%08lX, SA%u..SA%u)\n",
           (unsigned long)geo.mb,
           (unsigned long)geo.nvram_base,
           (unsigned long)(geo.nvram_end - 1UL),
           (unsigned)geo.base_sector,
           (unsigned)(geo.base_sector + geo.sector_count - 1U));

    p7501_hw_set_pc(AM34_SRAM_LOADER_BASE_ADDR);
    p7501_hw_resume_cpu();

    for (uint32_t s = 0u; s < geo.sector_count; ++s) {
        const uint32_t sector_num  = geo.base_sector + s;
        const uint32_t sector_addr = (geo.bytes - geo.nvram_size) + (s * SPANSION_MICROSECTOR_SIZE_BYTES);
        printf("[ERASE_NVRAM] Waiting SA%u (Offset 0x%06lX, 8 KB)...\n",
               (unsigned)sector_num, (unsigned long)sector_addr);
        p7501_led_sleep_ms(650u);
    }

    bool breakpoint_hit = false;
    for (int retry = 0; retry < 80; ++retry) {
        if (p7501_hw_is_cpu_halted()) {
            breakpoint_hit = true;
            break;
        }
        p7501_led_sleep_ms(100u);
    }

    p7501_hw_write_mem32(geo.nvram_base, 0x00F000F0UL);
    if (!breakpoint_hit) {
        printf("[ERASE_NVRAM] ERROR: Target AM34 breakpoint 'pi' timed out; NVRAM erase unverified.\n");
        return P7501_ERR_TIMEOUT;
    }

    printf("[ERASE_NVRAM] Success: AM34 CPU trapped at breakpoint 'pi'! NVRAM sanitization complete.\n");
    return P7501_OK;
}

p7501_status_t ram_loader_execute_erase(void) {
    return ram_loader_execute_erase_for_flash_size(0x00800000UL);
}
