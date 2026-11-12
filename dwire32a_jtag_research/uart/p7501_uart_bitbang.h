/**
 * @file p7501_uart_bitbang.h
 * @brief Low-level 115200 8N1 bit-banged UART primitives for connector P7501.
 *
 * Electrical mapping for Panasonic P7501 Service UART / Serial Boot mode:
 *   - GP2  (Pin 6 MAX TXD) : UART RX (Target -> Pico)
 *   - GP4  (Pin 3 MAX RXD) : UART TX (Pico -> Target)
 *   - GP3  (Pin 5 NSW5V)   : Kept in High-Z Input (never driven!)
 *   - GP26 (Pin 1 SW_UART) : Driven LOW to assert XFWE serial boot enable
 *   - GP5  (Pin 2 XRESET)  : Open-drain target reset
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_UART_BITBANG_H
#define P7501_UART_BITBANG_H

#include <stddef.h>
#include "dwire32a_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Configure P7501 GPIOs for 115200 8N1 UART / FMU instrumentation.
 */
void p7501_uart_configure_pins(
    bool sw_uart_out_low,
    bool config_sdata_tx,
    bool sdata_initial_high,
    bool use_12ma_drive
);

/**
 * @brief Restore GP26 to ADC0 Hi-Z and leave bus safely isolated with GP2 pull-up.
 */
void p7501_uart_restore_default_state(void);

/**
 * @brief Execute 15 ms open-drain reset pulse and wait up to 5 ms for GP2 idle HIGH.
 */
void p7501_uart_pulse_reset_and_settle(void);

/**
 * @brief Transmit a single byte at 115200 8N1 over GP4 (MAX RXD).
 */
void p7501_uart_tx_byte(uint8_t byte_val);

/**
 * @brief Transmit a byte buffer sequentially at 115200 8N1 over GP4 (MAX RXD).
 */
void p7501_uart_tx_bytes(const uint8_t *data, size_t len, uint32_t inter_byte_delay_us);

/**
 * @brief Non-blocking falling-edge detector & 115200 8N1 byte sampler on GP2 (MAX TXD).
 *
 * @param[in,out] prev_high  Previous GP2 state tracked across calls.
 * @param[out]    out_byte   Decoded 8-bit UART byte if valid start bit detected.
 * @param[out]    out_t0_us  Optional microsecond timestamp of falling start edge.
 * @return true if a complete byte was sampled, false otherwise.
 */
bool p7501_uart_try_sample_byte(bool *prev_high, uint8_t *out_byte, uint32_t *out_t0_us);

/**
 * @brief Print captured binary/ASCII UART buffer using `<XX>` escapes for control bytes.
 */
void p7501_uart_print_ascii_stream(const uint8_t *buf, uint32_t len);

#ifdef __cplusplus
}
#endif

#endif /* P7501_UART_BITBANG_H */
