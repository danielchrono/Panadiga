/**
 * @file dwire32a_hw.h
 * @brief Hardware Abstraction Layer (HAL) for Panasonic DWire32A interface.
 *
 * Provides open-drain reset control, calibrated VDD sense via ADC0,
 * and low-jitter half-duplex synchronous bit streaming via RP2040 PIO.
 *
 * Target: Panasonic UniPhier / Matsushita AM34 / MN103S SoC Family (MN2DS0004 / MN2DS0009 / MN2DS0016 / MN2DS0018)
 * Models: Panasonic DIGA Series (DMR-E / DMR-ExxH / DMR-ES / DMR-EH / DMR-EX / DMR-EZ)
 * Host:   Raspberry Pi Pico (RP2040 / RP2350)
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

/**
 * @brief Detach SCLK and SDATA pins from PIO and configure as floating inputs for sniffing.
 */
void dwire_hw_detach_pins_for_sniff(void);

/**
 * @brief Re-attach SCLK and SDATA pins to the PIO state machine after sniffing.
 */
void dwire_hw_attach_pins_after_sniff(void);

/**
 * @brief Place all 6 connector pins (EXTRG, SCLK, SDATA, nRESET) into High-Impedance (Hi-Z Input, no pulls).
 *
 * Allows the 1-piece 6-pin JST ZR-6 (B6B-ZR-3R4) plug to remain physically connected
 * to P7501/CN55001 at all times without loading or interfering with normal recorder boot.
 */
void dwire_hw_isolate_bus_hiz(void);

/**
 * @brief Restore active PIO / OCD drive on the 6-pin connector from High-Impedance isolation.
 */
void dwire_hw_engage_bus_active(void);

/**
 * @brief Auto-detect target NOR Flash geometry (2 MB, 4 MB, 8 MB, or 16 MB) via
 *        100% passive read-only Address Aliasing on the AM34 aperture (0x80000000).
 *
 * Compares the Bootloader vector fingerprint at 0x80000000 against +2 MB (0x80200000),
 * +4 MB (0x80400000), and +8 MB (0x80800000) to detect the highest decoded address line
 * without issuing any write cycles to the Flash array.
 *
 * @return Detected NOR Flash size in bytes (0x00200000, 0x00400000, 0x00800000, or 0x01000000).
 */
uint32_t dwire_hw_autodetect_flash_size(void);

/**
 * @brief Query Common Flash Interface (JEDEC CFI Query 0x98) to read laser-etched geometry.
 *
 * Sends command 0x0098 to offset 0xAA, inspects the 'QRY' header, extracts the 2^N size
 * exponent, and immediately resets the array to normal Read Mode with 0x00F0.
 *
 * @param out_size_bytes Pointer to receive detected size in bytes (or NULL).
 * @return true if valid 'QRY' signature and size exponent were decoded, false otherwise.
 */
bool dwire_hw_query_cfi(uint32_t *out_size_bytes);

#ifdef __cplusplus
}
#endif

#endif /* DWIRE32A_HW_H */
