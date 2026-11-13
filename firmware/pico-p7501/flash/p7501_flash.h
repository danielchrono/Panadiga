/**
 * @file p7501_flash.h
 * @brief Parallel NOR Flash geometry discovery & Spansion EEMUL NVRAM recovery engine.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_FLASH_H
#define P7501_FLASH_H

#include <stddef.h>
#include "p7501_types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct p7501_flash_geometry {
    uint32_t bytes;        /**< Total Flash size in bytes (2MB/4MB/8MB/16MB) */
    uint32_t mb;           /**< Total Flash size in MiB */
    uint32_t nvram_size;   /**< EEMUL NVRAM aperture size (64KB or 128KB for 16MB) */
    uint32_t nvram_base;   /**< Physical AM34 uncached start address (0x80xxxxxx) */
    uint32_t nvram_end;    /**< Physical AM34 uncached exclusive end address */
    uint32_t base_sector;  /**< First Spansion micro-sector index (e.g. SA134) */
    uint32_t sector_count; /**< Number of 8 KB micro-sectors in NVRAM partition */
} p7501_flash_geometry_t;

uint32_t p7501_flash_normalize_size(uint32_t flash_size_bytes);
p7501_flash_geometry_t p7501_flash_get_geometry(uint32_t flash_size_bytes);
uint32_t p7501_hw_autodetect_flash_size(void);
bool p7501_hw_query_cfi(uint32_t *out_size_bytes);
p7501_flash_geometry_t p7501_probe_flash_geometry(void);

uint32_t ram_loader_calc_crc32(const uint8_t *data, size_t length);
uint32_t ram_loader_calc_sum32(const uint32_t *words, size_t count);

p7501_status_t ram_loader_inject_for_flash_size(
    uint32_t target_sram_addr,
    bool verify,
    uint32_t flash_size_bytes
);
p7501_status_t ram_loader_inject(uint32_t target_sram_addr, bool verify);
p7501_status_t ram_loader_execute_erase_for_flash_size(uint32_t flash_size_bytes);
p7501_status_t ram_loader_execute_erase(void);

#ifdef __cplusplus
}
#endif

#endif /* P7501_FLASH_H */
