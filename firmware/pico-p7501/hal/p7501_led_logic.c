/**
 * @file p7501_led_logic.c
 * @brief Pure C11 LED pattern generator, manual lock semantics, and CLI arg parser.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "hal/p7501_led_logic.h"
#include "common/p7501_hex.h"

#include <stddef.h>

static void apply_mode_preserving_lock(p7501_led_state_t *state, pico_led_mode_t mode) {
    if (state->mode != mode) {
        state->mode = mode;
        state->phase = 0u;
    }
    state->current_on = p7501_led_pattern_output(state->mode, state->phase);
}

void p7501_led_state_init(p7501_led_state_t *state) {
    if (state == NULL) {
        return;
    }
    state->mode = LED_MODE_HEARTBEAT_IDLE;
    state->phase = 0u;
    state->manual_lock = false;
    state->current_on = p7501_led_pattern_output(state->mode, 0u);
}

bool p7501_led_pattern_output(pico_led_mode_t mode, uint8_t phase) {
    const uint8_t slot = (uint8_t)(phase % 10u);
    switch (mode) {
        case LED_MODE_BUS_ACTIVE:
            return true;
        case LED_MODE_GDB_RSP:
            return (slot % 2u) == 0u;
        case LED_MODE_BUS_FAULT:
            return slot == 0u || slot == 2u || slot == 4u;
        case LED_MODE_OFF:
            return false;
        case LED_MODE_HEARTBEAT_IDLE:
        default:
            return slot == 0u || slot == 1u;
    }
}

void p7501_led_state_set_manual(p7501_led_state_t *state, pico_led_mode_t mode) {
    if (state == NULL) {
        return;
    }
    state->manual_lock = true;
    apply_mode_preserving_lock(state, mode);
}

bool p7501_led_state_set_auto(p7501_led_state_t *state, pico_led_mode_t mode) {
    if (state == NULL || state->manual_lock) {
        return false;
    }
    apply_mode_preserving_lock(state, mode);
    return true;
}

void p7501_led_state_unlock_auto(p7501_led_state_t *state, pico_led_mode_t fallback_mode) {
    if (state == NULL) {
        return;
    }
    state->manual_lock = false;
    apply_mode_preserving_lock(state, fallback_mode);
}

void p7501_led_state_pulse_activity(p7501_led_state_t *state) {
    if (state == NULL) {
        return;
    }
    if (state->manual_lock && state->mode != LED_MODE_GDB_RSP) {
        return;
    }
    state->current_on = !state->current_on;
}

bool p7501_led_state_step_10hz(p7501_led_state_t *state) {
    if (state == NULL) {
        return false;
    }
    state->phase = (uint8_t)((state->phase + 1u) % 10u);
    state->current_on = p7501_led_pattern_output(state->mode, state->phase);
    return state->current_on;
}

const char *p7501_led_mode_name(pico_led_mode_t mode) {
    switch (mode) {
        case LED_MODE_BUS_ACTIVE:
            return "BUS_ACTIVE";
        case LED_MODE_GDB_RSP:
            return "GDB_RSP";
        case LED_MODE_BUS_FAULT:
            return "BUS_FAULT";
        case LED_MODE_OFF:
            return "OFF";
        case LED_MODE_HEARTBEAT_IDLE:
        default:
            return "HEARTBEAT_IDLE";
    }
}

bool p7501_led_parse_arg(
    const char *arg,
    pico_led_mode_t *out_mode,
    bool *out_is_auto,
    bool *out_is_query
) {
    if (out_mode == NULL || out_is_auto == NULL || out_is_query == NULL) {
        return false;
    }
    *out_mode = LED_MODE_HEARTBEAT_IDLE;
    *out_is_auto = false;
    *out_is_query = false;

    if (arg == NULL || arg[0] == '\0' || p7501_ascii_ieq(arg, "STATUS") || p7501_ascii_ieq(arg, "?")) {
        *out_is_query = true;
        return true;
    }
    if (p7501_ascii_ieq(arg, "AUTO")) {
        *out_is_auto = true;
        return true;
    }
    if (p7501_ascii_ieq(arg, "ACTIVE")) {
        *out_mode = LED_MODE_BUS_ACTIVE;
        return true;
    }
    if (p7501_ascii_ieq(arg, "GDB")) {
        *out_mode = LED_MODE_GDB_RSP;
        return true;
    }
    if (p7501_ascii_ieq(arg, "FAULT")) {
        *out_mode = LED_MODE_BUS_FAULT;
        return true;
    }
    if (p7501_ascii_ieq(arg, "OFF")) {
        *out_mode = LED_MODE_OFF;
        return true;
    }
    if (p7501_ascii_ieq(arg, "IDLE")) {
        *out_mode = LED_MODE_HEARTBEAT_IDLE;
        return true;
    }
    return false;
}
