/**
 * @file dwire32a_pico_main.c
 * @brief Slim RP2040 / RP2350 entrypoint for the Panadiga DWire32A Debug Probe.
 *
 * Initializes hardware subsystems (`dwire32a_hw`), status LED (`dwire32a_led`),
 * and GDB Remote Serial Protocol bridge (`gdb_rsp`), then runs the non-blocking
 * USB CDC loop dispatching either raw GDB RSP frames or CLI commands via
 * `cli/dwire32a_cli.h`.
 *
 * @standard ISO/IEC 9899:2011 (C11)
 */

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include "pico/stdlib.h"

#include "cli/dwire32a_cli.h"
#include "hal/dwire32a_hw.h"
#include "hal/dwire32a_led.h"
#include "dwire32a_types.h"
#include "gdb/gdb_rsp.h"

#define DWIRE_CLI_LINE_MAX 256U

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

        dwire_cli_process_command(line_buf);
        if (!gdb_rsp_is_active()) {
            printf("DWIRE32A> ");
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
    dwire_led_init();
    (void)dwire_hw_init();
    gdb_rsp_init();

    sleep_ms(250U);
    dwire_cli_print_banner();
    printf("DWIRE32A> ");
    fflush(stdout);

    char line_buf[DWIRE_CLI_LINE_MAX];
    size_t line_idx = 0U;
    bool last_was_cr = false;

    for (;;) {
        dwire_led_tick();

        const int ch = getchar_timeout_us(0);
        if (ch == PICO_ERROR_TIMEOUT) {
            tight_loop_contents();
            continue;
        }

        const char c = (char)ch;
        if (gdb_rsp_is_active()) {
            gdb_rsp_process_char(c);
            if (!gdb_rsp_is_active()) {
                dwire_led_set_mode(LED_MODE_HEARTBEAT_IDLE);
                printf("\n[GDB RSP] Detached. Returning to CLI.\nDWIRE32A> ");
                fflush(stdout);
            }
        } else {
            process_cli_char(c, line_buf, &line_idx, sizeof(line_buf), &last_was_cr);
        }
    }

    return 0;
}
