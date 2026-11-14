/**
 * @file p7501_pico_main.c
 * @brief Slim RP2040 / RP2350 entrypoint for the Panadiga P7501 UART Bench Probe.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include "pico/stdlib.h"

#include "cli/p7501_cli.h"
#include "hal/p7501_hw.h"
#include "hal/p7501_led.h"
#include "p7501_types.h"
#include "gdb/gdb_rsp.h"

#define P7501_CLI_LINE_MAX 256U

static void process_cli_char(
    char c,
    char *line_buf,
    size_t *line_idx,
    size_t line_cap,
    bool *last_was_cr
) {
    if ((c == '\r') || (c == '\n')) {
        if ((c == '\n') && (*last_was_cr)) {
            *last_was_cr = false;
            return;
        }
        *last_was_cr = (c == '\r');
        line_buf[*line_idx] = '\0';
        *line_idx = 0U;

        p7501_cli_process_command(line_buf);
        if (!gdb_rsp_is_active()) {
            printf("PANADIGA> ");
            fflush(stdout);
        }
        return;
    }

    *last_was_cr = false;
    if ((c == '\b') || (c == 0x7FU)) {
        if (*line_idx > 0U) {
            (*line_idx)--;
        }
    } else if (*line_idx < (line_cap - 1U)) {
        line_buf[(*line_idx)++] = c;
    }
}

int main(void) {
    stdio_init_all();
    p7501_led_init();
    (void)p7501_hw_init();
    gdb_rsp_init();

    sleep_ms(250U);
    p7501_cli_print_banner();
    printf("PANADIGA> ");
    fflush(stdout);

    char line_buf[P7501_CLI_LINE_MAX];
    size_t line_idx = 0U;
    bool last_was_cr = false;

    for (;;) {
        p7501_led_tick();

        const int ch = getchar_timeout_us(0);
        if (ch == PICO_ERROR_TIMEOUT) {
            tight_loop_contents();
            continue;
        }

        const char c = (char)ch;
        if (gdb_rsp_is_active()) {
            gdb_rsp_process_char(c);
            if (!gdb_rsp_is_active()) {
                p7501_led_set_auto_mode(LED_MODE_HEARTBEAT_IDLE);
                printf("\n[GDB RSP] Detached. Returning to CLI.\nPANADIGA> ");
                fflush(stdout);
            }
        } else {
            process_cli_char(c, line_buf, &line_idx, sizeof(line_buf), &last_was_cr);
        }
    }

    return 0;
}
