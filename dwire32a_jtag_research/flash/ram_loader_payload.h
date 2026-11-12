/**
 * @file ram_loader_payload.h
 * @brief Packaged AM34 / MN103S Autonomous JEDEC SRAM Payload for Panasonic DWire32A.
 *
 * Contains the native 32-bit machine code binary of the RAM loader injected
 * into the internal SRAM of the Matsushita MN2DS0016 SoC (UniPhier / AM34)
 * to erase the corrupted NVRAM emulation partition (EEMUL) on the Spansion
 * S29JL064H70TFI00 NOR Flash (IC55007), clearing the fatal U99 error state
 * without desoldering the memory IC from the VEP79138 digital board.
 *
 * Target Processor: Panasonic / Matsushita UniPhier AM34 / MN103S Family (MN2DS0004 / MN2DS0009 / MN2DS0016 / MN2DS0018)
 * Target Models:    Panasonic DIGA Series (DMR-E / DMR-ExxH / DMR-ES / DMR-EH / DMR-EX / DMR-EZ)
 * Flash Memory:     Parallel NOR Flash Top-Boot (2 MB / 4 MB / 8 MB / 16 MB — Spansion S29JL/GL, ST M29W, Macronix MX29LV)
 * Injection Engine: Raspberry Pi Pico (RP2040 / RP2350) DWire32A PIO Transceiver
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef RAM_LOADER_PAYLOAD_H
#define RAM_LOADER_PAYLOAD_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* =========================================================================
 * Physical Memory Architecture and JEDEC Command Constants
 * ========================================================================= */

/** Base address of the internal SRAM where the loader binary is injected. */
#define AM34_SRAM_LOADER_BASE_ADDR      0x00004000UL

/** Physical uncached direct aperture for Spansion NOR Flash (CS0/SB0). */
#define SPANSION_FLASH_UNCACHED_BASE    0x80000000UL

/**
 * JEDEC Unlock Addresses in 16-Bit Word Mode (BYTE# = 3.3V / High):
 * CPU byte-addressed bus shifts 16-bit JEDEC addresses left by 1 bit:
 *   Addr1: 0x555 << 1 = 0x0AAA -> 0x80000AAA
 *   Addr2: 0x2AA << 1 = 0x0554 -> 0x80000554
 */
#define SPANSION_JEDEC_UNLOCK_ADDR1     (SPANSION_FLASH_UNCACHED_BASE + 0x0AAAUL)
#define SPANSION_JEDEC_UNLOCK_ADDR2     (SPANSION_FLASH_UNCACHED_BASE + 0x0554UL)

/** JEDEC Flash Commands (16-bit Word Mode) */
#define SPANSION_CMD_UNLOCK1            0x00AAU
#define SPANSION_CMD_UNLOCK2            0x0055U
#define SPANSION_CMD_ERASE_SETUP        0x0080U
#define SPANSION_CMD_SECTOR_ERASE       0x0030U
#define SPANSION_CMD_READ_RESET         0x00F0U

/** Status Polling Flags */
#define SPANSION_POLL_DQ7_MASK          0x0080U /**< Data# Polling bit */
#define SPANSION_ERROR_DQ5_MASK         0x0020U /**< Timeout error bit */

/**
 * NVRAM Emulation Partition (EEMUL) Boundaries:
 * 8 parameter micro-sectors of 8 KB each (SA134 to SA141).
 * Resides at the top of the 8 MB flash space: 0x007F0000 to 0x007FFFFF.
 * Physical uncached addresses: 0x807F0000 to 0x80800000.
 */
#define SPANSION_NVRAM_BASE_OFFSET      0x007F0000UL
#define SPANSION_NVRAM_END_OFFSET       0x00800000UL
#define SPANSION_NVRAM_SECTOR_COUNT     8U
#define SPANSION_MICROSECTOR_SIZE_BYTES 0x00002000UL /**< 8192 bytes (8 KB) */

/** Alternate parameter base offset (SA126..SA133) in case of extended region */
#define SPANSION_PARAM_EXT_BASE_OFFSET  0x007E0000UL

