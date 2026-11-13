/**
 * @file p7501_uart_capture.h
 * @brief Shared 16 KB P7501 UART RX capture engine & passive boot monitor (`BOOTCAP`/`UARTMON`/`JIGBOOT`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_UART_CAPTURE_H
#define P7501_UART_CAPTURE_H

#include <stddef.h>
#include "p7501_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Access the single shared 16 KB P7501 UART RX capture buffer.
 *
 * @param[out] out_capacity Optional pointer receiving buffer size in bytes.
 * @return Pointer to shared capture buffer.
 */
uint8_t *p7501_uart_rx_buffer(size_t *out_capacity);

/**
 * @brief Poll GP2 (MAX TXD) and append decoded byte to the shared RX buffer if room permits.
 */
bool p7501_uart_poll_capture_byte(bool *prev_high, uint32_t *rx_len, uint32_t max_cap);

/**
 * @brief Capture 115200 8N1 serial traffic into the shared RX buffer for `dur_us` microseconds.
 */
uint32_t p7501_uart_capture_window(uint32_t dur_us, uint32_t max_cap);

/**
 * @brief Execute passive or reset-synchronized 115200 UART capture (`BOOTCAP`/`UARTMON`/`JIGBOOT`).
 */
void p7501_cmd_boot_capture(const char *arg, bool do_reset, bool is_jig, const char *cmd_name);

#ifdef __cplusplus
}
#endif

#endif /* P7501_UART_CAPTURE_H */
