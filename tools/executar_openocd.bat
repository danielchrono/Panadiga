@echo off
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
if errorlevel 1 (
    echo [ERRO] 'openocd' nao foi encontrado no PATH do sistema.
    echo Baixe o OpenOCD para Windows e adicione a pasta 'bin' a variavel PATH.
    echo.
    pause
    exit /b 1
)

echo [INFO] Iniciando OpenOCD com perfil Panasonic AM34...
echo Servidor GDB aberto na porta 3333, Telnet na porta 4444.
echo.

openocd -f "dwire32a_jtag_research/openocd_panasonic_am34.cfg"

pause
