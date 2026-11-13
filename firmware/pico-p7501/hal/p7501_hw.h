/**
 * @file p7501_hw.h
 * @brief Hardware Abstraction Layer (HAL) for Panasonic DMR-EH55 P7501 interface.
 *
 * Electrical Safety Guarantees:
 *   - GP3 (Pin 5 NSW5V / X_SW_5R2V) is strictly locked as Hi-Z Input (never driven).
 *   - GP5 (Pin 2 XRESET) is open-drain active-LOW only (never driven push-pull 3.3V).
 *   - GP2 (Pin 6 UARTP2M_T) is always input (115200 8N1 RX from SoC).
 *   - GP26 (Pin 1 SW_UART / ADC0) clears SIO OE before switching back to Hi-Z ADC0.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_HW_H
#define P7501_HW_H

#include "p7501_types.h"
#include "protocol/p7501_bus_protocol.h"

#ifdef __cplusplus
extern "C" {
#endif

/** Initialize ADC0, safe Hi-Z P7501 GPIOs, and passive PIO UART RX state machine. */
p7501_status_t p7501_hw_init(void);

/** Assert XRESET (GP5) active LOW via Open-Drain emulation (0V output). */
void p7501_hw_reset_assert(void);

/** Release XRESET (GP5) back to High-Z Input with internal pulls disabled. */
void p7501_hw_reset_release(void);

/** Safely control GP26 (Pin 1 SW_UART): LOW outputs 0V to enter XFWE serial boot; false restores Hi-Z ADC0. */
void p7501_hw_sw_uart_set_low(bool assert_low);

/** Restore GP26 cleanly from SIO digital mode back to High-Z ADC0 without sticky OE latch. */
void p7501_hw_restore_sw_uart_adc0(void);

/** Sample Pin 1 (SW_UART / GP26) voltage via ADC0 with selectable internal pull state. */
float p7501_hw_sample_sw_uart_volts(bool pull_down, bool pull_up);

/** Sample Pin 1 (SW_UART / GP26) voltage via ADC0 with active pull-down discrimination + Hi-Z read. */
float p7501_hw_read_vdd(void);

/** Check whether P7501 Pin 1 voltage exceeds P7501_VDD_THRESHOLD_V (2.5V). */
bool p7501_hw_is_target_powered(void);

/** Check whether P7501 hardware HAL has been initialized. */
bool p7501_hw_is_initialized(void);

/** Execute a discrete P7501 SW_UART + XRESET boot/reset profile (Modes 0..3). */
void p7501_hw_boot_sequence(p7501_boot_mode_t mode);

/** Place all 5 digital/analog P7501 pins safely into High-Z Isolation (NSW5V Hi-Z, XRESET Hi-Z, GP26 ADC0). */
void p7501_hw_isolate_bus_hiz(void);

/** Engage P7501 TX pin (GP4 UARTM2P_T) idle HIGH for active UART transmission, keeping NSW5V (GP3) Hi-Z. */
void p7501_hw_engage_uart_bus(void);

#ifdef __cplusplus
}
#endif

#endif /* P7501_HW_H */
