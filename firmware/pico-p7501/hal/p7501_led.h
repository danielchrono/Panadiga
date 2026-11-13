/**
 * @file p7501_led.h
 * @brief Onboard LED status and heartbeat driver for Raspberry Pi Pico (RP2040 & RP2350).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_LED_H
#define P7501_LED_H

#include <stdint.h>
#include <stdbool.h>

#ifndef PICO_DEFAULT_LED_PIN
#define PICO_DEFAULT_LED_PIN 25U
#endif

typedef enum {
    LED_MODE_HEARTBEAT_IDLE = 0, /**< 1 Hz pulse: Firmware idle, awaiting command */
    LED_MODE_BUS_ACTIVE     = 1, /**< Solid ON: UART capture or bus diagnostic in progress */
    LED_MODE_GDB_RSP        = 2, /**< 5 Hz toggle on packet: GDB/Ghidra debug session */
    LED_MODE_BUS_FAULT      = 3, /**< Rapid triple blink: VDD low (< 2.5V) or short */
} pico_led_mode_t;

void p7501_led_init(void);
void p7501_led_set_mode(pico_led_mode_t mode);
void p7501_led_notify_activity(void);
void p7501_led_tick(void);

#endif /* P7501_LED_H */
