/**
 * @file gdb_rsp_cpu.c
 * @brief GDB RSP AM34 / MN103S register file, execution control, and query dispatcher.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include "gdb/gdb_rsp_internal.h"
#include "hal/p7501_hw.h"
#include <string.h>

static am34_reg_file_t s_reg_file;

static uint32_t reg_file_get(size_t idx) {
    switch (idx) {
        case AM34_REG_D0:  return s_reg_file.d[0];
        case AM34_REG_D1:  return s_reg_file.d[1];
        case AM34_REG_D2:  return s_reg_file.d[2];
        case AM34_REG_D3:  return s_reg_file.d[3];
        case AM34_REG_A0:  return s_reg_file.a[0];
        case AM34_REG_A1:  return s_reg_file.a[1];
        case AM34_REG_A2:  return s_reg_file.a[2];
        case AM34_REG_A3:  return s_reg_file.a[3];
        case AM34_REG_MDR: return s_reg_file.mdr;
        case AM34_REG_PSW: return s_reg_file.psw;
        case AM34_REG_PC:  return s_reg_file.pc;
        case AM34_REG_SP:  return s_reg_file.sp;
        default:           return 0U;
    }
}

static void reg_file_set(size_t idx, uint32_t val) {
    switch (idx) {
        case AM34_REG_D0:  s_reg_file.d[0] = val; break;
        case AM34_REG_D1:  s_reg_file.d[1] = val; break;
        case AM34_REG_D2:  s_reg_file.d[2] = val; break;
        case AM34_REG_D3:  s_reg_file.d[3] = val; break;
        case AM34_REG_A0:  s_reg_file.a[0] = val; break;
        case AM34_REG_A1:  s_reg_file.a[1] = val; break;
        case AM34_REG_A2:  s_reg_file.a[2] = val; break;
        case AM34_REG_A3:  s_reg_file.a[3] = val; break;
        case AM34_REG_MDR: s_reg_file.mdr = val; break;
        case AM34_REG_PSW: s_reg_file.psw = val; break;
        case AM34_REG_PC:  s_reg_file.pc = val; break;
        case AM34_REG_SP:  s_reg_file.sp = val; break;
        default:           break;
    }
}

void gdb_cpu_init_registers(void) {
    memset(&s_reg_file, 0, sizeof(s_reg_file));
    s_reg_file.pc = 0x80000000UL;
    s_reg_file.sp = 0x0003FFF0UL;
}

void gdb_cpu_sync_registers(void) {
    s_reg_file.is_halted = p7501_hw_is_cpu_halted();
    const uint32_t dcsr = p7501_hw_read_dcsr();
    if ((dcsr != P7501_BUS_RESP_OPEN_HIGH) && (dcsr != P7501_BUS_RESP_OPEN_LOW)) {
        s_reg_file.psw = (s_reg_file.psw & 0xFFFF0000UL) | (dcsr & 0x0000FFFFUL);
    }
    if (s_reg_file.pc == 0U) {
        s_reg_file.pc = 0x80000000UL;
    }
    if (s_reg_file.sp == 0U) {
        s_reg_file.sp = 0x0003FFF0UL;
    }
}

void gdb_cpu_handle_ctrl_c(void) {
    p7501_hw_sw_uart_set_low(false);
    s_reg_file.is_halted = true;
    gdb_rsp_send_packet("S05");
}

static void handle_query_cmd(const char *cmd, char *resp_buf, size_t resp_cap) {
    if (strncmp(cmd, "qSupported", 10U) == 0) {
        gdb_rsp_send_packet("PacketSize=800;qXfer:memory-map:read+;qAttached+");
    } else if (strncmp(cmd, "qXfer:memory-map:read::", 23U) == 0) {
        gdb_mem_send_memory_map_xml(resp_buf, resp_cap);
    } else if (strcmp(cmd, "qAttached") == 0) {
        gdb_rsp_send_packet("1");
    } else if (strcmp(cmd, "qOffsets") == 0) {
        gdb_rsp_send_packet("Text=0;Data=0;Bss=0");
    } else {
        gdb_rsp_send_packet("");
    }
}

static void handle_read_regs(char *resp_buf) {
    gdb_cpu_sync_registers();
    char *ptr = resp_buf;
    for (size_t i = 0U; i < AM34_REG_COUNT; i++) {
        const uint32_t val = reg_file_get(i);
        for (int b = 0; b < 4; b++) {
            const uint8_t byte = (uint8_t)((val >> (b * 8)) & 0xFFU);
            *ptr++ = gdb_nibble_to_hex_char((uint8_t)(byte >> 4));
            *ptr++ = gdb_nibble_to_hex_char((uint8_t)(byte & 0x0FU));
        }
    }
    *ptr = '\0';
    gdb_rsp_send_packet(resp_buf);
}

static void handle_write_regs(const char *hex_str) {
    if (strlen(hex_str) < (AM34_REG_COUNT * 8U)) {
        gdb_rsp_send_packet("E01");
        return;
    }
    for (size_t i = 0U; i < AM34_REG_COUNT; i++) {
        uint32_t word = 0U;
        for (int b = 0; b < 4; b++) {
            const int h = gdb_hex_char_to_nibble(*hex_str++);
            const int l = gdb_hex_char_to_nibble(*hex_str++);
            if ((h >= 0) && (l >= 0)) {
                const uint8_t byte = (uint8_t)((h << 4) | l);
                word |= ((uint32_t)byte) << (b * 8);
            }
        }
        reg_file_set(i, word);
    }
    p7501_hw_set_pc(s_reg_file.pc);
    gdb_rsp_send_packet("OK");
}

static bool apply_optional_pc_arg(const char *arg) {
    if (*arg == '\0') {
        return true;
    }
    uint32_t target_addr = 0U;
    if (gdb_parse_hex_u32(arg, &target_addr, '\0') == NULL) {
        return false;
    }
    s_reg_file.pc = target_addr;
    p7501_hw_set_pc(target_addr);
    return true;
}

void gdb_cpu_dispatch_command(const char *cmd) {
    char resp_buf[GDB_RSP_MAX_PACKET_LEN];
    resp_buf[0] = '\0';

    switch (cmd[0]) {
        case '?':
            if (p7501_hw_read_dcsr() == P7501_BUS_RESP_OPEN_HIGH) {
                gdb_rsp_send_packet("S00");
            } else {
                gdb_cpu_sync_registers();
                gdb_rsp_send_packet(s_reg_file.is_halted ? "S05" : "S00");
            }
            break;

        case 'q':
            handle_query_cmd(cmd, resp_buf, sizeof(resp_buf));
            break;

        case 'g':
            handle_read_regs(resp_buf);
            break;

        case 'G':
            if (p7501_hw_read_dcsr() == P7501_BUS_RESP_OPEN_HIGH) {
                gdb_rsp_send_packet("E01");
            } else {
                handle_write_regs(&cmd[1]);
            }
            break;

        case 'm':
            gdb_mem_handle_read_cmd(&cmd[1], resp_buf);
            break;

        case 'M':
            gdb_mem_handle_write_cmd(&cmd[1]);
            break;

        case 'c':
            if (p7501_hw_read_dcsr() == P7501_BUS_RESP_OPEN_HIGH ||
                !apply_optional_pc_arg(&cmd[1])) {
                gdb_rsp_send_packet("E01");
                break;
            }
            p7501_hw_resume_cpu();
            s_reg_file.is_halted = false;
            gdb_rsp_send_packet("S05");
            break;

        case 's':
            if (p7501_hw_read_dcsr() == P7501_BUS_RESP_OPEN_HIGH ||
                !apply_optional_pc_arg(&cmd[1])) {
                gdb_rsp_send_packet("E01");
                break;
            }
            s_reg_file.pc += 2U;
            p7501_hw_set_pc(s_reg_file.pc);
            gdb_rsp_send_packet("S05");
            break;

        case 'k':
            p7501_hw_reset_assert();
            sleep_ms(P7501_RESET_ASSERT_MS);
            p7501_hw_reset_release();
            gdb_rsp_invalidate_cache();
            gdb_rsp_send_packet("OK");
            break;

        case 'D':
            gdb_rsp_send_packet("OK");
            gdb_rsp_set_active_flag(false);
            break;

        case 'R':
            p7501_hw_boot_sequence(P7501_BOOT_MODE_NORMAL_HIZ);
            gdb_rsp_invalidate_cache();
            s_reg_file.pc = 0x80000000UL;
            gdb_rsp_send_packet("OK");
            break;

        case 'Z':
        case 'z':
            gdb_rsp_send_packet(((cmd[1] == '0') || (cmd[1] == '1')) ? "OK" : "");
            break;

        default:
            gdb_rsp_send_packet("");
            break;
    }
}
