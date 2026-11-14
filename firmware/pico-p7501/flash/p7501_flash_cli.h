/**
 * @file p7501_flash_cli.h
 * @brief Flash geometry probe, CFI query, SRAM loader injection, and NVRAM erase CLI handlers.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef P7501_FLASH_CLI_H
#define P7501_FLASH_CLI_H

#ifdef __cplusplus
extern "C" {
#endif

void p7501_flash_cmd_autodetect(const char *arg);
void p7501_flash_cmd_cfi(const char *arg);
void p7501_flash_cmd_size(const char *arg);
void p7501_flash_cmd_inject_loader(const char *arg);
void p7501_flash_cmd_erase_nvram(const char *arg);

#ifdef __cplusplus
}
#endif

#endif /* P7501_FLASH_CLI_H */
