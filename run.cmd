@echo off
@chcp 65001 >nul
setlocal enabledelayedexpansion
title Panadiga Suite - Fast Launcher

set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

set "DIST_EXE=%SCRIPT_DIR%dist\panadiga-desktop.exe"
set "DIST_BIN_EXE=%SCRIPT_DIR%dist_bin\panadiga-desktop.exe"
set "CLI_EXE=%SCRIPT_DIR%dist\panadiga-cli.exe"

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
if "%1"=="rag" goto :run_cli_direct

:: Default: Run Desktop GUI immediately from dist\ or dist_bin\
if exist "%DIST_EXE%" (
    if not exist "%SCRIPT_DIR%dist\WebView2Loader.dll" (
        if exist "%SCRIPT_DIR%target\release\WebView2Loader.dll" copy /y "%SCRIPT_DIR%target\release\WebView2Loader.dll" "%SCRIPT_DIR%dist\WebView2Loader.dll" >nul
        if exist "%SCRIPT_DIR%target\debug\WebView2Loader.dll" copy /y "%SCRIPT_DIR%target\debug\WebView2Loader.dll" "%SCRIPT_DIR%dist\WebView2Loader.dll" >nul
    )
    echo [INFO] Iniciando Panadiga Desktop diretamente de dist\...
    start "" "%DIST_EXE%"
    exit /b 0
)

if exist "%DIST_BIN_EXE%" (
    if not exist "%SCRIPT_DIR%dist_bin\WebView2Loader.dll" (
        if exist "%SCRIPT_DIR%target\release\WebView2Loader.dll" copy /y "%SCRIPT_DIR%target\release\WebView2Loader.dll" "%SCRIPT_DIR%dist_bin\WebView2Loader.dll" >nul
        if exist "%SCRIPT_DIR%target\debug\WebView2Loader.dll" copy /y "%SCRIPT_DIR%target\debug\WebView2Loader.dll" "%SCRIPT_DIR%dist_bin\WebView2Loader.dll" >nul
    )
    echo [INFO] Iniciando Panadiga Desktop diretamente de dist_bin\...
    start "" "%DIST_BIN_EXE%"
    exit /b 0
)

:: If executable not found, compile and run
echo [AVISO] Executavel desktop nao encontrado em dist\.
echo [INFO] Executando build unificado acelerado...
call "%SCRIPT_DIR%build.cmd"
if exist "%DIST_EXE%" (
    start "" "%DIST_EXE%"
    exit /b 0
) else if exist "%DIST_BIN_EXE%" (
    start "" "%DIST_BIN_EXE%"
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
call "%SCRIPT_DIR%build.cmd"
if exist "%DIST_EXE%" (
    start "" "%DIST_EXE%"
) else if exist "%DIST_BIN_EXE%" (
    start "" "%DIST_BIN_EXE%"
)
exit /b 0

:run_cli
shift
if exist "%CLI_EXE%" (
    "%CLI_EXE%" %*
) else (
    cargo run -p panadiga -- %*
)
exit /b %errorlevel%

:run_cli_direct
if exist "%CLI_EXE%" (
    "%CLI_EXE%" %*
) else (
    cargo run -p panadiga -- %*
)
exit /b %errorlevel%

:help
echo ====================================================================
echo Panadiga Suite - Inicializador Rapido
echo ====================================================================
echo Uso:
echo   run.cmd                 Inicia a interface Desktop nativa instantaneamente
echo   run.cmd --dev           Inicia em modo de desenvolvimento (Hot-Reload)
echo   run.cmd --build         Forca compilacao completa e inicia o app
echo   run.cmd --cli [args]    Executa a CLI nativa com argumentos
echo   run.cmd nvram clean ... Atalho direto para comandos da CLI Panadiga
echo   run.cmd hdd inspect ... Atalho direto para inspecao de disco
echo   run.cmd ghidra ...      Atalho direto para gerador de scripts Ghidra
echo   run.cmd rag query ...   Atalho direto para consulta ao motor RAG Zero-Error
echo ====================================================================
exit /b 0
