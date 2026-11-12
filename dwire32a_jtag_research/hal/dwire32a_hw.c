/**
 * @file dwire32a_hw.c
 * @brief Hardware Abstraction Layer implementation for Panasonic DWire32A.
 *
 * Implements low-level electrical controls, ADC VDD voltage sensing,
 * Open-Drain reset drive, and PIO half-duplex shift operations.
 *
 * Target: Panasonic UniPhier / Matsushita AM34 / MN103S SoC Family (MN2DS0004 / MN2DS0009 / MN2DS0016 / MN2DS0018)
 * Models: Panasonic DIGA Series (DMR-E / DMR-ExxH / DMR-ES / DMR-EH / DMR-EX / DMR-EZ)
 * Host:   Raspberry Pi Pico (RP2040 / RP2350)
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "hal/dwire32a_hw.h"
#include "pico/stdlib.h"
#include "hardware/pio.h"
#include "hardware/adc.h"
#include "dwire32a.pio.h"

/* Static hardware instance state */
static PIO  s_pio_inst    = (PIO)0;
static uint s_pio_sm      = 0U;
static uint s_pio_offset  = 0U;
static bool s_initialized = false;

dwire_status_t dwire_hw_init(void) {
    s_pio_inst = pio0;
    s_pio_sm   = 0U;

    /* Initialize ADC subsystem for VDD monitoring */
    adc_init();
    adc_gpio_init(DWIRE_PIN_VDD_ADC);

    /* Initialize EXTRG as standard push-pull digital output (idle Low) */
    gpio_init(DWIRE_PIN_EXTRG);
    gpio_set_dir(DWIRE_PIN_EXTRG, GPIO_OUT);
    gpio_put(DWIRE_PIN_EXTRG, false);

    /* Initialize nRESET as Open-Drain (driven Low, released as High-Z Input) */
    gpio_init(DWIRE_PIN_NRESET);
    gpio_disable_pulls(DWIRE_PIN_NRESET); /* Disable internal pull-down so line floats to target VDD via board pull-up */
    gpio_put(DWIRE_PIN_NRESET, 0);
    dwire_hw_reset_release();

    /* Load and initialize the DWire32A PIO state machine (only if not already loaded) */
    if (!s_initialized) {
        if (!pio_can_add_program(s_pio_inst, &dwire32a_program)) {
            return DWIRE_ERR_PIO_INIT_FAILED;
        }

        s_pio_offset = pio_add_program(s_pio_inst, &dwire32a_program);
        dwire32a_program_init(s_pio_inst, s_pio_sm, s_pio_offset, DWIRE_PIN_SDATA, DWIRE_PIN_SCLK);

        s_initialized = true;
    }
    return DWIRE_OK;
}

void dwire_hw_reset_assert(void) {
    /* Drive nRESET actively to 0V (Output Low) */
    gpio_put(DWIRE_PIN_NRESET, 0);
    gpio_set_dir(DWIRE_PIN_NRESET, GPIO_OUT);
}

void dwire_hw_reset_release(void) {
    /* Release nRESET to High-Z (Input) so target pull-up resistor raises it to 3.3V */
    gpio_set_dir(DWIRE_PIN_NRESET, GPIO_IN);
}

void dwire_hw_extrg_set(bool level) {
    gpio_put(DWIRE_PIN_EXTRG, level);
}

float dwire_hw_read_vdd(void) {
    adc_select_input(DWIRE_ADC_CH_VDD);
    /* Active floating-pin discrimination: engage the internal pull-down (~50-80 kOhm)
     * for the duration of the measurement.  An UNCONNECTED GP26 discharges to ~0 V,
     * whereas a real 3.3 V rail behind the 1 kOhm series resistor stays above ~3.2 V
     * (resistor divider 1k : 60k).  This lets a genuinely powered target read ~3.25 V
     * without being mistaken for a floating input. */
    gpio_pull_down(DWIRE_PIN_VDD_ADC);
    sleep_us(DWIRE_ADC_PULLDOWN_SETTLE_US);
    /* Average DWIRE_ADC_SAMPLE_COUNT successive reads to suppress quantisation
     * noise and filter the RP2040 ESD-diode charge-share artefact that biases
     * a floating GP26 input toward AVDD.  At ~500 ns per SAR conversion the
     * total sampling window is < 4 µs — well within any power-state window. */
    uint32_t accumulator = 0U;
    for (uint8_t i = 0U; i < DWIRE_ADC_SAMPLE_COUNT; ++i) {
        accumulator += (uint32_t)adc_read();
    }
    gpio_disable_pulls(DWIRE_PIN_VDD_ADC);
    const float averaged = (float)accumulator / (float)DWIRE_ADC_SAMPLE_COUNT;
    return (averaged * DWIRE_ADC_VREF) / DWIRE_ADC_RESOLUTION;
}

bool dwire_hw_is_target_powered(void) {
    const float vdd = dwire_hw_read_vdd();
    /* A floating ADC input (probe disconnected) biases toward AVDD (3.3 V).
     * Readings at or above DWIRE_VDD_FLOATING_CEIL_V indicate a disconnected
     * probe, NOT a powered target — return false to prevent false alarms. */
    if (vdd >= DWIRE_VDD_FLOATING_CEIL_V) {
        return false; /* Floating input: no probe attached */
    }
    return (vdd >= DWIRE_VDD_THRESHOLD_V);
}

bool dwire_hw_is_initialized(void) {
    return s_initialized;
}

