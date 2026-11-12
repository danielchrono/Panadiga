/**
 * @file dwire32a_led.c
 * @brief Implementation of Onboard LED visual driver for Raspberry Pi Pico.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "hal/dwire32a_led.h"

#if !defined(PICO_HOST_TEST)
#include "pico/stdlib.h"
#endif

static pico_led_mode_t s_led_mode = LED_MODE_HEARTBEAT_IDLE;
static uint32_t s_led_last_ms = 0U;
static bool s_led_state = false;

void dwire_led_init(void) {
#if !defined(PICO_HOST_TEST)
    gpio_init(PICO_DEFAULT_LED_PIN);
    gpio_set_dir(PICO_DEFAULT_LED_PIN, GPIO_OUT);
    gpio_put(PICO_DEFAULT_LED_PIN, 0);
#endif
    s_led_state = false;
    s_led_mode = LED_MODE_HEARTBEAT_IDLE;
}

void dwire_led_set_mode(pico_led_mode_t mode) {
    s_led_mode = mode;
#if !defined(PICO_HOST_TEST)
    if (mode == LED_MODE_DWIRE_ACTIVE) {
        gpio_put(PICO_DEFAULT_LED_PIN, 1);
        s_led_state = true;
    }
#endif
}

void dwire_led_notify_activity(void) {
#if !defined(PICO_HOST_TEST)
    s_led_state = !s_led_state;
    gpio_put(PICO_DEFAULT_LED_PIN, s_led_state ? 1 : 0);
#endif
}

void dwire_led_tick(void) {
#if !defined(PICO_HOST_TEST)
    const uint32_t now = to_ms_since_boot(get_absolute_time());

    switch (s_led_mode) {
        case LED_MODE_HEARTBEAT_IDLE: {
            /* 1000ms cycle: ON for 100ms, OFF for 900ms */
            const uint32_t phase = now % 1000U;
            const bool target_state = (phase < 100U);
            if (target_state != s_led_state) {
                s_led_state = target_state;
                gpio_put(PICO_DEFAULT_LED_PIN, s_led_state ? 1 : 0);
            }
            break;
        }
        case LED_MODE_DWIRE_ACTIVE: {
            if (!s_led_state) {
                s_led_state = true;
                gpio_put(PICO_DEFAULT_LED_PIN, 1);
            }
            break;
        }
        case LED_MODE_GDB_RSP: {
            /* 100ms toggle rate (5 Hz) in GDB mode */
            if (now - s_led_last_ms >= 100U) {
                s_led_last_ms = now;
                s_led_state = !s_led_state;
                gpio_put(PICO_DEFAULT_LED_PIN, s_led_state ? 1 : 0);
            }
            break;
        }
        case LED_MODE_BUS_FAULT: {
            /* Rapid triple blink: 100ms on/off x 3, then 600ms pause (1200ms cycle) */
            const uint32_t phase = now % 1200U;
            const bool target_state = (phase < 600U) && (((phase / 100U) % 2U) == 0U);
            if (target_state != s_led_state) {
                s_led_state = target_state;
                gpio_put(PICO_DEFAULT_LED_PIN, s_led_state ? 1 : 0);
            }
            break;
        }
    }
#endif
}
