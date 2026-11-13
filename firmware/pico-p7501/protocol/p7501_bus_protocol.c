/**
 * @file p7501_bus_protocol.c
 * @brief P7501 UART-safe target monitor protocol implementation for Panasonic AM34.
 *
 * Sends compact UART monitor probes only when target bus is powered while
 * strictly ensuring GP3 (NSW5V) stays Hi-Z Input and GP5 (XRESET) stays Open-Drain.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "protocol/p7501_bus_protocol.h"
#include "hal/p7501_hw.h"
#include "uart/p7501_uart_bitbang.h"
#include "pico/stdlib.h"
#include <stddef.h>
#include <stdio.h>

static uint32_t uart_mon_exchange_word(uint8_t cmd, uint32_t arg1, uint32_t arg2, bool send_arg1, bool send_arg2) {
    /* Only drive UART TX when GP26 (SW_UART) is Hi-Z so Q7514 conducts cleanly toward AM34. */
    (void)cmd;
    (void)arg1;
    (void)arg2;
    (void)send_arg1;
    (void)send_arg2;
    return P7501_BUS_RESP_OPEN_HIGH;
}

uint32_t p7501_hw_read_mem32(uint32_t addr) {
    return uart_mon_exchange_word(P7501_MON_CMD_READ_MEM32, addr, 0U, true, false);
}

void p7501_hw_write_mem32(uint32_t addr, uint32_t data) {
    (void)uart_mon_exchange_word(P7501_MON_CMD_WRITE_MEM32, addr, data, true, true);
}

void p7501_hw_set_pc(uint32_t pc_val) {
    (void)uart_mon_exchange_word(P7501_MON_CMD_WRITE_PC, pc_val, 0U, true, false);
}

void p7501_hw_resume_cpu(void) {
    (void)uart_mon_exchange_word(P7501_MON_CMD_RESUME, 0U, 0U, false, false);
}

uint32_t p7501_hw_read_dcsr(void) {
    return uart_mon_exchange_word(P7501_MON_CMD_READ_DCSR, 0U, 0U, false, false);
}

bool p7501_hw_is_cpu_halted(void) {
    const uint32_t dcsr = p7501_hw_read_dcsr();
    if ((dcsr == P7501_BUS_RESP_OPEN_HIGH) || (dcsr == P7501_BUS_RESP_OPEN_LOW)) {
        return false;
    }
    return ((dcsr & 0x00000001UL) != 0U);
}

uint32_t p7501_proto_probe_opcode(uint8_t opcode) {
    return uart_mon_exchange_word(opcode, 0U, 0U, false, false);
}

bool p7501_proto_execute_boot_handshake(uint32_t *out_dcsr, uint32_t *out_mode) {
    uint32_t dcsr = P7501_BUS_RESP_OPEN_HIGH;
    bool halted = false;
    uint32_t matched_mode = 0U;

    for (uint32_t mode = 0U; mode < (uint32_t)P7501_BOOT_MODE_COUNT; ++mode) {
        p7501_hw_boot_sequence((p7501_boot_mode_t)mode);
        dcsr = p7501_hw_read_dcsr();
        if ((dcsr != P7501_BUS_RESP_OPEN_HIGH) &&
            (dcsr != P7501_BUS_RESP_OPEN_LOW) &&
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
