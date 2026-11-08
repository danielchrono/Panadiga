/**
 * @file dwire32a_led.h
 * @brief Onboard LED status and heartbeat driver for Raspberry Pi Pico (RP2040 & RP2350).
 *
 * Implements non-blocking visual telemetry:
 *   - Mode 0: Heartbeat pulse (1 Hz, 100ms on, 900ms off) -> Firmware active & USB CDC ready
 *   - Mode 1: Active DWire32A communication (Solid ON)   -> Bus transaction / Break-In / Scan
 *   - Mode 2: GDB RSP session active (5 Hz flicker / packet toggle) -> Active debug stepping
 *   - Mode 3: Bus fault / low voltage warning (Triple strobe) -> Target VDD < 3.0V or short
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef DWIRE32A_LED_H
#define DWIRE32A_LED_H

#include <stdint.h>
#include <stdbool.h>

#ifndef PICO_DEFAULT_LED_PIN
#define PICO_DEFAULT_LED_PIN 25U
#endif

typedef enum {
    LED_MODE_HEARTBEAT_IDLE = 0, /**< 1 Hz pulse: Firmware idle, awaiting command */
    LED_MODE_DWIRE_ACTIVE   = 1, /**< Solid ON: Bus transaction or scan in progress */
    LED_MODE_GDB_RSP        = 2, /**< 5 Hz toggle on packet: GDB/Ghidra debug session */
    LED_MODE_BUS_FAULT      = 3, /**< Rapid triple blink: VDD low (< 3.0V) or short */
} pico_led_mode_t;

/**
 * @brief Initialize the onboard LED GPIO pin.
 */
void dwire_led_init(void);

/**
 * @brief Set the active operational LED visual mode.
 *
 * @param mode The desired LED mode.
 */
void dwire_led_set_mode(pico_led_mode_t mode);

/**
 * @brief Notify the LED driver of an instantaneous bus activity event (packet toggle).
 */
void dwire_led_notify_activity(void);

/**
 * @brief Periodic non-blocking tick to update LED state machine.
 */
void dwire_led_tick(void);

#endif /* DWIRE32A_LED_H */
