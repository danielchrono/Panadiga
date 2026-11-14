/**
 * @file p7501_led.c
 * @brief Hardware LED driver with IRQ-safe 100ms repeating timer and CYW43 support.
 *
 * Fixes three real-world failure modes on the live P7501 bench:
 * 1. Immediate pin application when switching modes (`IDLE`, `GDB`, `FAULT`, `OFF`).
 * 2. Manual lock so 2-second Desktop `STATUS` telemetry never clobbers user `LED <mode>`.
 * 3. Non-blocking 10 Hz `repeating_timer` tick on RP2040/RP2350 GP25 (and chunked sleep
 *    fallback on CYW43 Pico W / Pico 2 W) so blocking UART captures never freeze the LED.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "hal/p7501_led.h"

#include "hardware/gpio.h"
#include "hardware/sync.h"
#include "pico/stdlib.h"
#include "pico/time.h"

#if defined(PANADIGA_BLE_ENABLED) && (PANADIGA_BLE_ENABLED != 0)
#include "pico/cyw43_arch.h"
#endif

static p7501_led_state_t s_led_state;
static absolute_time_t s_next_poll_tick;

#if !defined(PANADIGA_BLE_ENABLED) || (PANADIGA_BLE_ENABLED == 0)
static struct repeating_timer s_led_timer;
static bool s_timer_active = false;
#endif

static void drive_physical_led(bool on) {
#if defined(PANADIGA_BLE_ENABLED) && (PANADIGA_BLE_ENABLED != 0)
    cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, on ? 1 : 0);
#else
    gpio_put(PICO_DEFAULT_LED_PIN, on ? 1 : 0);
#endif
}

#if !defined(PANADIGA_BLE_ENABLED) || (PANADIGA_BLE_ENABLED == 0)
static bool led_timer_callback(struct repeating_timer *rt) {
    (void)rt;
    const uint32_t irq_state = save_and_disable_interrupts();
    const bool on = p7501_led_state_step_10hz(&s_led_state);
    drive_physical_led(on);
    restore_interrupts(irq_state);
    return true;
}
#endif

void p7501_led_init(void) {
    const uint32_t irq_state = save_and_disable_interrupts();
    p7501_led_state_init(&s_led_state);
#if !defined(PANADIGA_BLE_ENABLED) || (PANADIGA_BLE_ENABLED == 0)
    gpio_init(PICO_DEFAULT_LED_PIN);
    gpio_set_dir(PICO_DEFAULT_LED_PIN, GPIO_OUT);
#endif
    drive_physical_led(s_led_state.current_on);
    restore_interrupts(irq_state);

    s_next_poll_tick = make_timeout_time_ms(100u);

#if !defined(PANADIGA_BLE_ENABLED) || (PANADIGA_BLE_ENABLED == 0)
    if (!s_timer_active) {
        s_timer_active = add_repeating_timer_ms(-100, led_timer_callback, NULL, &s_led_timer);
    }
#endif
}

void p7501_led_set_mode(pico_led_mode_t mode) {
    const uint32_t irq_state = save_and_disable_interrupts();
    p7501_led_state_set_manual(&s_led_state, mode);
    drive_physical_led(s_led_state.current_on);
    restore_interrupts(irq_state);
    s_next_poll_tick = make_timeout_time_ms(100u);
}

void p7501_led_set_auto_mode(pico_led_mode_t mode) {
    const uint32_t irq_state = save_and_disable_interrupts();
    if (p7501_led_state_set_auto(&s_led_state, mode)) {
        drive_physical_led(s_led_state.current_on);
        s_next_poll_tick = make_timeout_time_ms(100u);
    }
    restore_interrupts(irq_state);
}

void p7501_led_unlock_auto(pico_led_mode_t fallback_mode) {
    const uint32_t irq_state = save_and_disable_interrupts();
    p7501_led_state_unlock_auto(&s_led_state, fallback_mode);
    drive_physical_led(s_led_state.current_on);
    restore_interrupts(irq_state);
    s_next_poll_tick = make_timeout_time_ms(100u);
}

void p7501_led_notify_activity(void) {
    const uint32_t irq_state = save_and_disable_interrupts();
    p7501_led_state_pulse_activity(&s_led_state);
    drive_physical_led(s_led_state.current_on);
    restore_interrupts(irq_state);
}

void p7501_led_tick(void) {
#if defined(PANADIGA_BLE_ENABLED) && (PANADIGA_BLE_ENABLED != 0)
    if (!time_reached(s_next_poll_tick)) {
        return;
    }
    s_next_poll_tick = make_timeout_time_ms(100u);
    const bool on = p7501_led_state_step_10hz(&s_led_state);
    drive_physical_led(on);
#else
    if (!s_timer_active && time_reached(s_next_poll_tick)) {
        s_next_poll_tick = make_timeout_time_ms(100u);
        const uint32_t irq_state = save_and_disable_interrupts();
        const bool on = p7501_led_state_step_10hz(&s_led_state);
        drive_physical_led(on);
        restore_interrupts(irq_state);
    }
#endif
}

void p7501_led_sleep_ms(uint32_t delay_ms) {
    uint32_t remaining = delay_ms;
    while (remaining > 0u) {
        const uint32_t slice = (remaining > 50u) ? 50u : remaining;
        sleep_ms(slice);
        p7501_led_tick();
        remaining -= slice;
    }
}

p7501_led_state_t p7501_led_get_state(void) {
    const uint32_t irq_state = save_and_disable_interrupts();
    const p7501_led_state_t snap = s_led_state;
    restore_interrupts(irq_state);
    return snap;
}

const char *p7501_led_backend_label(void) {
#if defined(PANADIGA_BLE_ENABLED) && (PANADIGA_BLE_ENABLED != 0)
    return "CYW43_WL_GPIO0 (Pico W / Pico 2 W)";
#else
    return "GP25 (Standard Pico / RP2040 / RP2350)";
#endif
}
