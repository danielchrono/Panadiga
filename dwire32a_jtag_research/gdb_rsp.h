/**
 * @file gdb_rsp.h
 * @brief GDB Remote Serial Protocol (RSP) Server for Panasonic AM34 / MN103S.
 *
 * Implements standard GDB remote protocol framing ($packet#checksum), register
 * access, memory read/write, breakpoint management, and single-step control
 * over USB CDC serial on Raspberry Pi Pico (RP2040) and Pico 2 (RP2350).
 *
 * Target: Panasonic MN2DS0016 (Matsushita AM34 / MN103S core)
 * PCB:    Panasonic DMR-EH55 Main Digital Board VEP79138
 *
 * Architecture:
 *   [ mn10300-elf-gdb / gdb-multiarch ] <--- USB CDC (RSP) ---> [ Pico RP2040/RP2350 ] <--- DWire32A (PIO) ---> [ MN2DS0016 ]
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef GDB_RSP_H
#define GDB_RSP_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include "dwire32a_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/* =========================================================================
 * Hardware & Memory Configuration
 * ========================================================================= */

/** Maximum size of GDB RSP packet buffer */
#define GDB_RSP_MAX_PACKET_LEN      2048U

/**
 * On RP2350 (520 KB SRAM), allocate 64 KB sector cache for full Spansion sector buffering.
 * On RP2040 (264 KB SRAM), allocate 8 KB microsector cache.
 */
#if defined(PICO_RP2350) && (PICO_RP2350 != 0)
    #define GDB_SECTOR_CACHE_SIZE   65536U
    #define GDB_TARGET_CHIP_LABEL   "RP2350 (Pico 2) - 520KB SRAM - 64KB Sector Cache"
#else
    #define GDB_SECTOR_CACHE_SIZE   8192U
    #define GDB_TARGET_CHIP_LABEL   "RP2040 (Pico 1) - 264KB SRAM - 8KB Sector Cache"
#endif

/* =========================================================================
 * AM34 / MN103S Register Definitions for GDB
 * ========================================================================= */

enum am34_gdb_reg {
    AM34_REG_D0  = 0,   /**< Data Register 0 (32-bit) */
    AM34_REG_D1  = 1,   /**< Data Register 1 (32-bit) */
    AM34_REG_D2  = 2,   /**< Data Register 2 (32-bit) */
    AM34_REG_D3  = 3,   /**< Data Register 3 (32-bit) */
    AM34_REG_A0  = 4,   /**< Address Register 0 (32-bit) */
    AM34_REG_A1  = 5,   /**< Address Register 1 (32-bit) */
    AM34_REG_A2  = 6,   /**< Address Register 2 (32-bit) */
    AM34_REG_A3  = 7,   /**< Address Register 3 (32-bit) */
    AM34_REG_MDR = 8,   /**< Multiply-Divide Register (32-bit) */
    AM34_REG_PSW = 9,   /**< Processor Status Word (32-bit) */
    AM34_REG_PC  = 10,  /**< Program Counter (32-bit) */
    AM34_REG_SP  = 11,  /**< Stack Pointer (32-bit) */
    AM34_REG_COUNT = 12 /**< Total general registers reported to GDB */
};
typedef enum am34_gdb_reg am34_gdb_reg_t;

/**
 * @brief Shadow register file maintaining current AM34 CPU state.
 */
typedef struct am34_reg_file {
    uint32_t d[4];      /**< D0, D1, D2, D3 */
    uint32_t a[4];      /**< A0, A1, A2, A3 */
    uint32_t mdr;       /**< MDR */
    uint32_t psw;       /**< PSW */
    uint32_t pc;        /**< PC */
    uint32_t sp;        /**< SP */
    bool is_halted;     /**< True if target CPU is held in OCD halt */
} am34_reg_file_t;

/* =========================================================================
 * Server API
 * ========================================================================= */

/**
 * @brief Initialize GDB RSP server state, register file, and sector cache.
 */
void gdb_rsp_init(void);

/**
 * @brief Check if GDB RSP server mode is currently active.
 *
 * @return true if communicating via GDB RSP, false if in interactive CLI console mode.
 */
bool gdb_rsp_is_active(void);

/**
 * @brief Switch GDB RSP mode on or off.
 *
 * @param active true to enter GDB RSP mode, false to return to CLI prompt.
 */
void gdb_rsp_set_active(bool active);

/**
 * @brief Feed a single received byte into the GDB RSP state machine.
 *
 * Handles frame start ('$'), checksum reception ('#xx'), ACK/NACK ('+', '-'),
 * and Ctrl-C ('\x03') async interrupt.
 *
 * @param ch Received ASCII or control character.
 */
void gdb_rsp_process_char(char ch);

/**
 * @brief Format and transmit a framed GDB RSP packet: "$<payload>#<checksum>".
 *
 * @param payload Null-terminated response string.
 */
void gdb_rsp_send_packet(const char *payload);

/**
 * @brief Invalidate internal sector memory cache.
 */
void gdb_rsp_invalidate_cache(void);

/**
 * @brief Get human-readable description of current hardware/cache allocation.
 */
const char *gdb_rsp_get_status_info(void);

#ifdef __cplusplus
}
#endif

#endif /* GDB_RSP_H */
