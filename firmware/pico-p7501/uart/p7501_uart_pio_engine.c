/**
 * @file p7501_uart_pio_engine.c
 * @brief Multi-baud PIO1 RX/TX engine with strict P7501 electrical safety invariants.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "uart/p7501_uart_pio_engine.h"

#include "hal/p7501_hw.h"
#include "p7501_types.h"

#include "hardware/adc.h"
#include "hardware/clocks.h"
#include "hardware/gpio.h"
#include "hardware/pio.h"
#include "pico/stdlib.h"
#include "p7501_uart_duplex.pio.h"

#define P7501_PIO_UART_INST pio1
#define P7501_PIO_RX_SM 0u
#define P7501_PIO_TX_SM 1u

static uint32_t s_active_baud = P7501_UART_DEFAULT_BAUD;
static bool s_rx_loaded = false;
static bool s_sw_uart_timed_hold = false;
static absolute_time_t s_sw_uart_release_deadline;
static uint s_rx_offset = 0u;
static bool s_tx_loaded = false;
static uint s_tx_offset = 0u;

static float compute_pio_clkdiv(uint32_t baud) {
    uint16_t div_i = 0u;
    uint8_t div_f = 0u;
    uint32_t sys_hz = clock_get_hz(clk_sys);

    p7501_calc_pio_clkdiv_fp16(sys_hz, baud, &div_i, &div_f);
    return (float)div_i + ((float)div_f / 256.0f);
}

static void ensure_programs_loaded(void) {
    if (!s_rx_loaded) {
        s_rx_offset = pio_add_program(P7501_PIO_UART_INST, &p7501_uart_rx_fast_program);
        s_rx_loaded = true;
    }
    if (!s_tx_loaded) {
        s_tx_offset = pio_add_program(P7501_PIO_UART_INST, &p7501_uart_tx_fast_program);
        s_tx_loaded = true;
    }
}

static void release_tx_pin_hiz(void) {
    pio_sm_set_enabled(P7501_PIO_UART_INST, P7501_PIO_TX_SM, false);
    gpio_set_function(P7501_PIN_TXD, GPIO_FUNC_SIO);
    gpio_set_dir(P7501_PIN_TXD, GPIO_IN);
    gpio_pull_up(P7501_PIN_TXD);
}

void p7501_pio_enforce_nsw5v_hiz(void) {
    gpio_set_function(P7501_PIN_NSW5V, GPIO_FUNC_SIO);
    gpio_set_dir(P7501_PIN_NSW5V, GPIO_IN);
    gpio_disable_pulls(P7501_PIN_NSW5V);
}

void p7501_pio_set_sw_uart(bool drive_low) {
    if (drive_low) {
        gpio_init(P7501_PIN_SW_UART_ADC);
        gpio_set_dir(P7501_PIN_SW_UART_ADC, GPIO_OUT);
        gpio_put(P7501_PIN_SW_UART_ADC, 0);
    } else {
        p7501_hw_restore_sw_uart_adc0();
    }
}

void p7501_pio_pulse_xreset_ms(uint32_t hold_low_ms) {
    p7501_pio_enforce_nsw5v_hiz();
    gpio_init(P7501_PIN_XRESET);
    gpio_put(P7501_PIN_XRESET, 0);
    gpio_set_dir(P7501_PIN_XRESET, GPIO_OUT);
    sleep_ms(hold_low_ms > 0u ? hold_low_ms : 15u);
    gpio_set_dir(P7501_PIN_XRESET, GPIO_IN);
    gpio_disable_pulls(P7501_PIN_XRESET);
}

uint32_t p7501_pio_get_baud(void) {
    return s_active_baud;
}

bool p7501_pio_set_baud(uint32_t baud) {
    if (!p7501_baud_is_valid(baud)) {
        return false;
    }
    s_active_baud = baud;
    if (s_rx_loaded) {
        pio_sm_set_clkdiv(P7501_PIO_UART_INST, P7501_PIO_RX_SM, compute_pio_clkdiv(s_active_baud));
    }
    if (s_tx_loaded) {
        pio_sm_set_clkdiv(P7501_PIO_UART_INST, P7501_PIO_TX_SM, compute_pio_clkdiv(s_active_baud));
    }
    return true;
}

void p7501_pio_begin_rx_session(p7501_boot_action_t action) {
    p7501_pio_begin_rx_session_ex(action, 0u);
}

void p7501_pio_begin_rx_session_ex(p7501_boot_action_t action, uint32_t jig_hold_ms) {
    float div = compute_pio_clkdiv(s_active_baud);

    s_sw_uart_timed_hold = false;
    ensure_programs_loaded();
    p7501_pio_enforce_nsw5v_hiz();
    release_tx_pin_hiz();

    if (action == P7501_BOOT_ACTION_JIG) {
        p7501_pio_set_sw_uart(true);
    } else {
        p7501_pio_set_sw_uart(false);
    }

    p7501_uart_rx_fast_init(
        P7501_PIO_UART_INST,
        P7501_PIO_RX_SM,
        s_rx_offset,
        P7501_PIN_RXD,
        div
    );

    if (action != P7501_BOOT_ACTION_PASSIVE) {
        sleep_ms(5u);
        p7501_pio_pulse_xreset_ms(15u);
        if (action == P7501_BOOT_ACTION_JIG && jig_hold_ms > 0u) {
            s_sw_uart_timed_hold = true;
            s_sw_uart_release_deadline = make_timeout_time_ms(jig_hold_ms);
        }
    }
}

bool p7501_pio_poll_sw_uart_release(void) {
    if (!s_sw_uart_timed_hold) {
        return false;
    }
    if (!time_reached(s_sw_uart_release_deadline)) {
        return false;
    }
    s_sw_uart_timed_hold = false;
    p7501_pio_set_sw_uart(false);
    p7501_hw_restore_sw_uart_adc0();
    return true;
}

void p7501_pio_end_session(void) {
    s_sw_uart_timed_hold = false;
    pio_sm_set_enabled(P7501_PIO_UART_INST, P7501_PIO_RX_SM, false);
    release_tx_pin_hiz();
    p7501_pio_set_sw_uart(false);
    p7501_hw_restore_sw_uart_adc0();
    p7501_pio_enforce_nsw5v_hiz();
    p7501_hw_isolate_bus_hiz();
}

bool p7501_pio_pop_rx_byte(uint8_t *out_byte) {
    if (pio_sm_is_rx_fifo_empty(P7501_PIO_UART_INST, P7501_PIO_RX_SM)) {
        return false;
    }
    uint32_t raw = pio_sm_get(P7501_PIO_UART_INST, P7501_PIO_RX_SM);
    if (out_byte) {
        *out_byte = (uint8_t)(raw >> 24);
    }
    return true;
}

bool p7501_pio_poll_and_clear_framing_err(void) {
    if (pio_interrupt_get(P7501_PIO_UART_INST, 4u)) {
        pio_interrupt_clear(P7501_PIO_UART_INST, 4u);
        return true;
    }
    return false;
}

bool p7501_pio_had_rx_overflow(void) {
    uint32_t mask = (1u << (PIO_FDEBUG_RXSTALL_LSB + P7501_PIO_RX_SM));
    return (P7501_PIO_UART_INST->fdebug & mask) != 0u;
}

static void engage_tx_pio(void) {
    float div = compute_pio_clkdiv(s_active_baud);

    ensure_programs_loaded();
    p7501_uart_tx_fast_init(
        P7501_PIO_UART_INST,
        P7501_PIO_TX_SM,
        s_tx_offset,
        P7501_PIN_TXD,
        div
    );
    pio_sm_set_pins_with_mask(
        P7501_PIO_UART_INST,
        P7501_PIO_TX_SM,
        (1u << P7501_PIN_TXD),
        (1u << P7501_PIN_TXD)
    );
    pio_sm_set_consecutive_pindirs(
        P7501_PIO_UART_INST,
        P7501_PIO_TX_SM,
        P7501_PIN_TXD,
        1u,
        true
    );
    pio_gpio_init(P7501_PIO_UART_INST, P7501_PIN_TXD);
    pio_sm_set_enabled(P7501_PIO_UART_INST, P7501_PIO_TX_SM, true);
}

void p7501_pio_tx_bytes(const uint8_t *data, size_t len, bool force_sw_uart_low) {
    uint32_t drain_us = ((p7501_baud_to_bit_ns(s_active_baud) * 12u) / 1000u) + 60u;

    if (!data || len == 0u) {
        return;
    }
    p7501_pio_enforce_nsw5v_hiz();
    p7501_pio_set_sw_uart(force_sw_uart_low);
    sleep_us(200u);

    engage_tx_pio();
    for (size_t i = 0u; i < len; ++i) {
        pio_sm_put_blocking(P7501_PIO_UART_INST, P7501_PIO_TX_SM, (uint32_t)data[i]);
    }
    while (!pio_sm_is_tx_fifo_empty(P7501_PIO_UART_INST, P7501_PIO_TX_SM)) {
        tight_loop_contents();
    }
    sleep_us(drain_us);
    release_tx_pin_hiz();
}

void p7501_pio_tx_break_ms(uint32_t break_ms, bool force_sw_uart_low) {
    p7501_pio_enforce_nsw5v_hiz();
    p7501_pio_set_sw_uart(force_sw_uart_low);
    release_tx_pin_hiz();

    gpio_set_function(P7501_PIN_TXD, GPIO_FUNC_SIO);
    gpio_put(P7501_PIN_TXD, 0);
    gpio_set_dir(P7501_PIN_TXD, GPIO_OUT);
    sleep_ms(break_ms > 0u ? break_ms : 120u);
    gpio_set_dir(P7501_PIN_TXD, GPIO_IN);
    gpio_pull_up(P7501_PIN_TXD);
}
