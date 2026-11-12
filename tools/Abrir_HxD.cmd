@echo off
setlocal enabledelayedexpansion

set "TARGET=%~1"
if "%TARGET%"=="" (
    if exist "%~dp0..\firmware_and_images\panasonic_dmr-eh55_flash_dump.bin" (
        set "TARGET=%~dp0..\firmware_and_images\panasonic_dmr-eh55_flash_dump.bin"
    ) else if exist "%~dp0..\firmware_and_images\nvram_corrupt_backup.bin" (
        set "TARGET=%~dp0..\firmware_and_images\nvram_corrupt_backup.bin"
    )
)

start "" "%~dp0HxD\app\HxD64.exe" !TARGET!