void dwire_hw_write_bits(uint32_t data, uint8_t n_bits) {
    if (!s_initialized || n_bits == 0U || n_bits > 32U) {
        return;
    }
    const uint32_t header = dwire_make_header(DWIRE_DIR_WRITE, n_bits);
    pio_sm_put_blocking(s_pio_inst, s_pio_sm, header);
    pio_sm_put_blocking(s_pio_inst, s_pio_sm, data);
}

uint32_t dwire_hw_read_bits(uint8_t n_bits) {
    if (!s_initialized || n_bits == 0U || n_bits > 32U) {
        return 0U;
    }
    const uint32_t header = dwire_make_header(DWIRE_DIR_READ, n_bits);
    pio_sm_put_blocking(s_pio_inst, s_pio_sm, header);
    uint32_t val = pio_sm_get_blocking(s_pio_inst, s_pio_sm);
    /* For right-shift into ISR MSB, right-align if less than 32 bits read */
    if (n_bits < 32U) {
        val >>= (32U - n_bits);
    }
    return val;
}

void dwire_hw_cold_break_in(dwire_breakin_mode_t mode) {
    /* Ensure bus pins are engaged (in case ISOLATE/HIZ was previously active) */
    dwire_hw_engage_bus_active();

    const bool extrg_idle   = ((uint32_t)mode & DWIRE_BREAKIN_FLAG_EXTRG_INVERT) != 0U;
    const bool extrg_active = !extrg_idle;

    /* Step 1: Hold nRESET asserted at 0V and configure EXTRG to idle level */
    dwire_hw_reset_assert();
    dwire_hw_extrg_set(extrg_idle);
    sleep_ms(DWIRE_RESET_ASSERT_MS);

    /* Step 2: Generate 32 clock cycles on SCLK with SDATA in Hi-Z to clock OCD domain */
    (void)dwire_hw_read_bits(DWIRE_BREAKIN_CLOCK_COUNT);

    /* Step 3: Transition EXTRG to active state and release nRESET to 3.3V (Hi-Z) */
    dwire_hw_extrg_set(extrg_active);
    sleep_us(DWIRE_RESET_SETTLE_US);
    dwire_hw_reset_release();

    /* Wait up to 2000 us for nRESET line to physically rise above VIH (if external pull-up is present) */
    for (uint32_t wait_us = 0U; wait_us < 2000U; wait_us += 10U) {
        if (gpio_get(DWIRE_PIN_NRESET)) {
            break;
        }
        sleep_us(10U);
    }

    /* Step 4: Mode-dependent post-reset handling */
    if (((uint32_t)mode & DWIRE_BREAKIN_FLAG_POST_BURST) != 0U) {
        /* Generate immediate secondary 32-clock burst */
        (void)dwire_hw_read_bits(DWIRE_BREAKIN_CLOCK_COUNT);
    } else {
        /* Allow core settling window */
        sleep_us(DWIRE_POST_RESET_DELAY_US);
    }
}

void dwire_hw_detach_pins_for_sniff(void) {
    /* Route pins away from PIO to SIO as high-impedance inputs with pulls disabled */
    gpio_set_function(DWIRE_PIN_SCLK, GPIO_FUNC_SIO);
    gpio_set_dir(DWIRE_PIN_SCLK, GPIO_IN);
    gpio_disable_pulls(DWIRE_PIN_SCLK);

    gpio_set_function(DWIRE_PIN_SDATA, GPIO_FUNC_SIO);
    gpio_set_dir(DWIRE_PIN_SDATA, GPIO_IN);
    gpio_disable_pulls(DWIRE_PIN_SDATA);
}

void dwire_hw_attach_pins_after_sniff(void) {
    /* Re-attach SCLK and SDATA to the PIO state machine */
    pio_gpio_init(s_pio_inst, DWIRE_PIN_SCLK);
    pio_gpio_init(s_pio_inst, DWIRE_PIN_SDATA);
    gpio_pull_up(DWIRE_PIN_SDATA);

    pio_sm_set_pins_with_mask(s_pio_inst, s_pio_sm, 0, (1u << DWIRE_PIN_SCLK) | (1u << DWIRE_PIN_SDATA));
    pio_sm_set_pindirs_with_mask(s_pio_inst, s_pio_sm, (1u << DWIRE_PIN_SCLK), (1u << DWIRE_PIN_SCLK) | (1u << DWIRE_PIN_SDATA));
}

void dwire_hw_isolate_bus_hiz(void) {
    /* 1. Release nRESET to High-Z Input with internal pulls disabled */
    dwire_hw_reset_release();
    gpio_disable_pulls(DWIRE_PIN_NRESET);

    /* 2. Switch EXTRG to High-Z Input with internal pulls disabled so internal DMD0/DMD1 pull-downs win */
    gpio_set_function(DWIRE_PIN_EXTRG, GPIO_FUNC_SIO);
    gpio_set_dir(DWIRE_PIN_EXTRG, GPIO_IN);
    gpio_disable_pulls(DWIRE_PIN_EXTRG);

    /* 3. Detach SCLK and SDATA from PIO into High-Z Input with pulls disabled */
    dwire_hw_detach_pins_for_sniff();
}

void dwire_hw_engage_bus_active(void) {
    /* Restore EXTRG as digital output (idle Low) and re-attach SCLK/SDATA to PIO0 */
    gpio_set_function(DWIRE_PIN_EXTRG, GPIO_FUNC_SIO);
    gpio_put(DWIRE_PIN_EXTRG, false);
    gpio_set_dir(DWIRE_PIN_EXTRG, GPIO_OUT);

    dwire_hw_attach_pins_after_sniff();
}
