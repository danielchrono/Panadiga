/**
 * @file p7501_led.h
 * @brief Hardware LED driver for Pico RP2040/RP2350 (GP25 GPIO or CYW43 WL_GPIO0).
 *
 * Manual user selections (`p7501_led_set_mode`) lock the LED against automatic
 * background overwrites (`p7501_led_set_auto_mode`) caused by 2-second Desktop
 * `STATUS` polling until `p7501_led_unlock_auto` (`LED AUTO`) is invoked.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_LED_H
#define P7501_LED_H

#include "hal/p7501_led_logic.h"

#ifdef __cplusplus
extern "C" {
#endif

#ifndef PICO_DEFAULT_LED_PIN
#define PICO_DEFAULT_LED_PIN 25U
#endif

/** Initialize onboard Pico LED hardware and start 10 Hz non-blocking timer tick. */
void p7501_led_init(void);

/** Manually lock the onboard LED to `mode` and apply physical pin state immediately. */
void p7501_led_set_mode(pico_led_mode_t mode);

/** Update automatic LED mode unless manual user lock is active. */
void p7501_led_set_auto_mode(pico_led_mode_t mode);

/** Clear manual user lock and restore automatic mode selection. */
void p7501_led_unlock_auto(pico_led_mode_t fallback_mode);

/** Toggle LED output immediately on packet or bus activity (when unlocked or in GDB mode). */
void p7501_led_notify_activity(void);

/** Periodic 10 Hz tick hook (also driven by repeating timer on non-CYW43 targets). */
void p7501_led_tick(void);

/** Sleep for `delay_ms` while keeping LED pattern responsive on all targets (incl. CYW43). */
void p7501_led_sleep_ms(uint32_t delay_ms);

/** Snapshot current LED mode, manual lock state, and physical output pin level. */
p7501_led_state_t p7501_led_get_state(void);

/** Describe physical LED pin backend (`GP25` or `CYW43_WL_GPIO0`). */
const char *p7501_led_backend_label(void);

#ifdef __cplusplus
}
#endif

#endif /* P7501_LED_H */
