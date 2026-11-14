/**
 * @file p7501_flash_cli.c
 * @brief CLI command implementations for NOR Flash probing, CFI, SRAM loader, and EEMUL NVRAM.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "flash/p7501_flash_cli.h"
#include "flash/p7501_flash.h"
#include "flash/ram_loader_payload.h"
#include "common/p7501_hex.h"
#include "gdb/gdb_rsp.h"
#include "hal/p7501_hw.h"
#include "hal/p7501_led.h"

#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

void p7501_flash_cmd_autodetect(const char *arg) {
    (void)arg;
    const p7501_flash_geometry_t geo = p7501_probe_flash_geometry();
    if (geo.bytes == 0U) {
        printf("[AUTODETECT] ERROR: No synchronous monitor online (DCSR=0xFFFFFFFF). Flash geometry unknown.\n");
        return;
    }
    printf("[AUTODETECT] Passive Address Aliasing Probe: %" PRIu32 " MB NOR Flash detected (EEMUL NVRAM @ 0x%08" PRIX32 ").\n",
           geo.mb, geo.nvram_base);
}

void p7501_flash_cmd_cfi(const char *arg) {
    (void)arg;
    uint32_t cfi_bytes = 0U;
    if (p7501_hw_query_cfi(&cfi_bytes)) {
        printf("[CFI] Spansion QRY valid: %" PRIu32 " bytes (%" PRIu32 " MB).\n",
               cfi_bytes, cfi_bytes / (1024UL * 1024UL));
    } else {
        printf("[CFI] No valid QRY response (check monitor attach & P7501 connections).\n");
    }
}

void p7501_flash_cmd_size(const char *arg) {
    if ((arg == NULL) || (*arg == '\0')) {
        const p7501_flash_geometry_t geo = p7501_flash_get_geometry(gdb_rsp_get_flash_size());
        printf("[FLASH_SIZE] Active: %" PRIu32 " MB (0x%08" PRIX32 ") | EEMUL NVRAM: 0x%08" PRIX32 "..0x%08" PRIX32 "\n",
               geo.mb, geo.bytes, geo.nvram_base, geo.nvram_end - 1UL);
        return;
    }

    uint32_t req = (uint32_t)strtoul(arg, NULL, 0);
    if (req <= 16U) {
        req *= 1024UL * 1024UL;
    }
    if (gdb_rsp_set_flash_size(req)) {
        const p7501_flash_geometry_t geo = p7501_flash_get_geometry(req);
        printf("[FLASH_SIZE] Configured %" PRIu32 " MB | EEMUL NVRAM: 0x%08" PRIX32 "..0x%08" PRIX32 "\n",
               geo.mb, geo.nvram_base, geo.nvram_end - 1UL);
    } else {
        printf("[FLASH_SIZE] ERROR: Unsupported size. Valid values: 2, 4, 8, 16 (MB).\n");
    }
}

void p7501_flash_cmd_inject_loader(const char *arg) {
    if (!p7501_ascii_ieq(arg, "CONFIRM")) {
        printf("[SAFETY INTERLOCK] Pass CONFIRM to inject AM34 SRAM loader: INJECT_LOADER CONFIRM\n");
        return;
    }

    p7501_led_set_auto_mode(LED_MODE_BUS_ACTIVE);
    const p7501_status_t rc = ram_loader_inject_for_flash_size(
        AM34_SRAM_LOADER_BASE_ADDR,
        true,
        gdb_rsp_get_flash_size()
    );
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);

    if (rc == P7501_OK) {
        printf("INJECT_LOADER: OK\n");
    } else {
        printf("INJECT_LOADER: ERROR (%d)\n", (int)rc);
    }
}

void p7501_flash_cmd_erase_nvram(const char *arg) {
    if (!p7501_ascii_ieq(arg, "CONFIRM")) {
        printf("[SAFETY INTERLOCK] Pass CONFIRM to erase Top-Boot EEMUL NVRAM: ERASE_NVRAM CONFIRM\n");
        return;
    }

    p7501_led_set_auto_mode(LED_MODE_BUS_ACTIVE);
    const p7501_status_t rc = ram_loader_execute_erase_for_flash_size(gdb_rsp_get_flash_size());
    p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);

    if (rc == P7501_OK) {
        printf("ERASE_NVRAM: OK\n");
    } else {
        printf("ERASE_NVRAM: ERROR (%d)\n", (int)rc);
    }
}
