/**
 * @file p7501_bus_protocol.h
 * @brief Matsushita AM34 / MN103S P7501 debug/monitor transaction primitives.
 *
 * Layered over `hal/p7501_hw.h` while strictly preserving P7501 electrical
 * safety (GP3 NSW5V kept Hi-Z Input; GP5 XRESET open-drain active-LOW;
 * GP4 TXD / GP2 RXD at 115200 8N1).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_BUS_PROTOCOL_H
#define P7501_BUS_PROTOCOL_H

#include "p7501_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/** Probe/read a 32-bit AM34 monitor word via P7501 UART transport. */
uint32_t p7501_hw_read_mem32(uint32_t addr);

/** Write a 32-bit AM34 word via P7501 UART monitor transport (protected by IPL air-gap). */
void p7501_hw_write_mem32(uint32_t addr, uint32_t data);

/** Set AM34 CPU Program Counter (PC) via monitor transport. */
void p7501_hw_set_pc(uint32_t pc_val);

/** Resume AM34 core execution via monitor transport. */
void p7501_hw_resume_cpu(void);

/** Query target debug/monitor control status register (returns open-bus high when idle UART). */
uint32_t p7501_hw_read_dcsr(void);

/** Check whether AM34 target monitor reports halted status (Bit 0 set). */
bool p7501_hw_is_cpu_halted(void);

/** Probe a raw monitor opcode across P7501 UART transport and capture 32-bit reply. */
uint32_t p7501_proto_probe_opcode(uint8_t opcode);

/** Sweep P7501 discrete boot profiles (0..3) and inspect resulting monitor state. */
bool p7501_proto_execute_boot_handshake(uint32_t *out_dcsr, uint32_t *out_mode);

#ifdef __cplusplus
}
#endif

#endif /* P7501_BUS_PROTOCOL_H */
