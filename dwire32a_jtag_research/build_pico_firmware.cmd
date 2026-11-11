@echo off
@chcp 65001 >nul
setlocal enabledelayedexpansion

set "TARGET_BOARD=all"
set "NO_PAUSE="
if "%~1"=="--no-pause" set "NO_PAUSE=1"
if "%~2"=="--no-pause" set "NO_PAUSE=1"
if defined CI set "NO_PAUSE=1"
if not "%~1"=="" if "%~1" neq "--no-pause" set "TARGET_BOARD=%~1"
if not "%~2"=="" if "%~2" neq "--no-pause" set "TARGET_BOARD=%~2"

echo ====================================================================
echo    PANASONIC DWIRE32A / OCD - MULTI-PLATFORM BUILD SYSTEM
echo    Targets: RP2040 (pico) and RP2350 (pico2)
echo    Selected: !TARGET_BOARD!
echo    Usage: build_pico_firmware.cmd [pico|pico2|all]
echo ====================================================================
echo.

:: 1. Check for PICO_SDK_PATH
if not defined PICO_SDK_PATH (
    echo [INFO] PICO_SDK_PATH is not set in environment. Searching common locations...
    if exist "C:\Program Files\Raspberry Pi\Pico SDK v2.0.0\pico-sdk" (
        set "PICO_SDK_PATH=C:\Program Files\Raspberry Pi\Pico SDK v2.0.0\pico-sdk"
    ) else if exist "C:\Program Files\Raspberry Pi\Pico SDK v1.5.1\pico-sdk" (
        set "PICO_SDK_PATH=C:\Program Files\Raspberry Pi\Pico SDK v1.5.1\pico-sdk"
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
    echo [WARNING] Pico SDK not found. CMake will attempt Git fetch if online.
)

:: 2. Check for CMake
where cmake >nul 2>nul
if errorlevel 1 (
    echo [ERROR] CMake not found on PATH!
    if not defined NO_PAUSE pause
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

cd /d "%~dp0"

:: 4. Build Targets
if "!TARGET_BOARD!"=="all" (
    call :BUILD_ONE pico panasonic_dwire32a_rp2040.uf2
    call :BUILD_ONE pico2 panasonic_dwire32a_rp2350.uf2
) else if "!TARGET_BOARD!"=="pico2" (
    call :BUILD_ONE pico2 panasonic_dwire32a_rp2350.uf2
) else (
    call :BUILD_ONE pico panasonic_dwire32a_rp2040.uf2
)

echo.
echo ====================================================================
echo    BUILD EXECUTION COMPLETED!
echo ====================================================================
if not defined NO_PAUSE pause
exit /b 0

:BUILD_ONE
set "CURR_BOARD=%~1"
set "OUT_NAME=%~2"
echo.
echo >>> [1/2] Configuring project for board !CURR_BOARD!...
if not exist "build_!CURR_BOARD!" mkdir "build_!CURR_BOARD!"
cmake -B "build_!CURR_BOARD!" -G "!CMAKE_GEN!" -DCMAKE_BUILD_TYPE=Release -DPICO_BOARD=!CURR_BOARD!
if errorlevel 1 (
    echo [ERROR] CMake configuration failed for !CURR_BOARD!
    exit /b 1
)

echo.
echo >>> [2/2] Building panasonic_dwire32a target for !CURR_BOARD!...
cmake --build "build_!CURR_BOARD!" --config Release
if errorlevel 1 (
    echo [ERROR] Build compilation failed for !CURR_BOARD!
    exit /b 1
)

if exist "build_!CURR_BOARD!\panasonic_dwire32a.uf2" (
    copy /y "build_!CURR_BOARD!\panasonic_dwire32a.uf2" "%~dp0!OUT_NAME!" >nul
    echo [SUCCESS] Generated: %~dp0!OUT_NAME!
)
exit /b 0
