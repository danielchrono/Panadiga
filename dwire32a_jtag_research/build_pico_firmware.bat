@echo off
setlocal enabledelayedexpansion

set "TARGET_BOARD=pico"
if not "%~1"=="" set "TARGET_BOARD=%~1"

echo ====================================================================
echo    PANASONIC DWIRE32A / OCD - RASPBERRY PI PICO BUILD SYSTEM
echo    Firmware Target: panasonic_dwire32a.uf2 (Board: !TARGET_BOARD!)
echo    Usage: build_pico_firmware.bat [pico|pico2]
echo ====================================================================
echo.

:: 1. Check for PICO_SDK_PATH
if not defined PICO_SDK_PATH (
    echo [INFO] PICO_SDK_PATH is not set in environment. Searching common locations...
    if exist "C:\Program Files\Raspberry Pi\Pico SDK v1.5.1\pico-sdk" (
        set "PICO_SDK_PATH=C:\Program Files\Raspberry Pi\Pico SDK v1.5.1\pico-sdk"
    ) else if exist "C:\Program Files\Raspberry Pi\Pico SDK v1.5.0\pico-sdk" (
        set "PICO_SDK_PATH=C:\Program Files\Raspberry Pi\Pico SDK v1.5.0\pico-sdk"
    ) else if exist "C:\pico\pico-sdk" (
        set "PICO_SDK_PATH=C:\pico\pico-sdk"
    ) else if exist "C:\pico-sdk" (
        set "PICO_SDK_PATH=C:\pico-sdk"
    ) else if exist "%USERPROFILE%\pico-sdk" (
        set "PICO_SDK_PATH=%USERPROFILE%\pico-sdk"
    ) else if exist "%USERPROFILE%\.pico-sdk\sdk\2.0.0" (
        set "PICO_SDK_PATH=%USERPROFILE%\.pico-sdk\sdk\2.0.0"
    ) else if exist "%USERPROFILE%\.pico-sdk\sdk\1.5.1" (
        set "PICO_SDK_PATH=%USERPROFILE%\.pico-sdk\sdk\1.5.1"
    ) else if exist "%~dp0..\pico-sdk" (
        set "PICO_SDK_PATH=%~dp0..\pico-sdk"
    )
)

if defined PICO_SDK_PATH (
    echo [OK] Using Pico SDK at: "!PICO_SDK_PATH!"
) else (
    echo [WARNING] Raspberry Pi Pico SDK not automatically located!
    echo [INFO] CMake will attempt to fetch the SDK from Git if online,
    echo        or you may set PICO_SDK_PATH manually:
    echo        set PICO_SDK_PATH=C:\path\to\pico-sdk
    echo.
)

:: 2. Check for CMake
where cmake >nul 2>nul
if errorlevel 1 (
    echo [ERROR] CMake not found on PATH!
    echo         Please install CMake or add it to PATH.
    pause
    exit /b 1
)

:: 3. Detect Build Generator
set "CMAKE_GEN=Ninja"
where ninja >nul 2>nul
if errorlevel 1 (
    where mingw32-make >nul 2>nul
    if not errorlevel 1 (
        set "CMAKE_GEN=MinGW Makefiles"
    ) else (
        where nmake >nul 2>nul
        if not errorlevel 1 (
            set "CMAKE_GEN=NMake Makefiles"
        )
    )
)
echo [OK] Selected CMake Generator: !CMAKE_GEN!

:: 4. Create Build Directory
cd /d "%~dp0"
if not exist "build" mkdir "build"

echo.
echo [1/2] Configuring project with CMake for board !TARGET_BOARD!...
cmake -B build -G "!CMAKE_GEN!" -DCMAKE_BUILD_TYPE=Release -DPICO_BOARD=!TARGET_BOARD!
if errorlevel 1 (
    echo [ERROR] CMake configuration failed!
    echo         Ensure arm-none-eabi-gcc toolchain is installed and on PATH.
    pause
    exit /b 1
)

echo.
echo [2/2] Building panasonic_dwire32a target...
cmake --build build --config Release
if errorlevel 1 (
    echo [ERROR] Build compilation failed!
    pause
    exit /b 1
)

echo.
echo ====================================================================
if exist "build\panasonic_dwire32a.uf2" (
    echo [SUCCESS] FIRMWARE COMPILED SUCCESSFULLY!
    echo.
    echo Target binary location:
    echo   %~dp0build\panasonic_dwire32a.uf2
    echo.
    echo FLASHING INSTRUCTIONS (DRAG ^& DROP):
    echo   1. Unplug the Raspberry Pi Pico from USB.
    echo   2. Press and hold down the white BOOTSEL button on the Pico.
    echo   3. While holding BOOTSEL, plug the Pico back into the PC USB port.
    echo   4. Release the BOOTSEL button.
    echo   5. A removable drive named 'RPI-RP2' will mount in Windows Explorer.
    echo   6. Drag and drop 'panasonic_dwire32a.uf2' onto the 'RPI-RP2' drive.
    echo   7. The Pico will reboot automatically into DWire32A USB CDC mode.
    echo   8. Open PuTTY, TeraTerm, or the Tauri app on the newly assigned COM port!
) else (
    echo [WARNING] Build finished but 'panasonic_dwire32a.uf2' was not generated.
    echo           Check the build directory for .elf or compiler warnings.
)
echo ====================================================================
echo.
pause
