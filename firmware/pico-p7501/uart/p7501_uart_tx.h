/**
 * @file p7501_uart_tx.h
 * @brief PIO1 active TX (`UARTTXHEX`), pattern-triggered injection (`WAITTX`), and `BRIDGE`.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_UART_TX_H
#define P7501_UART_TX_H

#ifdef __cplusplus
extern "C" {
#endif

/** Transmit hex bytes or BREAK over GP4/Q7514 (`UARTTXHEX [LOW:]<hex|BRK> [rx_ms]`). */
void p7501_cmd_uart_tx_hex(const char *arg);

/** Boot/listen for trigger substring on GP2 then inject hex/BREAK (`WAITTX <ms> <mode> "<trig>" <hex|BRK> [post_ms]`). */
void p7501_cmd_waittx(const char *arg);

/** Transparent full-duplex USB CDC <-> SC0 PIO1 passthrough (`BRIDGE [baud] [NORMAL|JIG|JIG:<ms>|NOPULL]`, exit `~~~`). */
void p7501_cmd_bridge(const char *arg);

#ifdef __cplusplus
}
#endif

#endif /* P7501_UART_TX_H */
