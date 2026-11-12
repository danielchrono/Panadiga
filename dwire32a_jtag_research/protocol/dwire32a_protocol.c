/**
 * @file dwire32a_protocol.c
 * @brief Matsushita AM34 / MN103S On-Chip Debugger (DWire32A) protocol implementation.
 *
 * Encodes 8-bit OCD opcodes and 32-bit address/data operands onto the
 * half-duplex PIO DWire32A transceiver.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "protocol/dwire32a_protocol.h"
#include "hal/dwire32a_hw.h"
#include <stddef.h>

void dwire_hw_write_word32(uint32_t word) {
    dwire_hw_write_bits(word, 32U);
}

uint32_t dwire_hw_read_word32(void) {
    return dwire_hw_read_bits(32U);
}

void dwire_hw_write_mem32(uint32_t addr, uint32_t data) {
    dwire_hw_write_bits((uint32_t)DWIRE_OCD_CMD_WRITE_MEM32, DWIRE_OPCODE_BIT_LEN);
    dwire_hw_write_bits(addr, 32U);
    dwire_hw_write_bits(data, 32U);
}

uint32_t dwire_hw_read_mem32(uint32_t addr) {
    dwire_hw_write_bits((uint32_t)DWIRE_OCD_CMD_READ_MEM32, DWIRE_OPCODE_BIT_LEN);
    dwire_hw_write_bits(addr, 32U);
    return dwire_hw_read_bits(32U);
}

void dwire_hw_set_pc(uint32_t pc_val) {
    dwire_hw_write_bits((uint32_t)DWIRE_OCD_CMD_WRITE_PC, DWIRE_OPCODE_BIT_LEN);
    dwire_hw_write_bits(pc_val, 32U);
}

void dwire_hw_resume_cpu(void) {
    dwire_hw_write_bits((uint32_t)DWIRE_OCD_CMD_RESUME, DWIRE_OPCODE_BIT_LEN);
}

uint32_t dwire_hw_read_dcsr(void) {
    dwire_hw_write_bits((uint32_t)DWIRE_OCD_CMD_READ_DCSR, DWIRE_OPCODE_BIT_LEN);
    return dwire_hw_read_bits(32U);
}

bool dwire_hw_is_cpu_halted(void) {
    const uint32_t dcsr = dwire_hw_read_dcsr();
    /* Open bus returns 0xFFFFFFFF (pull-up) or 0x00000000 (ground) */
    if (dcsr == DWIRE_BUS_RESP_OPEN_HIGH || dcsr == DWIRE_BUS_RESP_OPEN_LOW) {
        return false;
    }
    /* Bit 0 is the CPU Halt status flag in AM34 DCSR register */
    return ((dcsr & 0x00000001UL) != 0U);
}

bool dwire_proto_execute_breakin_handshake(uint32_t *out_dcsr, uint32_t *out_mode) {
    uint32_t dcsr = DWIRE_BUS_RESP_OPEN_HIGH;
    bool halted = false;
    uint32_t matched_mode = 0U;

    for (uint32_t mode = 0U; mode < (uint32_t)DWIRE_BREAKIN_MODE_COUNT; ++mode) {
        dwire_hw_cold_break_in((dwire_breakin_mode_t)mode);
        dcsr = dwire_hw_read_dcsr();
        if (dcsr != DWIRE_BUS_RESP_OPEN_HIGH &&
            dcsr != DWIRE_BUS_RESP_OPEN_LOW &&
            ((dcsr & 0x00000001UL) != 0U)) {
            halted = true;
            matched_mode = mode;
            break;
        }
    }

    if (out_dcsr != NULL) {
        *out_dcsr = dcsr;
    }
    if (out_mode != NULL) {
        *out_mode = matched_mode;
    }
    return halted;
}
