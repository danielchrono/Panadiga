/**
 * @file dwire32a_types.h
 * @brief Strict type definitions, register bitfields, and physical constants
 *        for the Panasonic DWire32A / Matsushita AM34 OCD interface.
 *
 * Target Processor: Panasonic UniPhier / Matsushita AM34 / MN103S SoC Family (MN2DS0004 / MN2DS0009 / MN2DS0016 / MN2DS0018)
 * Target Hardware:  Panasonic DIGA Series (DMR-E / DMR-ExxH / DMR-ES / DMR-EH / DMR-EX / DMR-EZ — 2MB / 4MB / 8MB / 16MB NOR Flash)
 * Host Controller:  Universal RP2 Family (RP2040 / RP2350 Dual-Core)
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef DWIRE32A_TYPES_H
#define DWIRE32A_TYPES_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* =========================================================================
 * Physical GPIO Pin Allocations (RP2 Family - RP2040 / RP2350)
 * ========================================================================= */

/**
 * @brief Host GPIO pin assignments for the DWire32A CN55001 connector.
 */
enum dwire_gpio_pin {
    DWIRE_PIN_EXTRG   = 2,  /**< GP2 (Pin 4): External Trigger (OCD stop request) */
    DWIRE_PIN_SCLK    = 3,  /**< GP3 (Pin 5): Synchronous Serial Clock Output */
    DWIRE_PIN_SDATA   = 4,  /**< GP4 (Pin 6): Bidirectional Half-Duplex Data */
    DWIRE_PIN_NRESET  = 5,  /**< GP5 (Pin 7): Active-Low Target Reset (Open-Drain) */
    DWIRE_PIN_VDD_ADC = 26, /**< GP26 (Pin 31 / ADC0): Target 3.3V VDD Sense */
};
typedef enum dwire_gpio_pin dwire_gpio_pin_t;

/**
 * @brief ADC Input Channel index corresponding to DWIRE_PIN_VDD_ADC (GP26).
 */
enum dwire_adc_channel {
    DWIRE_ADC_CH_VDD = 0,   /**< ADC Channel 0 corresponds to GPIO 26 */
};
typedef enum dwire_adc_channel dwire_adc_channel_t;

/* =========================================================================
 * Electrical and Temporal Constants
 * ========================================================================= */

/** Nominal DWire32A synchronous bus bit-clock frequency (1 MHz). */
#define DWIRE_BUS_FREQ_HZ         1000000UL

/** RP2040 PIO state machine execution frequency (4 cycles per bit = 4 MHz). */
#define DWIRE_SM_FREQ_HZ          4000000UL

/** Threshold voltage below which the target is considered unpowered or disconnected. */
#define DWIRE_VDD_THRESHOLD_V     2.5f

/**
 * @brief Upper sanity boundary for a VDD reading (above any legitimate 3.3 V rail).
 *
 * Floating-pin detection is performed ACTIVELY inside dwire_hw_read_vdd() by
 * engaging the internal pull-down during the measurement: an unconnected GP26
 * collapses to ~0 V while a real rail behind the 1 kOhm series resistor stays
 * at ~3.2-3.3 V.  A legitimately powered target (e.g. 3.24 V) must therefore
 * NEVER be classified as floating, so this ceiling sits above the ADC full scale.
 */
#define DWIRE_VDD_FLOATING_CEIL_V 3.45f

/** Settling time (us) after engaging the GP26 pull-down before sampling the ADC. */
#define DWIRE_ADC_PULLDOWN_SETTLE_US 300U

/**
 * @brief Number of ADC samples averaged per VDD measurement.
 *
 * Averaging 8 successive samples at the RP2040 SAR ADC rate (~0.5 µs/sample)
 * takes ≤ 4 µs and reduces quantisation noise by √8 ≈ 2.8 LSB.
 */
#define DWIRE_ADC_SAMPLE_COUNT    8U

/** RP2040 ADC reference voltage (3.3V VREG/AVDD). */
#define DWIRE_ADC_VREF            3.3f

/** RP2040 12-bit successive approximation ADC full-scale resolution (2^12). */
#define DWIRE_ADC_RESOLUTION     4096.0f

/** Target reset assertion hold time (milliseconds) for stable brownout detection. */
#define DWIRE_RESET_ASSERT_MS     10U