/* =========================================================================
 * AM34 Machine Code Payload Specifications
 * ========================================================================= */

/** Payload size in bytes (4-byte aligned). */
#define AM34_RAM_LOADER_SIZE_BYTES      80U

/** Payload size in 32-bit words. */
#define AM34_RAM_LOADER_WORD_COUNT      (AM34_RAM_LOADER_SIZE_BYTES / 4U)

/** Expected IEEE 802.3 CRC-32 checksum of the binary payload. */
#define AM34_RAM_LOADER_CRC32           0x5F21ADC7UL

/** Additive 32-bit word checksum (little-endian sum of all 20 words). */
#define AM34_RAM_LOADER_SUM32           0xFD456B22UL

/* =========================================================================
 * Native AM34 / MN103S Machine Code Opcode Array
 * =========================================================================
 *
 * Detailed Instruction-by-Instruction Disassembly:
 *
 * Offset Hex Bytes                 Assembly Mnemonic           Comment
 * -------------------------------------------------------------------------
 * 0x0000 FC 80 AA 0A 00 80         mov 0x80000AAA, a0          ; a0 = JEDEC Unlock Addr 1
 * 0x0006 FC 84 54 05 00 80         mov 0x80000554, a1          ; a1 = JEDEC Unlock Addr 2
 * 0x000C FC 88 00 00 7F 80         mov 0x807F0000, a2          ; a2 = NVRAM Sector Start (SA134)
 * 0x0012 FC 8C 00 00 80 80         mov 0x80800000, a3          ; a3 = Flash End Limit (Boundary)
 * 0x0018 2C AA 00                  mov 0x00AA, d0              ; d0 = JEDEC Unlock 1 Command
 * 0x001B 2D 55 00                  mov 0x0055, d1              ; d1 = JEDEC Unlock 2 Command
 * 0x001E 2E 80 00                  mov 0x0080, d2              ; d2 = JEDEC Erase Setup Command
 * 0x0021 2F 30 00                  mov 0x0030, d3              ; d3 = JEDEC Sector Erase Command
 *
 * sector_loop:
 * 0x0024 F0 70                     movhu d0, (a0)              ; Cycle 1: [0x80000AAA] = 0xAA
 * 0x0026 F0 75                     movhu d1, (a1)              ; Cycle 2: [0x80000554] = 0x55
 * 0x0028 F0 72                     movhu d2, (a0)              ; Cycle 3: [0x80000AAA] = 0x80
 * 0x002A F0 70                     movhu d0, (a0)              ; Cycle 4: [0x80000AAA] = 0xAA
 * 0x002C F0 75                     movhu d1, (a1)              ; Cycle 5: [0x80000554] = 0x55
 * 0x002E F0 7B                     movhu d3, (a2)              ; Cycle 6: [sector_ptr] = 0x30
 *
 * poll_dq7:
 * 0x0030 F0 68                     movhu (a2), d0              ; Read flash status register
 * 0x0032 F5 00 80                  btst 0x0080, d0             ; Test DQ7 Data# Polling bit
 * 0x0035 C8 F9                     beq poll_dq7                ; If DQ7 == 0 (busy), loop
 *
 * reset_read_mode:
 * 0x0037 2C F0 00                  mov 0x00F0, d0              ; d0 = JEDEC Read Reset Command
 * 0x003A F0 78                     movhu d0, (a2)              ; [sector_ptr] = 0xF0 (Read Mode)
 *
 * next_sector:
 * 0x003C FC 28 00 20 00 00         add 0x00002000, a2          ; a2 += 8 KB (Next micro-sector)
 * 0x0042 FC 58 00 00 80 80         cmp 0x80800000, a2          ; Reached 0x80800000?
 * 0x0048 C0 CE                     bne reload_d0_sector_loop   ; If not all erased, loop (reloads d0=0x00AA!)
 *
 * finish_trap:
 * 0x004A FD                        pi                          ; Program Interrupt (OCD Breakpoint)
 *
 * halt_loop:
 * 0x004B 97                        nop                         ; Safety halt loop
 * 0x004C EA FD                     bra halt_loop               ; Infinite branch to self
 * 0x004E 97 97                     nop, nop                    ; Alignment padding to 80 bytes
 * ========================================================================= */

