/**
 * @file p7501_diag_pwrtrace.h
 * @brief P7501 boot power-rail and digital line glitch monitor (`PWRTRACE`).
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_DIAG_PWRTRACE_H
#define P7501_DIAG_PWRTRACE_H

#ifdef __cplusplus
extern "C" {
#endif

/** Trace SW_UART ADC0 voltage min/max and NSW5V/RXD/TXD transitions during boot (`PWRTRACE [ms] [boot]`). */
void p7501_cmd_pwrtrace(const char *arg);

#ifdef __cplusplus
}
#endif

#endif /* P7501_DIAG_PWRTRACE_H */
