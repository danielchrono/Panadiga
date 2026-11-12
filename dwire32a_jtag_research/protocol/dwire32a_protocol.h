/**
 * @file dwire32a_protocol.h
 * @brief Matsushita AM34 / MN103S On-Chip Debugger (DWire32A) bus protocol primitives.
 *
 * Implements high-level word, memory, register, and Cold Break-In handshake
 * transactions layered above the RP2040/RP2350 PIO bit transceiver HAL.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef DWIRE32A_PROTOCOL_H
#define DWIRE32A_PROTOCOL_H

#include "dwire32a_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Transmit a 32-bit word to the target via DWire32A.
 *
 * @param word 32-bit data word.
 */
void dwire_hw_write_word32(uint32_t word);

/**
 * @brief Receive a 32-bit word from the target via DWire32A.
 *
 * @return 32-bit received data word.
 */
uint32_t dwire_hw_read_word32(void);

/**
 * @brief Write a 32-bit word to target memory at specified address via DWire32A OCD.
 *
 * @param addr 32-bit target physical address.
 * @param data 32-bit data to write.
 */
void dwire_hw_write_mem32(uint32_t addr, uint32_t data);

/**
 * @brief Read a 32-bit word from target memory at specified address via DWire32A OCD.
 *
 * @param addr 32-bit target physical address.
 * @return 32-bit data read from target memory.
 */
uint32_t dwire_hw_read_mem32(uint32_t addr);

/**
 * @brief Set the AM34 CPU Program Counter (PC) via DWire32A OCD.
 *
 * @param pc_val Target execution address.
 */
void dwire_hw_set_pc(uint32_t pc_val);

/**
 * @brief Resume CPU core execution from current PC.
 */
void dwire_hw_resume_cpu(void);

/**
 * @brief Read the AM34 On-Chip Debugger Control and Status Register (DCSR).
 *
 * @return 32-bit DCSR register value.
 */
uint32_t dwire_hw_read_dcsr(void);

/**
 * @brief Check whether the AM34 CPU is currently in OCD Halt state.
 *
 * @return true if core is halted, false if running or bus floating.
 */
bool dwire_hw_is_cpu_halted(void);

/**
 * @brief Execute multi-mode Cold Break-In sweep until AM34 OCD responds on DCSR.
 *
 * Evaluates all configured Cold Break-In polarity/burst profiles (0..3) and
 * captures the last observed DCSR and matched profile index.
 *
 * @param[out] out_dcsr Optional pointer receiving final DCSR register value.
 * @param[out] out_mode Optional pointer receiving matched break-in profile mode.
 * @return true if CPU entered OCD Halt (DCSR bit 0 set and bus driven), false otherwise.
 */
bool dwire_proto_execute_breakin_handshake(uint32_t *out_dcsr, uint32_t *out_mode);

#ifdef __cplusplus
}
#endif

#endif /* DWIRE32A_PROTOCOL_H */
