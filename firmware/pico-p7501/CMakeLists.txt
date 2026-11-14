# ==============================================================================
# Panasonic DMR-EH55 DIGA Engineering Suite
# Subsystem: P7501 Async UART (115200 8N1) & Discrete Boot Probe (RP2040 / RP2350)
# Target Hardware: Raspberry Pi Pico (RP2040) / Raspberry Pi Pico 2 (RP2350)
# ==============================================================================

cmake_minimum_required(VERSION 3.13)

# Default to standard RP2040 (pico) if not specified; supports pico, pico_w, pico2, pico2_w
if(NOT DEFINED PICO_BOARD)
    set(PICO_BOARD pico CACHE STRING "Target Pico Board: 'pico', 'pico_w', 'pico2', or 'pico2_w'")
endif()

# Include official Raspberry Pi Pico SDK import helper
include(pico_sdk_import.cmake)

project(panasonic_p7501_uart C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

# Initialize the Raspberry Pi Pico SDK (auto-configures architecture for RP2040 or RP2350)
pico_sdk_init()

message(STATUS "[PANADIGA] Configuring P7501 UART Probe for Board: ${PICO_BOARD} (SOC: ${PICO_PLATFORM})")

# Define main firmware executable (modular architecture grouped by concern)
add_executable(panasonic_p7501_uart
    p7501_pico_main.c
    common/p7501_hex.c
    hal/p7501_hw.c
    hal/p7501_led_logic.c
    hal/p7501_led.c
    gdb/gdb_rsp_codec.c
    gdb/gdb_rsp_memory.c
    gdb/gdb_rsp_cpu.c
    protocol/p7501_bus_protocol.c
    flash/p7501_flash.c
    flash/p7501_ram_loader.c
    diag/p7501_diag_electrical.c
    diag/p7501_diag_bus.c
    diag/p7501_diag_busdiag.c
    diag/p7501_diag_pwrtrace.c
    diag/p7501_piologic_codec.c
    diag/p7501_diag_piologic.c
    flash/p7501_flash_cli.c
    uart/p7501_uart_bitbang.c
    uart/p7501_uart_codec.c
    uart/p7501_uart_args.c
    uart/p7501_uart_capture.c
    uart/p7501_fmu_probe.c
    uart/p7501_fmu_batch.c
    uart/p7501_uart_interactive.c
    uart/p7501_uart_pio_engine.c
    uart/p7501_uart_baud.c
    uart/p7501_uart_stream.c
    uart/p7501_uart_tx.c
    uart/p7501_pana_rsp_codec.c
    uart/p7501_pana_rsp.c
    cli/p7501_cli_help.c
    cli/p7501_cli.c
)

# Header include paths
target_include_directories(panasonic_p7501_uart PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}
)

# Assemble PIO UART program and generate C header
pico_generate_pio_header(panasonic_p7501_uart ${CMAKE_CURRENT_LIST_DIR}/p7501_uart.pio)
pico_generate_pio_header(panasonic_p7501_uart ${CMAKE_CURRENT_LIST_DIR}/p7501_uart_duplex.pio)
pico_generate_pio_header(panasonic_p7501_uart ${CMAKE_CURRENT_LIST_DIR}/p7501_logic_sampler.pio)

# Link essential Pico SDK and RP2040/RP2350 hardware drivers
target_link_libraries(panasonic_p7501_uart
    pico_stdlib
    hardware_pio
    hardware_adc
    hardware_clocks
)

# Optional Bluetooth Low Energy (CYW43439 / BTstack) support for Pico W and Pico 2 W
if(PICO_BOARD STREQUAL "pico_w" OR PICO_BOARD STREQUAL "pico2_w")
    message(STATUS "[PANADIGA] Enabling Bluetooth LE Nordic UART Service (NUS) for ${PICO_BOARD}")
    target_compile_definitions(panasonic_p7501_uart PRIVATE
        PANADIGA_BLE_ENABLED=1
    )
    target_link_libraries(panasonic_p7501_uart
        pico_cyw43_arch_none
        pico_btstack_ble
    )
endif()

# Enable USB CDC serial console, disable UART0
pico_enable_stdio_usb(panasonic_p7501_uart 1)
pico_enable_stdio_uart(panasonic_p7501_uart 0)

# Generate UF2, ELF, BIN, HEX, and MAP outputs
pico_add_extra_outputs(panasonic_p7501_uart)
