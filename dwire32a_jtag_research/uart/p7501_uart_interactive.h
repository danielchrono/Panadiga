/**
 * @file p7501_uart_interactive.h
 * @brief Interactive P7501 line command & BREAK probe (`UARTSEND`, `UARTPROBE`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_UART_INTERACTIVE_H
#define P7501_UART_INTERACTIVE_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Transmit an ASCII line + CRLF over SW_UART=LOW P7501 and capture reply (`UARTSEND`).
 */
void p7501_cmd_uart_send(const char *arg);

/**
 * @brief Multi-mode UART TX/BREAK probe over P7501 lines (`UARTPROBE`).
 */
void p7501_cmd_uart_probe(const char *arg);

#ifdef __cplusplus
}
#endif

#endif /* P7501_UART_INTERACTIVE_H */
