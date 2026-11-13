/**
 * @file gdb_rsp.h
 * @brief Public GDB Remote Serial Protocol (RSP) Server API for Panasonic AM34 / MN103S.
 *
 * Implements standard GDB remote protocol framing ($packet#checksum), register
 * access, memory read/write with hardware IPL air-gap, breakpoint management,
 * and single-step control over USB CDC serial on RP2040 / RP2350.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef GDB_RSP_H
#define GDB_RSP_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include "p7501_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/** Maximum size of GDB RSP packet buffer. */
#define GDB_RSP_MAX_PACKET_LEN      2048U

/**
 * On RP2350 (520 KB SRAM), allocate 64 KB sector cache for full Spansion sector buffering.
 * On RP2040 (264 KB SRAM), allocate 8 KB microsector cache.
 */
#if defined(PICO_RP2350) && (PICO_RP2350 != 0)
    #define GDB_SECTOR_CACHE_SIZE   65536U
    #if defined(PICO_RP2350_RISCV) && (PICO_RP2350_RISCV != 0)
        #define GDB_TARGET_CHIP_LABEL   "RP2350 (Pico 2 RISC-V Hazard3) - 520KB SRAM - 64KB Sector Cache"
    #else
        #define GDB_TARGET_CHIP_LABEL   "RP2350 (Pico 2 ARM Cortex-M33) - 520KB SRAM - 64KB Sector Cache"
    #endif
#else
    #define GDB_SECTOR_CACHE_SIZE   8192U
    #define GDB_TARGET_CHIP_LABEL   "RP2040 (Pico 1 Dual Cortex-M0+) - 264KB SRAM - 8KB Sector Cache"
#endif

/** AM34 / MN103S register layout indices exposed to GDB. */
enum am34_gdb_reg {
    AM34_REG_D0  = 0,
    AM34_REG_D1  = 1,
    AM34_REG_D2  = 2,
    AM34_REG_D3  = 3,
    AM34_REG_A0  = 4,
    AM34_REG_A1  = 5,
    AM34_REG_A2  = 6,
    AM34_REG_A3  = 7,
    AM34_REG_MDR = 8,
    AM34_REG_PSW = 9,
    AM34_REG_PC  = 10,
    AM34_REG_SP  = 11,
    AM34_REG_COUNT = 12
};
typedef enum am34_gdb_reg am34_gdb_reg_t;

/** Shadow register file maintaining current AM34 CPU state. */
typedef struct am34_reg_file {
    uint32_t d[4];
    uint32_t a[4];
    uint32_t mdr;
    uint32_t psw;
    uint32_t pc;
    uint32_t sp;
    bool is_halted;
} am34_reg_file_t;

void gdb_rsp_init(void);

#if defined(PICO_HOST_TEST) && (PICO_HOST_TEST != 0)
void gdb_rsp_test_inject_packet(const char *payload);
void gdb_rsp_test_get_last_response(char *out, size_t max_len);
#endif

bool gdb_rsp_is_active(void);
void gdb_rsp_set_active(bool active);
void gdb_rsp_process_char(char ch);
void gdb_rsp_send_packet(const char *payload);
void gdb_rsp_invalidate_cache(void);
const char *gdb_rsp_get_status_info(void);
bool gdb_rsp_set_flash_size(uint32_t flash_size_bytes);
uint32_t gdb_rsp_get_flash_size(void);

#ifdef __cplusplus
}
#endif

#endif /* GDB_RSP_H */
