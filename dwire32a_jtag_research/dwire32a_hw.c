/**
 * @file dwire32a_hw.c
 * @brief Hardware Abstraction Layer implementation for Panasonic DWire32A.
 *
 * Implements low-level electrical controls, ADC VDD voltage sensing,
 * Open-Drain reset drive, and PIO half-duplex shift operations.
 *
 * Target: Panasonic DMR-EH55 (MN2DS0016 / AM34)
 * Host:   Raspberry Pi Pico (RP2040)
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "dwire32a_hw.h"
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
    gpio_put(DWIRE_PIN_NRESET, 0);
    dwire_hw_reset_release();

    /* Load and initialize the DWire32A PIO state machine */
    if (!pio_can_add_program(s_pio_inst, &dwire32a_program)) {
        return DWIRE_ERR_PIO_INIT_FAILED;
    }

    s_pio_offset = pio_add_program(s_pio_inst, &dwire32a_program);
    dwire32a_program_init(s_pio_inst, s_pio_sm, s_pio_offset, DWIRE_PIN_SDATA, DWIRE_PIN_SCLK);

    s_initialized = true;
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
    const uint16_t raw_adc = adc_read();
    return ((float)raw_adc * DWIRE_ADC_VREF) / DWIRE_ADC_RESOLUTION;
}

bool dwire_hw_is_target_powered(void) {
    return (dwire_hw_read_vdd() >= DWIRE_VDD_THRESHOLD_V);
}

bool dwire_hw_is_initialized(void) {
    return s_initialized;
}

void dwire_hw_write_bits(uint32_t data, uint8_t n_bits) {
    if (n_bits == 0U || n_bits > 32U) {
        return;
    }
    const uint32_t header = dwire_make_header(DWIRE_DIR_WRITE, n_bits);
    pio_sm_put_blocking(s_pio_inst, s_pio_sm, header);
    pio_sm_put_blocking(s_pio_inst, s_pio_sm, data);
}

uint32_t dwire_hw_read_bits(uint8_t n_bits) {
    if (n_bits == 0U || n_bits > 32U) {
        return 0U;
    }
    const uint32_t header = dwire_make_header(DWIRE_DIR_READ, n_bits);
    pio_sm_put_blocking(s_pio_inst, s_pio_sm, header);
    return pio_sm_get_blocking(s_pio_inst, s_pio_sm);
}

void dwire_hw_cold_break_in(dwire_breakin_mode_t mode) {
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

    /* Step 4: Mode-dependent post-reset handling */
    if (((uint32_t)mode & DWIRE_BREAKIN_FLAG_POST_BURST) != 0U) {
        /* Generate immediate secondary 32-clock burst */
        (void)dwire_hw_read_bits(DWIRE_BREAKIN_CLOCK_COUNT);
    } else {
        /* Allow core settling window */
        sleep_us(DWIRE_POST_RESET_DELAY_US);
    }
}

void dwire_hw_write_word32(uint32_t word) {
    dwire_hw_write_bits(word, 32U);
}

uint32_t dwire_hw_read_word32(void) {
    return dwire_hw_read_bits(32U);
}

void dwire_hw_write_mem32(uint32_t addr, uint32_t data) {
    dwire_hw_write_bits((uint32_t)DWIRE_OCD_CMD_WRITE_MEM32, 8U);
    dwire_hw_write_bits(addr, 32U);
    dwire_hw_write_bits(data, 32U);
}

uint32_t dwire_hw_read_mem32(uint32_t addr) {
    dwire_hw_write_bits((uint32_t)DWIRE_OCD_CMD_READ_MEM32, 8U);
    dwire_hw_write_bits(addr, 32U);
    return dwire_hw_read_bits(32U);
}

void dwire_hw_set_pc(uint32_t pc_val) {
    dwire_hw_write_bits((uint32_t)DWIRE_OCD_CMD_WRITE_PC, 8U);
    dwire_hw_write_bits(pc_val, 32U);
}

void dwire_hw_resume_cpu(void) {
    dwire_hw_write_bits((uint32_t)DWIRE_OCD_CMD_RESUME, 8U);
}

uint32_t dwire_hw_read_dcsr(void) {
    dwire_hw_write_bits((uint32_t)DWIRE_OCD_CMD_READ_DCSR, 8U);
    return dwire_hw_read_bits(32U);
}

bool dwire_hw_is_cpu_halted(void) {
    const uint32_t dcsr = dwire_hw_read_dcsr();
    /* Open bus returns 0xFFFFFFFF (pull-up) or 0x00000000 (ground) */
    if (dcsr == DWIRE_BUS_RESP_OPEN_HIGH || dcsr == DWIRE_BUS_RESP_OPEN_LOW) {
        return false;
    }
    /* Bit 0 is the CPU Halt status flag in AM34 DCSR register */
    return ((dcsr & 0x00000001UL) != 0U);
}
