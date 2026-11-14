/**
 * @file p7501_uart_pio_engine.h
 * @brief Multi-baud PIO1 RX/TX engine, Safe XRESET/SW_UART session control, and GP2 timing.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_UART_PIO_ENGINE_H
#define P7501_UART_PIO_ENGINE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "uart/p7501_uart_codec.h"

#ifdef __cplusplus
extern "C" {
#endif

uint32_t p7501_pio_get_baud(void);
bool p7501_pio_set_baud(uint32_t baud);

void p7501_pio_enforce_nsw5v_hiz(void);
void p7501_pio_set_sw_uart(bool drive_low);
void p7501_pio_pulse_xreset_ms(uint32_t hold_low_ms);

void p7501_pio_begin_rx_session(p7501_boot_action_t action);
void p7501_pio_begin_rx_session_ex(p7501_boot_action_t action, uint32_t jig_hold_ms);
bool p7501_pio_poll_sw_uart_release(void);
void p7501_pio_end_session(void);

bool p7501_pio_pop_rx_byte(uint8_t *out_byte);
bool p7501_pio_poll_and_clear_framing_err(void);
bool p7501_pio_had_rx_overflow(void);

void p7501_pio_tx_bytes(const uint8_t *data, size_t len, bool force_sw_uart_low);
void p7501_pio_tx_break_ms(uint32_t break_ms, bool force_sw_uart_low);

#ifdef __cplusplus
}
#endif

#endif /* P7501_UART_PIO_ENGINE_H */
