/**
 * @file ram_loader_am34_target.c
 * @brief Autonomous AM34 / MN103S position-independent SRAM payload.
 *
 * Injected into internal SRAM via P7501 / AM34 monitor transport
 * to erase the corrupted NVRAM emulation partition (EEMUL) on the Spansion
 * S29JL064H70TFI00 NOR Flash, clearing the DMR-EH55 U99 fatal state.
 *
 * Target Processor: Matsushita UniPhier / AM34 / MN103S
 * Target Hardware:  Panasonic DMR-EH55 Digital Board (VEP79138K)
 *
 * Compilation:
 *   mn10300-elf-gcc -mam33-2 -std=c11 -Os -fPIC -nostdlib -c ram_loader.c -o ram_loader.o
 *   mn10300-elf-objcopy -O binary -j .text.entry -j .text ram_loader.o ram_loader.bin
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include <stdint.h>
#include <stdbool.h>

/* =========================================================================
 * Physical Memory Bus Architecture
 * ========================================================================= */

/**
 * @brief Direct Physical Uncached Region Base Address.
 *
 * CRITICAL ARCHITECTURAL REQUIREMENT:
 * The AM34 architecture caches accesses to the mirror at 0x00000000.
 * In order for JEDEC command cycles to be driven out to external memory
 * pins (CS0/SB0), all flash register accesses MUST target the physical
 * uncached aperture starting at 0x80000000.
 */
#define FLASH_UNCACHED_BASE         0x80000000UL

/**
 * @brief JEDEC Word-Mode Byte-Bus Address Offsets.
 *
 * The Spansion S29JL064H operates in 16-bit word mode (BYTE# = High).
 * In 16-bit mode, standard JEDEC address cycles (0x555 and 0x2AA) are shifted
 * left by 1 bit on the CPU byte-addressed address bus:
 *   Addr1: 0x555 << 1 = 0x0AAA
 *   Addr2: 0x2AA << 1 = 0x0554
 */
#define JEDEC_UNLOCK_ADDR1_OFFSET   0x0AAAUL
#define JEDEC_UNLOCK_ADDR2_OFFSET   0x0554UL

/* =========================================================================
 * JEDEC Standard Flash Command Definitions (16-bit Word Mode)
 * ========================================================================= */

#define JEDEC_CMD_AA                0x00AAU /**< Unlock cycle 1 and 4 command */
#define JEDEC_CMD_55                0x0055U /**< Unlock cycle 2 and 5 command */
#define JEDEC_CMD_ERASE_SETUP       0x0080U /**< Erase setup command */
#define JEDEC_CMD_SECTOR_ERASE      0x0030U /**< Sector erase initiate command */
#define JEDEC_CMD_READ_RESET        0x00F0U /**< Reset array to Read Mode */

/* =========================================================================
 * Hardware Status Polling Bitmasks (Data# Polling DQ7 / Exceeded Timing DQ5)
 * ========================================================================= */

#define JEDEC_POLL_DQ7_MASK         0x0080U /**< Data# Polling: Inverted data during erase; 1 when done */
#define JEDEC_ERROR_DQ5_MASK        0x0020U /**< Exceeded Timing Limit flag (timeout error) */

/* =========================================================================
 * Spansion S29JL064H Top-Boot Partition Boundaries (EEMUL Region)
 * ========================================================================= */

/** Base address of the 8 x 8 KB parameter micro-sectors (SA134 to SA141). */
#define SPANSION_NVRAM_BASE_ADDR    0x007F0000UL

/** Upper limit boundary of the 64 Mbit (8 MB) flash memory space. */
#define SPANSION_NVRAM_END_ADDR     0x00800000UL

/** Micro-sector size stride (8 KB per sector). */
#define SPANSION_MICROSECTOR_STRIDE 0x00002000UL

/* =========================================================================
 * Hardware Access Macros
 * ========================================================================= */

#define FLASH_REG16(addr) (*(volatile uint16_t *)((uintptr_t)(addr)))
#define JEDEC_REG_UNLOCK1 FLASH_REG16(FLASH_UNCACHED_BASE + JEDEC_UNLOCK_ADDR1_OFFSET)
#define JEDEC_REG_UNLOCK2 FLASH_REG16(FLASH_UNCACHED_BASE + JEDEC_UNLOCK_ADDR2_OFFSET)

/* =========================================================================
 * JEDEC Sector Erase Algorithm
 * ========================================================================= */

/**
 * @brief Erase a specific sector on Spansion S29JL064H and poll DQ7 for completion.
 *
 * Implements the standard 6-cycle JEDEC sector erase sequence followed by
 * DQ7 Data# Polling with DQ5 timeout monitoring.
 *
 * @param sector_offset Relative byte offset of sector within flash (e.g. 0x007F0000).
 * @return true if sector erased successfully, false on timeout/hardware fault.
 */
static bool spansion_erase_sector(uint32_t sector_offset) {
    volatile uint16_t *sector_ptr =
        (volatile uint16_t *)((uintptr_t)(FLASH_UNCACHED_BASE + sector_offset));

    /* 6-cycle JEDEC Sector Erase command sequence in 16-bit word mode */
    JEDEC_REG_UNLOCK1 = JEDEC_CMD_AA;
    JEDEC_REG_UNLOCK2 = JEDEC_CMD_55;
    JEDEC_REG_UNLOCK1 = JEDEC_CMD_ERASE_SETUP;
    JEDEC_REG_UNLOCK1 = JEDEC_CMD_AA;
    JEDEC_REG_UNLOCK2 = JEDEC_CMD_55;
    *sector_ptr       = JEDEC_CMD_SECTOR_ERASE;

    /*
     * Poll DQ7 Data# Polling:
     * While erase is in progress, DQ7 outputs 0.
     * When erase completes, the sector is 0xFFFF and DQ7 outputs 1.
     */
    while ((*sector_ptr & JEDEC_POLL_DQ7_MASK) == 0U) {
        /* Check if DQ5 error flag is asserted (internal erase timeout) */
        if ((*sector_ptr & JEDEC_ERROR_DQ5_MASK) != 0U) {
            /* Re-check DQ7 to avoid false error on boundary completion */
            if ((*sector_ptr & JEDEC_POLL_DQ7_MASK) == 0U) {
                *sector_ptr = JEDEC_CMD_READ_RESET;
                return false; /* Hardware erase error */
            }
            break;
        }
    }

    /* Restore flash to normal array read mode */
    *sector_ptr = JEDEC_CMD_READ_RESET;
    return true;
}

/* =========================================================================
 * RAM Loader Entry Point
 * ========================================================================= */

/**
 * @brief Execution entry point invoked from AM34 SRAM.
 *
 * Sequentially erases the 8 micro-sectors of the EEMUL partition (0x7F0000 - 0x7FFFFF)
 * and triggers a Program Interrupt (pi) breakpoint instruction to return control.
 */
void __attribute__((section(".text.entry"), noreturn)) _start(void) {
    /* Erase all 8 corrupted NVRAM micro-sectors (SA134 to SA141) */
    for (uint32_t offset = SPANSION_NVRAM_BASE_ADDR;
         offset < SPANSION_NVRAM_END_ADDR;
         offset += SPANSION_MICROSECTOR_STRIDE) {
        (void)spansion_erase_sector(offset);
    }

    /*
     * Signal completion to the host debugger.
     * "pi" is the native AM33/AM34 Program Interrupt / Software Breakpoint instruction.
     */
    __asm__ volatile ("pi");

    /* Lock core in infinite halt loop if resumed */
    while (1) {
        __asm__ volatile ("nop");
    }
}
