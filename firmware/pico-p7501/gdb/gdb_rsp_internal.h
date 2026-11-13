/**
 * @file gdb_rsp_internal.h
 * @brief Internal shared interfaces across GDB RSP codec, memory cache, and AM34 CPU modules.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#ifndef GDB_RSP_INTERNAL_H
#define GDB_RSP_INTERNAL_H

#include "gdb/gdb_rsp.h"

#ifdef __cplusplus
extern "C" {
#endif

#if defined(PICO_HOST_TEST) && (PICO_HOST_TEST != 0)
void sleep_ms(uint32_t ms);
#else
#include "pico/stdlib.h"
#endif

int gdb_hex_char_to_nibble(char c);
char gdb_nibble_to_hex_char(uint8_t n);
uint8_t gdb_compute_checksum(const char *payload);
const char *gdb_parse_hex_u32(const char *str, uint32_t *out_val, char delim);
const char *gdb_parse_addr_and_len(
    const char *str,
    uint32_t *out_addr,
    uint32_t *out_len,
    char end_delim
);

void gdb_rsp_set_active_flag(bool active);

void gdb_mem_handle_read_cmd(const char *arg, char *resp_buf);
void gdb_mem_handle_write_cmd(const char *arg);
void gdb_mem_send_memory_map_xml(char *resp_buf, size_t resp_cap);

void gdb_cpu_init_registers(void);
void gdb_cpu_sync_registers(void);
void gdb_cpu_handle_ctrl_c(void);
void gdb_cpu_dispatch_command(const char *cmd);

#ifdef __cplusplus
}
#endif

#endif /* GDB_RSP_INTERNAL_H */
