@echo off
@chcp 65001 >nul
setlocal enabledelayedexpansion
title Panadiga - GDB Remote RSP Bridge Server

echo ====================================================================
echo    PANADIGA: SERVIDOR PONTE GDB REMOTE SERIAL PROTOCOL (RSP)
echo    Target: Panasonic UniPhier MN2DS0016 (Matsushita AM34 / MN103S)
echo ====================================================================
echo.

cd /d "%~dp0.."

set "PANADIGA_EXE=target\release\panadiga.exe"
if not exist "!PANADIGA_EXE!" (
    set "PANADIGA_EXE=target\debug\panadiga.exe"
)

if not exist "!PANADIGA_EXE!" (
    echo [INFO] Binario panadiga.exe nao encontrado compilado. Compilando com cargo...
    cargo build --bin panadiga
    if errorlevel 1 (
        echo [ERRO] Falha ao compilar panadiga com cargo.
        pause
        exit /b 1
    )
    set "PANADIGA_EXE=target\debug\panadiga.exe"
)

echo [1/2] Varrendo portas seriais conectadas...
"!PANADIGA_EXE!" pico list
echo.

set "PORT=%~1"
if "!PORT!"=="" (
    set /p "PORT=Digite a porta COM do seu Raspberry Pi Pico (ex: COM3): "
)

if "!PORT!"=="" (
    echo [ERRO] Porta COM invalida.
    pause
    exit /b 1
)

echo.
echo [2/2] Iniciando ponte GDB na porta !PORT! e escutando em 127.0.0.1:3333...
echo Pressione Ctrl+C para encerrar o servidor a qualquer momento.
echo.

"!PANADIGA_EXE!" pico gdb-bridge --port !PORT! --listen 127.0.0.1:3333

pause