/** Target reset release settle delay (microseconds) for pull-up rise time. */
#define DWIRE_RESET_SETTLE_US     10U

/** Post-reset settling window (microseconds) before initial command transaction. */
#define DWIRE_POST_RESET_DELAY_US 200U

/** Clock count required during reset to wake up the internal OCD clock domain. */
#define DWIRE_BREAKIN_CLOCK_COUNT 32U

/** Startup delay after host USB enumeration before starting bus operations. */
#define DWIRE_HOST_BOOT_DELAY_MS  2000U

/* =========================================================================
 * Cold Break-In Hardware Modes
 * ========================================================================= */

/**
 * @brief Cold Break-In modes for capturing the AM34 core out of reset.
 *
 * The Matsushita MN103S / AM34 On-Chip Debugger (OCD) requires hardware
 * synchronization during the reset de-assertion phase to halt the core
 * before execution begins at the boot vector (0x80000000).
 *
 * Bit 0: EXTRG Polarity (0 = Idle Low / Active High, 1 = Idle High / Active Low)
 * Bit 1: Post-Reset Burst (0 = 200 us delay, 1 = immediate 32-clock burst)
 */
enum dwire_breakin_mode {
    /**
     * Mode 0: EXTRG Idle LOW -> Active HIGH upon reset release.
     *         Generates 32 clocks while nRESET=0, then settles for 200 us.
     */
    DWIRE_BREAKIN_MODE_EXTRG_HIGH_PRE_ONLY = 0,

    /**
     * Mode 1: EXTRG Idle HIGH -> Active LOW upon reset release (inverted trigger).
     *         Generates 32 clocks while nRESET=0, then settles for 200 us.
     */
    DWIRE_BREAKIN_MODE_EXTRG_LOW_PRE_ONLY  = 1,

    /**
     * Mode 2: EXTRG Idle LOW -> Active HIGH upon reset release.
     *         Generates 32 clocks during reset + immediate 32 clocks post-reset.
     */
    DWIRE_BREAKIN_MODE_EXTRG_HIGH_PRE_POST = 2,

    /**
     * Mode 3: EXTRG Idle HIGH -> Active LOW upon reset release (inverted trigger).
     *         Generates 32 clocks during reset + immediate 32 clocks post-reset.
     */
    DWIRE_BREAKIN_MODE_EXTRG_LOW_PRE_POST  = 3,

    /** Total number of break-in modes evaluated during opcode discovery. */
    DWIRE_BREAKIN_MODE_COUNT               = 4,
};
typedef enum dwire_breakin_mode dwire_breakin_mode_t;

/** Break-in bitmask flags. */
#define DWIRE_BREAKIN_FLAG_EXTRG_INVERT  (1U << 0) /**< Bit 0: Active-low trigger */
#define DWIRE_BREAKIN_FLAG_POST_BURST    (1U << 1) /**< Bit 1: Immediate post-reset clock burst */

/* =========================================================================
 * PIO Half-Duplex Bus Protocol and FIFO Header Formatting
 * ========================================================================= */

/**
 * @brief Direction flag for DWire32A half-duplex transaction.
 */
enum dwire_direction {
    DWIRE_DIR_READ  = 0, /**< Master receives data from target (SDATA = Hi-Z/Input) */
    DWIRE_DIR_WRITE = 1, /**< Master transmits data to target (SDATA = Output) */
};
typedef enum dwire_direction dwire_direction_t;

/** Field position and mask definitions for the RP2040 PIO FIFO header word. */
#define DWIRE_HEADER_COUNT_MASK   0x000000FFUL /**< Bits [7:0]: (bit_count - 1) */
#define DWIRE_HEADER_COUNT_SHIFT  0U
#define DWIRE_HEADER_DIR_MASK     0x00000100UL /**< Bit [8]: Transfer Direction */
#define DWIRE_HEADER_DIR_SHIFT    8U
#define DWIRE_HEADER_RESERVED_MASK 0xFFFFFE00UL

