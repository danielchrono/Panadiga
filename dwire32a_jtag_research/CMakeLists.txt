# ==============================================================================
# Panasonic DMR-EH55 DIGA Engineering Suite
# Subsystem: DWire32A / OCD Interface Engine for Raspberry Pi Pico (RP2040 & RP2350)
# Target Hardware: Raspberry Pi Pico (RP2040) / Raspberry Pi Pico 2 (RP2350)
# ==============================================================================

cmake_minimum_required(VERSION 3.13)

# Default to standard RP2040 (pico) if not specified, support pico2 (RP2350)
if(NOT DEFINED PICO_BOARD)
    set(PICO_BOARD pico CACHE STRING "Target Pico Board: 'pico' (RP2040) or 'pico2' (RP2350)")
endif()

# Include official Raspberry Pi Pico SDK import helper
include(pico_sdk_import.cmake)

project(panasonic_dwire32a C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

# Initialize the Raspberry Pi Pico SDK (auto-configures architecture for RP2040 or RP2350)
pico_sdk_init()

message(STATUS "[PANADIGA] Configuring DWire32A Engine for Board: ${PICO_BOARD} (SOC: ${PICO_PLATFORM})")

# Define main firmware executable
add_executable(panasonic_dwire32a
    dwire32a_pico_main.c
    dwire32a_hw.c
    gdb_rsp.c
)

# Header include paths
target_include_directories(panasonic_dwire32a PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}
)

# Assemble PIO program and generate C header
pico_generate_pio_header(panasonic_dwire32a ${CMAKE_CURRENT_LIST_DIR}/dwire32a.pio)

# Link essential Pico SDK and RP2040/RP2350 hardware drivers
target_link_libraries(panasonic_dwire32a
    pico_stdlib
    hardware_pio
    hardware_adc
    hardware_clocks
)

# Enable USB CDC serial console, disable UART0
pico_enable_stdio_usb(panasonic_dwire32a 1)
pico_enable_stdio_uart(panasonic_dwire32a 0)

# Generate UF2, ELF, BIN, HEX, and MAP outputs
pico_add_extra_outputs(panasonic_dwire32a)
