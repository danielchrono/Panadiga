/**
 * @file p7501_cli_help.h
 * @brief Banner, MCU identification, and CLI command reference output for Panadiga P7501.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_CLI_HELP_H
#define P7501_CLI_HELP_H

#ifdef __cplusplus
extern "C" {
#endif

/** Print identification banner, active MCU, P7501 pinout summary, and GDB status. */
void p7501_cli_print_banner(void);

/** Print grouped bench CLI command reference (electrical, UART PIO1, FMU, Flash, GDB, LED). */
void p7501_cli_print_help(void);

#ifdef __cplusplus
}
#endif

#endif /* P7501_CLI_HELP_H */
