/**
 * @file dwire32a_hw.h
 * @brief Hardware Abstraction Layer (HAL) for Panasonic DWire32A interface.
 *
 * Provides open-drain reset control, calibrated VDD sense via ADC0,
 * and low-jitter half-duplex synchronous bit streaming via RP2040 PIO.
 *
 * Target: Panasonic DMR-EH55 (MN2DS0016 / AM34)
 * Host:   Raspberry Pi Pico (RP2040)
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef DWIRE32A_HW_H
#define DWIRE32A_HW_H

#include "dwire32a_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Initialize all hardware subsystems: GPIOs, ADC0, and PIO state machine.
 *
 * Configures:
 * - nRESET as GPIO Input (Hi-Z, inactive open-drain).
 * - EXTRG as GPIO Output (Low).
 * - ADC0 on GP26 for VDD sensing.
 * - PIO0 State Machine 0 for DWire32A half-duplex synchronous communication.
 *
 * @return DWIRE_OK on success, or negative error code on failure.
 */
dwire_status_t dwire_hw_init(void);

/**
 * @brief Assert hardware reset to target using open-drain emulation.
 *
 * CRITICAL SAFETY NOTE:
 * The line is driven strictly to 0V (GND) by configuring GP5 as output low.
 * GP5 is NEVER driven high (3.3V push-pull) to avoid bus contention with
 * the Panasonic onboard reset supervisor IC.
 */
void dwire_hw_reset_assert(void);

/**
 * @brief Release hardware reset to target (return to High-Z).
 *
 * Configures GP5 as input (Hi-Z), allowing the board's onboard pull-up
 * resistor to smoothly pull nRESET up to target VDD (3.3V).
 */
void dwire_hw_reset_release(void);

/**
 * @brief Set the logical state of the EXTRG (External Trigger) pin.
 *
 * @param level True to drive 3.3V (Logic 1), False to drive 0V (Logic 0).
 */
void dwire_hw_extrg_set(bool level);

/**
 * @brief Sample the target VDD voltage rail via ADC0 (GP26).
 *
 * Measures target reference voltage to verify board power status.
 *
 * @return Measured voltage in Volts (0.0V to 3.3V).
 */
float dwire_hw_read_vdd(void);

/**
 * @brief Check whether target board is powered and exceeding threshold voltage.
 *
 * @return true if VDD >= DWIRE_VDD_THRESHOLD_V, false otherwise.
 */
bool dwire_hw_is_target_powered(void);

/**
 * @brief Check whether the DWire32A hardware layer has been successfully initialized.
 *
 * @return true if initialized, false otherwise.
 */
bool dwire_hw_is_initialized(void);

/**
 * @brief Transmit n_bits from master to target via PIO state machine.
 *
 * Generates exact SCLK clock pulses at DWIRE_BUS_FREQ_HZ (1 MHz) while
 * shifting out bits onto SDATA with setup/hold timing.
 *
 * @param data    Data word to transmit (shifted out LSB first).
 * @param n_bits  Number of bits to send (1 to 32).
 */
void dwire_hw_write_bits(uint32_t data, uint8_t n_bits);

/**
 * @brief Receive n_bits from target to master via PIO state machine.
 *
 * Generates exact SCLK clock pulses at DWIRE_BUS_FREQ_HZ (1 MHz) while
 * keeping SDATA in Hi-Z (input) and sampling on falling edges.
 *
 * @param n_bits  Number of bits to receive (1 to 32).
 * @return 32-bit word containing the received data.
 */
uint32_t dwire_hw_read_bits(uint8_t n_bits);

/**
 * @brief Execute the Cold Break-In sequence to halt the CPU out of reset.
 *
 * Halts the MN2DS0016 / AM34 core before bootloader execution by coordinating
 * nRESET de-assertion, EXTRG trigger polarity, and synchronous SCLK clocking.
 *
 * @param mode Break-in profile from enum dwire_breakin_mode.
 */
void dwire_hw_cold_break_in(dwire_breakin_mode_t mode);

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

#ifdef __cplusplus
}
#endif

#endif /* DWIRE32A_HW_H */
