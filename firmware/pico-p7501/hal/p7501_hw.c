/**
 * @file p7501_hw.c
 * @brief Safe Hardware Abstraction Layer implementation for Panasonic P7501 bus.
 *
 * Implements Hi-Z safe defaults, GP26 ADC0 restoration (clearing SIO OE before
 * switching pad mux), Open-Drain XRESET control, and passive PIO0 115200 8N1
 * UART RX initialization on GP2 without ever touching GP3 (NSW5V).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "hal/p7501_hw.h"
#include "pico/stdlib.h"
#include "hardware/pio.h"
#include "hardware/adc.h"
#include "p7501_uart.pio.h"

static PIO  s_pio_inst    = (PIO)0;
static uint s_pio_sm      = 0U;
static uint s_pio_offset  = 0U;
static bool s_initialized = false;

static void enforce_nsw5v_hiz_safe(void) {
    /* CRITICAL SAFETY INVARIANT: Pin 5 NSW5V (GP3) must ALWAYS stay Hi-Z Input with no pulls */
    gpio_set_function(P7501_PIN_NSW5V, GPIO_FUNC_SIO);
    gpio_set_dir(P7501_PIN_NSW5V, GPIO_IN);
    gpio_disable_pulls(P7501_PIN_NSW5V);
}

void p7501_hw_restore_sw_uart_adc0(void) {
    /* Clear SIO output enable BEFORE adc_gpio_init() so GP26 can never stay latched LOW */
    gpio_set_function(P7501_PIN_SW_UART_ADC, GPIO_FUNC_SIO);
    gpio_set_dir(P7501_PIN_SW_UART_ADC, GPIO_IN);
    gpio_disable_pulls(P7501_PIN_SW_UART_ADC);
    adc_gpio_init(P7501_PIN_SW_UART_ADC);
}

p7501_status_t p7501_hw_init(void) {
    s_pio_inst = pio0;
    s_pio_sm   = 0U;

    adc_init();
    p7501_hw_restore_sw_uart_adc0();

    /* Pin 6 UARTP2M_T (GP2): Input from Target TX */
    gpio_init(P7501_PIN_RXD);
    gpio_set_dir(P7501_PIN_RXD, GPIO_IN);
    gpio_pull_up(P7501_PIN_RXD);

    /* Pin 5 NSW5V (GP3): Strictly Hi-Z Input */
    gpio_init(P7501_PIN_NSW5V);
    enforce_nsw5v_hiz_safe();

    /* Pin 3 UARTM2P_T (GP4): Idle Hi-Z Input until UART TX requested */
    gpio_init(P7501_PIN_TXD);
    gpio_set_dir(P7501_PIN_TXD, GPIO_IN);
    gpio_disable_pulls(P7501_PIN_TXD);

    /* Pin 2 XRESET (GP5): Open-Drain (latch 0, released to Hi-Z Input) */
    gpio_init(P7501_PIN_XRESET);
    gpio_disable_pulls(P7501_PIN_XRESET);
    gpio_put(P7501_PIN_XRESET, false);
    p7501_hw_reset_release();

    if (!s_initialized) {
        if (!pio_can_add_program(s_pio_inst, &p7501_uart_rx_program)) {
            return P7501_ERR_PIO_INIT_FAILED;
        }
        s_pio_offset = pio_add_program(s_pio_inst, &p7501_uart_rx_program);
        p7501_uart_rx_program_init(
            s_pio_inst,
            s_pio_sm,
            s_pio_offset,
            P7501_PIN_RXD,
            P7501_UART_BAUD_RATE
        );
        s_initialized = true;
    }
    return P7501_OK;
}

void p7501_hw_reset_assert(void) {
    gpio_set_function(P7501_PIN_XRESET, GPIO_FUNC_SIO);
    gpio_put(P7501_PIN_XRESET, false);
    gpio_set_dir(P7501_PIN_XRESET, GPIO_OUT);
}

void p7501_hw_reset_release(void) {
    gpio_set_function(P7501_PIN_XRESET, GPIO_FUNC_SIO);
    gpio_set_dir(P7501_PIN_XRESET, GPIO_IN);
    gpio_disable_pulls(P7501_PIN_XRESET);
}

