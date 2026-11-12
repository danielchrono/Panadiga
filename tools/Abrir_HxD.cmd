@echo off
setlocal enabledelayedexpansion

set "TARGET=%~1"
if "%TARGET%"=="" (
    if exist "%~dp0..\firmware_and_images\IC55007.bin" (
        set "TARGET=%~dp0..\firmware_and_images\IC55007.bin"
    ) else if exist "%~dp0..\firmware_and_images\dmr-ez485v\panasonic_dmr_ez485v_flash_dump_4mb.bin" (
        set "TARGET=%~dp0..\firmware_and_images\dmr-ez485v\panasonic_dmr_ez485v_flash_dump_4mb.bin"
    ) else if exist "%~dp0..\firmware_and_images\panasonic DMR EH-55 Firmware hdd_100.bin" (
        set "TARGET=%~dp0..\firmware_and_images\panasonic DMR EH-55 Firmware hdd_100.bin"
    ) else if exist "%~dp0..\firmware_and_images\PANA_DVD.FRM" (
        set "TARGET=%~dp0..\firmware_and_images\PANA_DVD.FRM"
    )
)

start "" "%~dp0HxD\app\HxD64.exe" !TARGET!
