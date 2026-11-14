/**
 * @file p7501_uart_baud.h
 * @brief PIO1 multi-baud runtime configuration and edge-width autobaud scanner (`BAUD`, `BAUDSCAN`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_UART_BAUD_H
#define P7501_UART_BAUD_H

#ifdef __cplusplus
extern "C" {
#endif

/** Query or set PIO1 RX/TX baudrate (`BAUD [bps]`). */
void p7501_cmd_baud(const char *arg);

/** Measure GP2 pulse widths during boot/window and snap to standard baudrate (`BAUDSCAN [ms] [boot] [APPLY]`). */
void p7501_cmd_baudscan(const char *arg);

#ifdef __cplusplus
}
#endif

#endif /* P7501_UART_BAUD_H */
