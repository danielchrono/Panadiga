/**
 * @file dwire32a_cli.h
 * @brief Interactive USB CDC command-line interface and dispatcher for Panadiga DWire32A probe.
 *
 * Separates command parsing, help formatting, safety interlocks, and subsystem routing
 * from the microcontroller initialization & main polling loop.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef DWIRE32A_CLI_H
#define DWIRE32A_CLI_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Trim leading and trailing ASCII whitespace in-place.
 *
 * @param str Mutable string buffer (may be NULL).
 * @return Pointer to first non-whitespace character inside `str` (or NULL).
 */
char *dwire_cli_trim_whitespace(char *str);

/**
 * @brief Print startup identification banner and hardware status summary.
 */
void dwire_cli_print_banner(void);

/**
 * @brief Print complete command reference grouped by functional module.
 */
void dwire_cli_print_help(void);

/**
 * @brief Parse and execute a single null-terminated USB CDC CLI command line.
 *
 * @param line Mutable command line buffer received over USB CDC.
 */
void dwire_cli_process_command(char *line);

#ifdef __cplusplus
}
#endif

#endif /* DWIRE32A_CLI_H */
