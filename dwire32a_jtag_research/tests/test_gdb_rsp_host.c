/**
 * @file test_gdb_rsp_host.c
 * @brief Host-side C11 unit test suite for DWire32A GDB RSP Server.
 *
 * Verifies RSP packet framing, memory read/write protection air-gaps,
 * non-overlapping XML memory-map generation, register file access,
 * and deterministic E05 error rejection on IPL writes.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <assert.h>

#include "../gdb_rsp.h"
#include "../dwire32a_hw.h"

/* =========================================================================
 * Mock Hardware & Target Memory Subsystem
 * ========================================================================= */

#define MOCK_SRAM_SIZE      (256 * 1024)
#define MOCK_NVRAM_SIZE     (64 * 1024)
#define MOCK_FLASH_SIZE     (8 * 1024 * 1024)

static uint8_t s_mock_sram[MOCK_SRAM_SIZE];
static uint8_t s_mock_nvram[MOCK_NVRAM_SIZE];
static uint8_t s_mock_flash[MOCK_FLASH_SIZE];

static bool s_mock_halted = true;
static uint32_t s_mock_dcsr = 0x00000001UL; /* Halted flag set */
static uint32_t s_mock_pc = 0x80000000UL;

/* Mock HAL implementation */
uint32_t dwire_hw_read_mem32(uint32_t addr) {
    if (addr < MOCK_SRAM_SIZE) {
        uint32_t val;
        memcpy(&val, &s_mock_sram[addr], 4);
        return val;
    }
    if (addr >= 0x807F0000UL && addr < (0x807F0000UL + MOCK_NVRAM_SIZE)) {
        uint32_t offset = addr - 0x807F0000UL;
        uint32_t val;
        memcpy(&val, &s_mock_nvram[offset], 4);
        return val;
    }
    if (addr >= 0x80000000UL && addr < (0x80000000UL + MOCK_FLASH_SIZE)) {
        uint32_t offset = addr - 0x80000000UL;
        uint32_t val;
        memcpy(&val, &s_mock_flash[offset], 4);
        return val;
    }
    return 0xFFFFFFFFUL;
}

void dwire_hw_write_mem32(uint32_t addr, uint32_t data) {
    if (addr < MOCK_SRAM_SIZE) {
        memcpy(&s_mock_sram[addr], &data, 4);
        return;
    }
    if (addr >= 0x807F0000UL && addr < (0x807F0000UL + MOCK_NVRAM_SIZE)) {
        uint32_t offset = addr - 0x807F0000UL;
        memcpy(&s_mock_nvram[offset], &data, 4);
        return;
    }
    /* Any write to 0x80000000..0x807EFFFF is physically ignored in Flash */
}

bool dwire_hw_is_cpu_halted(void) {
    return s_mock_halted;
}

uint32_t dwire_hw_read_dcsr(void) {
    return s_mock_dcsr;
}

void dwire_hw_set_pc(uint32_t pc_val) {
    s_mock_pc = pc_val;
}

void dwire_hw_resume_cpu(void) {
    s_mock_halted = false;
    s_mock_dcsr = 0x00000000UL;
}

void dwire_hw_reset_assert(void) {}
void dwire_hw_reset_release(void) {}
void dwire_hw_cold_break_in(dwire_breakin_mode_t mode) { (void)mode; }
void dwire_hw_extrg_set(bool level) { (void)level; }
void sleep_ms(uint32_t ms) { (void)ms; }

/* =========================================================================
 * Test Verification Utilities
 * ========================================================================= */

static void reset_mock_state(void) {
    memset(s_mock_sram, 0xAA, sizeof(s_mock_sram));
    memset(s_mock_nvram, 0xFF, sizeof(s_mock_nvram));
    memset(s_mock_flash, 0x55, sizeof(s_mock_flash));
    s_mock_halted = true;
    s_mock_dcsr = 0x00000001UL;
    s_mock_pc = 0x80000000UL;
    gdb_rsp_init();
    gdb_rsp_set_active(true);
}

static void send_cmd_and_get_response(const char *cmd, char *resp_out, size_t max_len) {
    gdb_rsp_test_inject_packet(cmd);
    gdb_rsp_test_get_last_response(resp_out, max_len);
}

/* =========================================================================
 * Test Cases
 * ========================================================================= */

static void test_query_supported(void) {
    char resp[GDB_RSP_MAX_PACKET_LEN];
    send_cmd_and_get_response("qSupported", resp, sizeof(resp));
    assert(strstr(resp, "PacketSize=") != NULL);
    assert(strstr(resp, "qXfer:memory-map:read+") != NULL);
    printf("  [PASS] test_query_supported\n");
}

static void test_xml_memory_map_no_overlap(void) {
    char resp[GDB_RSP_MAX_PACKET_LEN];
    send_cmd_and_get_response("qXfer:memory-map:read::0,800", resp, sizeof(resp));

    /* Must contain non-overlapping ROM length 0x007F0000 */
    assert(strstr(resp, "start=\"0x80000000\" length=\"0x007F0000\"") != NULL);
    /* Must contain Flash NVRAM length 0x00010000 */
    assert(strstr(resp, "start=\"0x807F0000\" length=\"0x00010000\"") != NULL);
    /* Must contain SRAM at 0x00000000 */
    assert(strstr(resp, "start=\"0x00000000\" length=\"0x00040000\"") != NULL);
    /* Must not contain overlapping length 0x00800000 */
    assert(strstr(resp, "start=\"0x80000000\" length=\"0x00800000\"") == NULL);

    printf("  [PASS] test_xml_memory_map_no_overlap (OPUS-06 verified)\n");
}