/* =========================================================================
 * MANDATORY SAFETY ADVISORY (READ-ONLY FIRST PROTOCOL):
 *
 * DO NOT execute or inject this payload until:
 * 1. A complete 8 MB read-only memory dump has been performed twice via
 *    the DWire32A OCD 'DUMP' command and verified offline.
 * 2. The native bootloader disassembly has been analyzed with GNU binutils
 *    (mn10300-elf-objdump) to confirm exact branch offset conventions and
 *    opcode semantics for the Matsushita MN2DS0016 / AM34 core.
 * 3. The exact boundaries of the EEMUL partition have been verified in the dump.
 * ========================================================================= */

static const uint8_t am34_ram_loader_bin[AM34_RAM_LOADER_SIZE_BYTES] = {
    /* 0x0000: mov 0x80000AAA, a0 */
    0xFC, 0x80, 0xAA, 0x0A, 0x00, 0x80,
    /* 0x0006: mov 0x80000554, a1 */
    0xFC, 0x84, 0x54, 0x05, 0x00, 0x80,
    /* 0x000C: mov 0x807F0000, a2 */
    0xFC, 0x88, 0x00, 0x00, 0x7F, 0x80,
    /* 0x0012: mov 0x80800000, a3 */
    0xFC, 0x8C, 0x00, 0x00, 0x80, 0x80,
    /* 0x0018: mov 0x00AA, d0 */
    0x2C, 0xAA, 0x00,
    /* 0x001B: mov 0x0055, d1 */
    0x2D, 0x55, 0x00,
    /* 0x001E: mov 0x0080, d2 */
    0x2E, 0x80, 0x00,
    /* 0x0021: mov 0x0030, d3 */
    0x2F, 0x30, 0x00,

    /* 0x0024: sector_loop: 6-cycle JEDEC Sector Erase */
    0xF0, 0x70,  /* movhu d0, (a0) -> Unlock 1 */
    0xF0, 0x75,  /* movhu d1, (a1) -> Unlock 2 */
    0xF0, 0x72,  /* movhu d2, (a0) -> Erase Setup */
    0xF0, 0x70,  /* movhu d0, (a0) -> Unlock 1 */
    0xF0, 0x75,  /* movhu d1, (a1) -> Unlock 2 */
    0xF0, 0x7B,  /* movhu d3, (a2) -> Sector Erase Trigger */

    /* 0x0030: poll_dq7: DQ7 Data# Polling */
    0xF0, 0x68,        /* movhu (a2), d0 */
    0xF5, 0x00, 0x80,  /* btst  0x0080, d0 */
    0xC8, 0xF9,        /* beq   poll_dq7 */

    /* 0x0037: reset_read_mode: Reset array to Read Mode */
    0x2C, 0xF0, 0x00,  /* mov   0x00F0, d0 */
    0xF0, 0x78,        /* movhu d0, (a2) */

    /* 0x003C: next_sector: Advance to next 8 KB micro-sector */
    0xFC, 0x28, 0x00, 0x20, 0x00, 0x00,  /* add 0x00002000, a2 */
    0xFC, 0x58, 0x00, 0x00, 0x80, 0x80,  /* cmp 0x80800000, a2 */
    0xC0, 0xCE,                          /* bne reload_d0 (0x0018: reloads d0=0x00AA!) */

    /* 0x004A: finish_trap: Trigger OCD Breakpoint */
    0xFD,                                /* pi (Program Interrupt) */

    /* 0x004B: halt_loop: Infinite loop */
    0x97,                                /* nop */
    0xEA, 0xFD,                          /* bra halt_loop (-3) */
    0x97, 0x97                           /* nop, nop (padding to 80 bytes) */
};

#include "flash/dwire32a_flash.h"

#ifdef __cplusplus
}
#endif

#endif /* RAM_LOADER_PAYLOAD_H */