/**
 * @brief 32-bit Header word structure written to PIO TX FIFO preceding each transaction.
 *
 * Memory layout matches RP2040 PIO shift-right (LSB first) deserialization:
 * - Bits [7:0]: (n_bits - 1), loaded into scratch register X.
 * - Bit  [8]:   Direction (0=RX, 1=TX), loaded into scratch register Y.
 * - Bits [31:9]: Reserved (cleared to 0).
 */
typedef union dwire_pio_header {
    struct {
        uint32_t bit_count_minus_one : 8;  /**< Number of bits minus 1 */
        uint32_t direction           : 1;  /**< 0 = Read (RX), 1 = Write (TX) */
        uint32_t reserved            : 23; /**< Unused bits */
    } fields;
    uint32_t raw;
} dwire_pio_header_t;

/**
 * @brief Construct a 32-bit DWire32A PIO header word without magic numbers.
 *
 * @param direction Direction enum (DWIRE_DIR_READ or DWIRE_DIR_WRITE).
 * @param bit_count Number of bits to transfer (1 to 32).
 * @return Formatted 32-bit header word for RP2040 PIO TX FIFO.
 */
static inline uint32_t dwire_make_header(dwire_direction_t direction, uint8_t bit_count) {
    uint32_t count_val = (bit_count > 0U) ? ((uint32_t)bit_count - 1U) : 0U;
    return (((uint32_t)direction & 0x01UL) << DWIRE_HEADER_DIR_SHIFT) |
           ((count_val & DWIRE_HEADER_COUNT_MASK) << DWIRE_HEADER_COUNT_SHIFT);
}

/* =========================================================================
 * Protocol Validation & Scanner Constants
 * ========================================================================= */

/** Undriven bus floating high (pull-up idle state, no slave responding). */
#define DWIRE_BUS_RESP_OPEN_HIGH  0xFFFFFFFFUL

/** Undriven bus pulled low or grounded (short circuit or ground rail). */
#define DWIRE_BUS_RESP_OPEN_LOW   0x00000000UL

/** Opcode scanner sweep parameters. */
#define DWIRE_OPCODE_MIN          0x00U
#define DWIRE_OPCODE_MAX          0xFFU
#define DWIRE_OPCODE_BIT_LEN      8U
#define DWIRE_RESP_BIT_LEN        32U

/* =========================================================================
 * Panasonic / Matsushita DWire32A On-Chip Debugger (OCD) Protocol Commands
 * ========================================================================= */

#define DWIRE_OCD_CMD_NOP             0x00U /**< No-operation / bus idle */
#define DWIRE_OCD_CMD_READ_DCSR       0x01U /**< Read Debug Control & Status Register */
#define DWIRE_OCD_CMD_WRITE_DCSR      0x02U /**< Write Debug Control & Status Register */
#define DWIRE_OCD_CMD_READ_MEM32      0x10U /**< Read 32-bit word from memory bus */
#define DWIRE_OCD_CMD_WRITE_MEM32     0x11U /**< Write 32-bit word to memory bus */
#define DWIRE_OCD_CMD_READ_PC         0x20U /**< Read CPU Program Counter */
#define DWIRE_OCD_CMD_WRITE_PC        0x21U /**< Write CPU Program Counter */
#define DWIRE_OCD_CMD_RESUME          0x30U /**< Resume core execution */
#define DWIRE_OCD_CMD_HALT            0x31U /**< Force core into OCD Halt */
#define DWIRE_OCD_CMD_STEP            0x32U /**< Execute single instruction step */

/* =========================================================================
 * Return Status Codes
 * ========================================================================= */

/**
 * @brief Status and error codes for DWire32A driver operations.
 */
enum dwire_status {
    DWIRE_OK                  = 0,  /**< Operation completed successfully */
    DWIRE_ERR_INVALID_PARAM   = -1, /**< Invalid parameter passed to function */
    DWIRE_ERR_TARGET_UNPOWERED= -2, /**< VDD voltage is below minimum threshold */
    DWIRE_ERR_PIO_INIT_FAILED = -3, /**< Failed to allocate PIO state machine */
    DWIRE_ERR_TIMEOUT         = -4, /**< Target response timed out */
    DWIRE_ERR_VERIFY_FAILED   = -5, /**< Memory verification mismatch */
};
typedef enum dwire_status dwire_status_t;

#ifdef __cplusplus
}
#endif

#endif /* DWIRE32A_TYPES_H */
