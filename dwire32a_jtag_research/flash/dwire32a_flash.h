/**
 * @file dwire32a_flash.h
 * @brief Parallel NOR Flash geometry discovery & Spansion EEMUL NVRAM recovery engine.
 *
 * Supports multi-geometry Top-Boot NOR Flash devices across Panasonic DIGA
 * recorders (2 MB / 4 MB / 8 MB / 16 MB), CFI query inspection, and AM34
 * SRAM loader injection for U99 NVRAM sector sanitization.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef DWIRE32A_FLASH_H
#define DWIRE32A_FLASH_H

#include <stddef.h>
#include "dwire32a_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Flash geometry metadata resolved from chip capacity in bytes.
 */
typedef struct dwire_flash_geometry {
    uint32_t bytes;        /**< Total Flash size in bytes (2MB/4MB/8MB/16MB) */
    uint32_t mb;           /**< Total Flash size in MiB */
    uint32_t nvram_size;   /**< EEMUL NVRAM aperture size (64KB or 128KB for 16MB) */
    uint32_t nvram_base;   /**< Physical AM34 uncached start address (0x80xxxxxx) */
    uint32_t nvram_end;    /**< Physical AM34 uncached exclusive end address */
    uint32_t base_sector;  /**< First Spansion micro-sector index (e.g. SA134) */
    uint32_t sector_count; /**< Number of 8 KB micro-sectors in NVRAM partition */
} dwire_flash_geometry_t;

/**
 * @brief Normalize candidate Flash size to a valid supported geometry (default: 8 MB).
 */
uint32_t dwire_flash_normalize_size(uint32_t flash_size_bytes);

/**
 * @brief Compute NVRAM boundaries and sector layout for a given Flash size.
 */
dwire_flash_geometry_t dwire_flash_get_geometry(uint32_t flash_size_bytes);

/**
 * @brief Auto-detect target NOR Flash geometry (2 MB, 4 MB, 8 MB, or 16 MB) via
 *        100% passive read-only Address Aliasing on the AM34 aperture (0x80000000).
 *
 * @return Detected NOR Flash size in bytes (0x00200000, 0x00400000, 0x00800000, or 0x01000000).
 */
uint32_t dwire_hw_autodetect_flash_size(void);

/**
 * @brief Query Common Flash Interface (JEDEC CFI Query 0x98) to read laser-etched geometry.
 *
 * @param[out] out_size_bytes Pointer to receive detected size in bytes (or NULL).
 * @return true if valid 'QRY' signature and size exponent were decoded, false otherwise.
 */
bool dwire_hw_query_cfi(uint32_t *out_size_bytes);

/**
 * @brief Discover Flash geometry passively and synchronize GDB RSP memory map limits.
 *
 * @return Resolved Flash and NVRAM geometry descriptor.
 */
dwire_flash_geometry_t dwire_probe_flash_geometry(void);

/**
 * @brief Calculate IEEE 802.3 CRC-32 checksum of a memory buffer.
 */
uint32_t ram_loader_calc_crc32(const uint8_t *data, size_t length);

/**
 * @brief Calculate additive 32-bit checksum of 32-bit words in payload.
 */
uint32_t ram_loader_calc_sum32(const uint32_t *words, size_t count);

/**
 * @brief Patch and inject the AM34 RAM loader binary word-by-word into target SRAM via PIO.
 *
 * @param target_sram_addr Destination address in target SRAM (default: 0x00004000).
 * @param verify           Whether to read back each word to verify integrity.
 * @param flash_size_bytes Total NOR Flash size (2MB/4MB/8MB/16MB) for NVRAM bounds patching.
 * @return DWIRE_OK on success, or error code on failure.
 */
dwire_status_t ram_loader_inject_for_flash_size(
    uint32_t target_sram_addr,
    bool verify,
    uint32_t flash_size_bytes
);

/**
 * @brief Convenience wrapper injecting RAM loader configured for default 8 MB Flash.
 */
dwire_status_t ram_loader_inject(uint32_t target_sram_addr, bool verify);

/**
 * @brief Execute the SRAM loader on AM34 CPU to erase NVRAM micro-sectors for target size.
 *
 * @param flash_size_bytes Configured NOR Flash size in bytes.
 * @return DWIRE_OK on successful erasure, or error code on failure.
 */
dwire_status_t ram_loader_execute_erase_for_flash_size(uint32_t flash_size_bytes);

/**
 * @brief Convenience wrapper executing NVRAM sector erase for default 8 MB Flash.
 */
dwire_status_t ram_loader_execute_erase(void);

#ifdef __cplusplus
}
#endif

#endif /* DWIRE32A_FLASH_H */
