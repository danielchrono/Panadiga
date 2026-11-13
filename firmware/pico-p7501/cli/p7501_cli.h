/**
 * @file p7501_cli.h
 * @brief Interactive USB CDC command-line interface and dispatcher for Panadiga P7501 probe.
 *
 * Separates command parsing, help formatting, safety interlocks, and subsystem routing
 * from the microcontroller initialization & main polling loop.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_CLI_H
#define P7501_CLI_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Trim leading and trailing ASCII whitespace in-place.
 *
 * @param str Mutable string buffer (may be NULL).
 * @return Pointer to first non-whitespace character inside `str` (or NULL).
 */
char *p7501_cli_trim_whitespace(char *str);

/**
 * @brief Print startup identification banner and hardware status summary.
 */
void p7501_cli_print_banner(void);

/**
 * @brief Print complete command reference grouped by functional module.
 */
void p7501_cli_print_help(void);

/**
 * @brief Parse and execute a single null-terminated USB CDC CLI command line.
 *
 * @param line Mutable command line buffer received over USB CDC.
 */
void p7501_cli_process_command(char *line);

#ifdef __cplusplus
}
#endif

#endif /* P7501_CLI_H */
