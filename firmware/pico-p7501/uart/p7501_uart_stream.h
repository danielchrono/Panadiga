/**
 * @file p7501_uart_stream.h
 * @brief PIO1 real-time UART streaming (`UARTSTREAM`) and microsecond hex capture (`HEXCAP`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_UART_STREAM_H
#define P7501_UART_STREAM_H

#ifdef __cplusplus
extern "C" {
#endif

/** Stream real-time decoded target UART RX bytes on GP2 (`UARTSTREAM [ms] [boot]`). */
void p7501_cmd_uart_stream(const char *arg);

/** Capture microsecond-timestamped hex + ASCII frames on GP2 (`HEXCAP [ms] [boot]`). */
void p7501_cmd_hexcap(const char *arg);

#ifdef __cplusplus
}
#endif

#endif /* P7501_UART_STREAM_H */
