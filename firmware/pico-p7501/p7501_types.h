/**
 * @file p7501_types.h
 * @brief Strict type definitions, P7501 GPIO mapping, and physical constants
 *        for the Panasonic DMR-EH55 P7501 Async UART (115200 8N1) & Boot Probe.
 *
 * Verified Physical Connector: P7501 (JST B6B-ZR-3R4, "TO JIGU", Main PCB NA p.55/62-64)
 * Target Digital Board:       VEP79138K via P59001 Interconnect (NA p.97)
 * Target Processor Family:    Matsushita UniPhier AM34 / MN103S
 * Host Controller:            Universal RP2 Family (RP2040 / RP2350 Dual-Core)
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_TYPES_H
#define P7501_TYPES_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* =========================================================================
 * Physical GPIO Pin Allocations (RP2040 / RP2350 -> Panasonic P7501)
 * ========================================================================= */

/**
 * @brief Host GPIO pin assignments mapped 1:1 to Panasonic P7501 (B6B-ZR-3R4).
 *
 * Electrical Safety Invariant:
 *   - GP3 (Pin 5 NSW5V / X_SW_5R2V) connects to a ~5.15V upstream rail through
 *     D7510 and MUST ALWAYS remain High-Z Input with internal pulls disabled.
 *   - GP5 (Pin 2 XRESET) is strictly Open-Drain (drive 0V LOW, release to Hi-Z).
 *   - GP2 (Pin 6 UARTP2M_T) is always input RX from the Main SoC.
 */
enum p7501_gpio_pin {
    P7501_PIN_RXD         = 2,  /**< GP2 (P7501 Pin 6 UARTP2M_T): 115200 8N1 RX (Target -> Pico) */
    P7501_PIN_NSW5V       = 3,  /**< GP3 (P7501 Pin 5 NSW5V): Rail Sense ONLY (Keep Hi-Z Input!) */
    P7501_PIN_TXD         = 4,  /**< GP4 (P7501 Pin 3 UARTM2P_T): 115200 8N1 TX (Pico -> Target) */
    P7501_PIN_XRESET      = 5,  /**< GP5 (P7501 Pin 2 XRESET): Active-Low Reset (Open-Drain ONLY) */
    P7501_PIN_SW_UART_ADC = 26, /**< GP26 (P7501 Pin 1 SW_UART / ADC0): Serial Boot Mux & VDD Sense */
};
typedef enum p7501_gpio_pin p7501_gpio_pin_t;

/**
 * @brief ADC Input Channel index corresponding to P7501_PIN_SW_UART_ADC (GP26).
 */
enum p7501_adc_channel {
    P7501_ADC_CH_VDD = 0, /**< ADC Channel 0 corresponds to GPIO 26 */
};
typedef enum p7501_adc_channel p7501_adc_channel_t;

/* =========================================================================
 * Electrical and Temporal Constants
 * ========================================================================= */

/** Nominal P7501 asynchronous UART baud rate (115200 8N1). */
#define P7501_UART_BAUD_RATE         115200UL

/** PIO UART state machine oversampling rate (8 cycles per bit = 921.6 kHz). */
#define P7501_PIO_UART_CYCLES_PER_BIT 8UL

/** Threshold voltage below which target SW_UART sense is unpowered/disconnected. */
#define P7501_VDD_THRESHOLD_V        2.5f

/**
 * @brief Pull-down discrimination floor on GP26 (ADC0).
 *
 * Connected P7501 Pin 1 clamps around 2.68-2.70 V under RP2 internal pull-down
 * (~50 kOhm against Q7510/R7562/R7559), while a floating or unpowered wire drops
 * below 1.50 V (~0.00 V).
 */
#define P7501_VDD_PULLDOWN_PRESENT_V 1.50f

/**
 * @brief Upper sanity boundary for SW_UART ADC0 sense (3.3V VREF domain).
 *
 * Floating-pin discrimination inside p7501_hw_read_vdd() engages internal
 * pull-down briefly: an unconnected GP26 collapses to ~0 V, while connected P7501
 * Pin 1 clamps around 2.68-2.70 V (via Q7510/R7562/R7559) or reads ~3.25 V Hi-Z.
 */
#define P7501_VDD_FLOATING_CEIL_V    3.45f

