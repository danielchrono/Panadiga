# ==============================================================================
# Raspberry Pi Pico SDK Import Script
# ==============================================================================
# This script locates and initializes the Raspberry Pi Pico SDK.
# It can pull the SDK from the PICO_SDK_PATH environment variable,
# from a local cache, or fetch it directly from GitHub if configured.
# ==============================================================================

if (DEFINED ENV{PICO_SDK_PATH} AND (NOT PICO_SDK_PATH))
    set(PICO_SDK_PATH $ENV{PICO_SDK_PATH})
    message(STATUS "Using PICO_SDK_PATH from environment: '${PICO_SDK_PATH}'")
endif ()

if (DEFINED ENV{PICO_SDK_FETCH_FROM_GIT} AND (NOT PICO_SDK_FETCH_FROM_GIT))
    set(PICO_SDK_FETCH_FROM_GIT $ENV{PICO_SDK_FETCH_FROM_GIT})
    message(STATUS "Using PICO_SDK_FETCH_FROM_GIT from environment: '${PICO_SDK_FETCH_FROM_GIT}'")
endif ()

if (DEFINED ENV{PICO_SDK_FETCH_FROM_GIT_PATH} AND (NOT PICO_SDK_FETCH_FROM_GIT_PATH))
    set(PICO_SDK_FETCH_FROM_GIT_PATH $ENV{PICO_SDK_FETCH_FROM_GIT_PATH})
    message(STATUS "Using PICO_SDK_FETCH_FROM_GIT_PATH from environment: '${PICO_SDK_FETCH_FROM_GIT_PATH}'")
endif ()

set(PICO_SDK_PATH "${PICO_SDK_PATH}" CACHE PATH "Path to the Raspberry Pi Pico SDK")

if (NOT PICO_SDK_PATH)
    if (PICO_SDK_FETCH_FROM_GIT)
        include(FetchContent)
        FetchContent_Declare(
                pico_sdk
                GIT_REPOSITORY https://github.com/raspberrypi/pico-sdk
                GIT_TAG master
        )
        if (NOT pico_sdk_POPULATED)
            message(STATUS "Fetching Raspberry Pi Pico SDK from GitHub...")
            FetchContent_Populate(pico_sdk)
            set(PICO_SDK_PATH ${pico_sdk_SOURCE_DIR})
        endif ()
    else ()
        message(FATAL_ERROR
                "Pico SDK location was not specified.\n"
                "Please set the PICO_SDK_PATH environment variable (e.g. export PICO_SDK_PATH=/path/to/pico-sdk)\n"
                "or set PICO_SDK_FETCH_FROM_GIT=ON to fetch automatically from GitHub."
        )
    endif ()
endif ()

get_filename_component(PICO_SDK_PATH "${PICO_SDK_PATH}" REALPATH BASE_DIR "${CMAKE_BINARY_DIR}")
if (NOT EXISTS "${PICO_SDK_PATH}")
    message(FATAL_ERROR "Directory '${PICO_SDK_PATH}' for PICO_SDK_PATH does not exist!")
endif ()

set(PICO_SDK_INIT_CMAKE_FILE ${PICO_SDK_PATH}/pico_sdk_init.cmake)
if (NOT EXISTS ${PICO_SDK_INIT_CMAKE_FILE})
    message(FATAL_ERROR "Directory '${PICO_SDK_PATH}' does not appear to contain the Raspberry Pi Pico SDK (missing pico_sdk_init.cmake)!")
endif ()

set(PICO_SDK_PATH ${PICO_SDK_PATH} CACHE PATH "Path to the Raspberry Pi Pico SDK" FORCE)

include(${PICO_SDK_INIT_CMAKE_FILE})
