@echo off
@chcp 65001 >nul
setlocal enabledelayedexpansion
title Panadiga Suite - Fast Launcher

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

set DESKTOP_EXE=%SCRIPT_DIR%dist_bin\panadiga-desktop.exe
set CLI_EXE=%SCRIPT_DIR%dist_bin\panadiga-cli.exe

:: Argument dispatching
if "%1"=="--help" goto :help
if "%1"=="-h" goto :help
if "%1"=="--dev" goto :run_dev
if "%1"=="--build" goto :build_and_run
if "%1"=="--cli" goto :run_cli
if "%1"=="nvram" goto :run_cli_direct
if "%1"=="hdd" goto :run_cli_direct
if "%1"=="frm" goto :run_cli_direct
if "%1"=="headers" goto :run_cli_direct
if "%1"=="scan" goto :run_cli_direct
if "%1"=="manuals" goto :run_cli_direct
if "%1"=="pico" goto :run_cli_direct
if "%1"=="ghidra" goto :run_cli_direct

:: Default: Run Desktop GUI immediately
if exist "%DESKTOP_EXE%" (
    echo [INFO] Iniciando Panadiga Desktop diretamente do binario consolidado...
    start "" "%DESKTOP_EXE%"
    exit /b 0
)

echo [AVISO] Executavel desktop nao encontrado em dist_bin\.
echo [INFO] Executando build acelerado (paralelismo total)...
call build_all.bat
if exist "%DESKTOP_EXE%" (
    start "" "%DESKTOP_EXE%"
    exit /b 0
) else (
    echo [ERRO] Falha ao localizar o binario compilado apos o build.
    pause
    exit /b 1
)

:run_dev
echo [INFO] Iniciando ambiente de desenvolvimento Desktop (Vite + Tauri Dev)...
call npm --prefix apps\desktop run tauri dev
exit /b 0

:build_and_run
echo [INFO] Forcando rebuild completo do ecossistema...
call build_all.bat
if exist "%DESKTOP_EXE%" (
    start "" "%DESKTOP_EXE%"
)
exit /b 0

:run_cli
shift
if exist "%CLI_EXE%" (
    "%CLI_EXE%" %*
) else (
    cargo run -- %*
)
exit /b %errorlevel%

:run_cli_direct
if exist "%CLI_EXE%" (
    "%CLI_EXE%" %*
) else (
    cargo run -- %*
)
exit /b %errorlevel%

:help
echo ====================================================================
echo Panadiga Suite - Inicializador Rapido
echo ====================================================================
echo Uso:
echo   run.bat                 Inicia a interface Desktop nativa instantaneamente
echo   run.bat --dev           Inicia em modo de desenvolvimento (Hot-Reload)
echo   run.bat --build         Forca compilacao completa e inicia o app
echo   run.bat --cli [args]    Executa a CLI nativa com argumentos
echo   run.bat nvram clean ... Atalho direto para comandos da CLI Panadiga
echo   run.bat hdd inspect ... Atalho direto para inspecao de disco
echo   run.bat ghidra ...      Atalho direto para gerador de scripts Ghidra
echo ====================================================================
exit /b 0