/** Settling time (us) after engaging GP26 pull-down before sampling ADC0. */
#define P7501_ADC_PULLDOWN_SETTLE_US 300U

/** Number of ADC samples averaged per VDD sense reading. */
#define P7501_ADC_SAMPLE_COUNT       8U

/** RP2040/RP2350 ADC reference voltage (3.3V VREG/AVDD). */
#define P7501_ADC_VREF               3.3f

/** RP2040/RP2350 12-bit SAR ADC full-scale resolution (2^12). */
#define P7501_ADC_RESOLUTION         4096.0f

/** Target open-drain XRESET hold time (ms) for clean cold reboot. */
#define P7501_RESET_ASSERT_MS        10U

/** Settle window (us) between SW_UART setup and XRESET release. */
#define P7501_RESET_SETTLE_US        10U

/** Post-reset settle window (us). */
#define P7501_POST_RESET_DELAY_US    200U

/* =========================================================================
 * Discrete P7501 Boot & Reset Sequence Profiles (SW_UART + XRESET)
 * ========================================================================= */

/**
 * @brief Boot control sequences coordinating P7501 Pin 1 (SW_UART) & Pin 2 (XRESET).
 */
enum p7501_boot_mode {
    /** Mode 0: Normal Boot — SW_UART Hi-Z + open-drain XRESET pulse (200 us settle). */
    P7501_BOOT_MODE_NORMAL_HIZ         = 0,
    /** Mode 1: Serial Mux Pulse — SW_UART LOW during XRESET, released to Hi-Z after reset. */
    P7501_BOOT_MODE_SERIAL_LOW_RELEASE = 1,
    /** Mode 2: JIG Serial Boot Hold — SW_UART held LOW across XRESET (XFWE serial boot). */
    P7501_BOOT_MODE_SERIAL_LOW_HOLD    = 2,
    /** Mode 3: Normal Hi-Z Extended Settle — SW_UART Hi-Z + extended post-reset wait. */
    P7501_BOOT_MODE_HIZ_EXTENDED       = 3,
    /** Total number of discrete P7501 boot profiles. */
    P7501_BOOT_MODE_COUNT              = 4,
};
typedef enum p7501_boot_mode p7501_boot_mode_t;

/* =========================================================================
 * P7501 Monitor / Bus Response Constants
 * ========================================================================= */

/** Idle high line response (pull-up idle, no monitor responder driving low). */
#define P7501_BUS_RESP_OPEN_HIGH     0xFFFFFFFFUL

/** Grounded/shorted response. */
#define P7501_BUS_RESP_OPEN_LOW      0x00000000UL

/** Monitor probe sweep bounds. */
#define P7501_PROBE_MIN              0x00U
#define P7501_PROBE_MAX              0xFFU

/** Logical monitor protocol opcodes (forwarded over P7501 UART monitor when attached). */
#define P7501_MON_CMD_NOP            0x00U
#define P7501_MON_CMD_READ_DCSR      0x01U
#define P7501_MON_CMD_WRITE_DCSR     0x02U
#define P7501_MON_CMD_READ_MEM32     0x10U
#define P7501_MON_CMD_WRITE_MEM32    0x11U
#define P7501_MON_CMD_READ_PC        0x20U
#define P7501_MON_CMD_WRITE_PC       0x21U
#define P7501_MON_CMD_RESUME         0x30U
#define P7501_MON_CMD_HALT           0x31U

/* =========================================================================
 * Return Status Codes
 * ========================================================================= */

enum p7501_status {
    P7501_OK                   = 0,  /**< Operation completed successfully */
    P7501_ERR_INVALID_PARAM    = -1, /**< Invalid parameter passed to function */
    P7501_ERR_TARGET_UNPOWERED = -2, /**< Target SW_UART/VDD voltage below threshold */
    P7501_ERR_PIO_INIT_FAILED  = -3, /**< Failed to allocate PIO UART state machine */
    P7501_ERR_TIMEOUT          = -4, /**< Target response timed out */
    P7501_ERR_VERIFY_FAILED    = -5, /**< Memory verification mismatch */
};
typedef enum p7501_status p7501_status_t;

#ifdef __cplusplus
}
#endif

#endif /* P7501_TYPES_H */