void p7501_hw_sw_uart_set_low(bool assert_low) {
    if (assert_low) {
        gpio_set_function(P7501_PIN_SW_UART_ADC, GPIO_FUNC_SIO);
        gpio_disable_pulls(P7501_PIN_SW_UART_ADC);
        gpio_put(P7501_PIN_SW_UART_ADC, false);
        gpio_set_dir(P7501_PIN_SW_UART_ADC, GPIO_OUT);
    } else {
        p7501_hw_restore_sw_uart_adc0();
    }
}

float p7501_hw_read_vdd(void) {
    p7501_hw_restore_sw_uart_adc0();
    adc_select_input(P7501_ADC_CH_VDD);

    /* Brief pull-down discriminates an open floating pin (~0V) from a connected P7501 (~2.68V via Q7510) */
    gpio_pull_down(P7501_PIN_SW_UART_ADC);
    sleep_us(P7501_ADC_PULLDOWN_SETTLE_US);

    uint32_t accumulator = 0U;
    for (uint8_t i = 0U; i < P7501_ADC_SAMPLE_COUNT; ++i) {
        accumulator += (uint32_t)adc_read();
    }

    gpio_disable_pulls(P7501_PIN_SW_UART_ADC);
    const float averaged = (float)accumulator / (float)P7501_ADC_SAMPLE_COUNT;
    return (averaged * P7501_ADC_VREF) / P7501_ADC_RESOLUTION;
}

bool p7501_hw_is_target_powered(void) {
    const float vdd = p7501_hw_read_vdd();
    return (vdd >= P7501_VDD_THRESHOLD_V) && (vdd < P7501_VDD_FLOATING_CEIL_V);
}

bool p7501_hw_is_initialized(void) {
    return s_initialized;
}

void p7501_hw_boot_sequence(p7501_boot_mode_t mode) {
    const bool use_serial_boot =
        (mode == P7501_BOOT_MODE_SERIAL_LOW_RELEASE) ||
        (mode == P7501_BOOT_MODE_SERIAL_LOW_HOLD);

    p7501_hw_isolate_bus_hiz();
    p7501_hw_sw_uart_set_low(use_serial_boot);

    p7501_hw_reset_assert();
    sleep_ms(P7501_RESET_ASSERT_MS);
    p7501_hw_reset_release();

    for (uint32_t wait_us = 0U; wait_us < 2000U; wait_us += 10U) {
        if (gpio_get(P7501_PIN_XRESET)) {
            break;
        }
        sleep_us(10U);
    }

    if (mode == P7501_BOOT_MODE_SERIAL_LOW_RELEASE) {
        sleep_us(P7501_RESET_SETTLE_US);
        p7501_hw_sw_uart_set_low(false);
    } else if (mode == P7501_BOOT_MODE_HIZ_EXTENDED) {
        sleep_ms(5U);
    } else {
        sleep_us(P7501_POST_RESET_DELAY_US);
    }

    if (mode != P7501_BOOT_MODE_SERIAL_LOW_HOLD) {
        p7501_hw_restore_sw_uart_adc0();
    }
    enforce_nsw5v_hiz_safe();
}

void p7501_hw_isolate_bus_hiz(void) {
    p7501_hw_reset_release();
    enforce_nsw5v_hiz_safe();

    gpio_set_function(P7501_PIN_TXD, GPIO_FUNC_SIO);
    gpio_set_dir(P7501_PIN_TXD, GPIO_IN);
    gpio_disable_pulls(P7501_PIN_TXD);

    gpio_set_function(P7501_PIN_RXD, GPIO_FUNC_SIO);
    gpio_set_dir(P7501_PIN_RXD, GPIO_IN);
    gpio_disable_pulls(P7501_PIN_RXD);

    p7501_hw_restore_sw_uart_adc0();
}

void p7501_hw_engage_uart_bus(void) {
    enforce_nsw5v_hiz_safe();

    gpio_set_function(P7501_PIN_RXD, GPIO_FUNC_SIO);
    gpio_set_dir(P7501_PIN_RXD, GPIO_IN);
    gpio_pull_up(P7501_PIN_RXD);

    gpio_set_function(P7501_PIN_TXD, GPIO_FUNC_SIO);
    gpio_put(P7501_PIN_TXD, true);
    gpio_set_dir(P7501_PIN_TXD, GPIO_OUT);

    p7501_hw_restore_sw_uart_adc0();
}
