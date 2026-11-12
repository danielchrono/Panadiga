@echo off
@chcp 65001 >nul
setlocal enabledelayedexpansion
title Panadiga - OpenOCD Matsushita AM34 Debug Engine

echo ====================================================================
echo    PANADIGA: OPENOCD MATSUSHITA AM34 ON-CHIP DEBUGGER
echo    Target: Panasonic UniPhier MN2DS0016 (Matsushita AM34 / MN103S)
echo    Config: dwire32a_jtag_research/openocd_panasonic_am34.cfg
echo ====================================================================
echo.

cd /d "%~dp0.."

where openocd >nul 2>&1
if not errorlevel 1 (
    set "OPENOCD_CMD=openocd"
    goto :RUN_OPENOCD
)

where wsl >nul 2>&1
if not errorlevel 1 (
    wsl which openocd >nul 2>&1
    if not errorlevel 1 (
        echo [INFO] OpenOCD detectado no WSL Ubuntu!
        echo [INFO] Iniciando OpenOCD com perfil Panasonic AM34 via WSL...
        echo Servidor GDB aberto na porta 3333, Telnet na porta 4444.
        echo.
        wsl bash -c "cd '$(wslpath '%CD%')' && openocd -f dwire32a_jtag_research/openocd_panasonic_am34.cfg"
        pause
        exit /b 0
    )
)

echo [ERRO] 'openocd' nao foi encontrado no PATH do Windows nem no WSL.
echo Para instalar:
echo   - No Windows: Baixe o OpenOCD (ex: via winget ou xPack) e adicione ao PATH.
echo   - No WSL/Linux: sudo apt update ^&^& sudo apt install -y openocd
echo.
pause
exit /b 1

:RUN_OPENOCD
echo [INFO] Iniciando OpenOCD com perfil Panasonic AM34...
echo Servidor GDB aberto na porta 3333, Telnet na porta 4444.
echo.

!OPENOCD_CMD! -f "dwire32a_jtag_research/openocd_panasonic_am34.cfg"

pause
