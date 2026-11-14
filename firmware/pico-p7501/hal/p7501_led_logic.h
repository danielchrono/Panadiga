/**
 * @file p7501_led_logic.h
 * @brief Pure C11 state machine for Pico onboard LED patterns and manual override lock.
 *
 * Host-testable with zero Pico SDK dependencies (`PICO_HOST_TEST=1`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_LED_LOGIC_H
#define P7501_LED_LOGIC_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    LED_MODE_HEARTBEAT_IDLE = 0, /**< 1 Hz pulse (200ms ON / 800ms OFF) */
    LED_MODE_BUS_ACTIVE     = 1, /**< Solid ON: active bus capture or transaction */
    LED_MODE_GDB_RSP        = 2, /**< 5 Hz fast blink: GDB RSP or UART bridge */
    LED_MODE_BUS_FAULT      = 3, /**< Rapid triple blink: VDD low / fault condition */
    LED_MODE_OFF            = 4, /**< Solid OFF: manually suppressed */
} pico_led_mode_t;

typedef struct p7501_led_state {
    pico_led_mode_t mode;
    uint8_t phase;
    bool manual_lock;
    bool current_on;
} p7501_led_state_t;

/** Initialize LED state machine to unlocked 1 Hz heartbeat. */
void p7501_led_state_init(p7501_led_state_t *state);

/** Compute deterministic LED GPIO output level for a given `(mode, phase 0..9)`. */
bool p7501_led_pattern_output(pico_led_mode_t mode, uint8_t phase);

/** Lock a user-selected LED mode so automatic telemetry (`STATUS`) does not override it. */
void p7501_led_state_set_manual(p7501_led_state_t *state, pico_led_mode_t mode);

/** Apply an automatic LED transition only when manual override lock is NOT engaged. */
bool p7501_led_state_set_auto(p7501_led_state_t *state, pico_led_mode_t mode);

/** Release manual override lock and immediately transition to `fallback_mode`. */
void p7501_led_state_unlock_auto(p7501_led_state_t *state, pico_led_mode_t fallback_mode);

/** Toggle LED output transiently on packet/bus activity when compatible with current lock. */
void p7501_led_state_pulse_activity(p7501_led_state_t *state);

/** Advance 10 Hz phase (`0..9`) and return updated physical LED state. */
bool p7501_led_state_step_10hz(p7501_led_state_t *state);

/** Return canonical uppercase string identifier for `mode`. */
const char *p7501_led_mode_name(pico_led_mode_t mode);

/**
 * @brief Parse `LED` CLI argument (`IDLE|ACTIVE|GDB|FAULT|OFF|AUTO|STATUS`).
 *
 * @return true when recognized, false on invalid mode keyword.
 */
bool p7501_led_parse_arg(
    const char *arg,
    pico_led_mode_t *out_mode,
    bool *out_is_auto,
    bool *out_is_query
);

#ifdef __cplusplus
}
#endif

#endif /* P7501_LED_LOGIC_H */