static void test_sram_memory_read_write(void) {
    char resp[GDB_RSP_MAX_PACKET_LEN];

    /* Write 4 bytes to SRAM at 0x00001000 */
    send_cmd_and_get_response("M00001000,4:efbeadde", resp, sizeof(resp));
    assert(strcmp(resp, "OK") == 0);

    /* Read back 4 bytes from SRAM at 0x00001000 */
    send_cmd_and_get_response("m00001000,4", resp, sizeof(resp));
    assert(strcmp(resp, "efbeadde") == 0);

    printf("  [PASS] test_sram_memory_read_write\n");
}

static void test_nvram_memory_write_allowed(void) {
    char resp[GDB_RSP_MAX_PACKET_LEN];

    /* NVRAM microsectors SA134..SA141 at 0x807F0000 are writable for U99 recovery */
    send_cmd_and_get_response("M807F0000,4:00112233", resp, sizeof(resp));
    assert(strcmp(resp, "OK") == 0);

    send_cmd_and_get_response("m807F0000,4", resp, sizeof(resp));
    assert(strcmp(resp, "00112233") == 0);

    printf("  [PASS] test_nvram_memory_write_allowed\n");
}

static void test_ipl_write_rejection_e05(void) {
    char resp[GDB_RSP_MAX_PACKET_LEN];

    /* OPUS-07: Writing to IPL root-of-trust (0x80000000) must return E05 error code */
    send_cmd_and_get_response("M80000000,4:00000000", resp, sizeof(resp));
    assert(strcmp(resp, "E05") == 0);

    /* Writing near end of IPL (0x807EFFF0) must also return E05 */
    send_cmd_and_get_response("M807EFFF0,4:ffffffff", resp, sizeof(resp));
    assert(strcmp(resp, "E05") == 0);

    /* Writing that spans across the boundary into NVRAM must be rejected */
    send_cmd_and_get_response("M807EFFFE,4:11223344", resp, sizeof(resp));
    assert(strcmp(resp, "E05") == 0);

    /* Writing across unmapped boundary at end of SRAM (0x0003FFFE..0x00040001) must be rejected */
    send_cmd_and_get_response("M0003FFFE,4:11223344", resp, sizeof(resp));
    assert(strcmp(resp, "E05") == 0);

    /* Malformed read packet without comma delimiter must return E02 */
    send_cmd_and_get_response("m80000000", resp, sizeof(resp));
    assert(strcmp(resp, "E02") == 0);

    printf("  [PASS] test_ipl_write_rejection_e05 (OPUS-07 verified)\n");
}

static void test_register_access_no_ub(void) {
    char resp[GDB_RSP_MAX_PACKET_LEN];

    /* Read registers: must return exactly 12 * 8 = 96 hex characters */
    send_cmd_and_get_response("g", resp, sizeof(resp));
    assert(strlen(resp) == (AM34_REG_COUNT * 8));

    /* Write registers with known pattern */
    char write_cmd[GDB_RSP_MAX_PACKET_LEN];
    snprintf(write_cmd, sizeof(write_cmd), "G");
    for (size_t i = 0; i < AM34_REG_COUNT; i++) {
        strcat(write_cmd, "01020304");
    }
    send_cmd_and_get_response(write_cmd, resp, sizeof(resp));
    assert(strcmp(resp, "OK") == 0);

    /* Verify written PC was passed to dwire_hw_set_pc */
    assert(s_mock_pc == 0x04030201UL);

    printf("  [PASS] test_register_access_no_ub (OPUS-14 verified)\n");
}

static void test_halt_and_continue_status(void) {
    char resp[GDB_RSP_MAX_PACKET_LEN];

    /* CPU halted: '?' must return S05 */
    s_mock_halted = true;
    s_mock_dcsr = 0x00000001UL;
    send_cmd_and_get_response("?", resp, sizeof(resp));
    assert(strcmp(resp, "S05") == 0);

    /* CPU running: '?' must return S00 */
    s_mock_halted = false;
    s_mock_dcsr = 0x00000000UL;
    send_cmd_and_get_response("?", resp, sizeof(resp));
    assert(strcmp(resp, "S00") == 0);

    printf("  [PASS] test_halt_and_continue_status (OPUS-05 verified)\n");
}

int main(void) {
    printf("======================================================================\n");
    printf(">> Running Panadiga DWire32A C11 Host Test Suite (PICO_HOST_TEST=1)\n");
    printf("======================================================================\n");

    reset_mock_state();
    test_query_supported();

    reset_mock_state();
    test_xml_memory_map_no_overlap();

    reset_mock_state();
    test_sram_memory_read_write();

    reset_mock_state();
    test_nvram_memory_write_allowed();

    reset_mock_state();
    test_ipl_write_rejection_e05();

    reset_mock_state();
    test_register_access_no_ub();

    reset_mock_state();
    test_halt_and_continue_status();

    printf("======================================================================\n");
    printf("[SUCCESS] All 7 C11 Pico Firmware Host Unit Tests Passed Cleanly!\n");
    printf("======================================================================\n");
    return 0;
}
