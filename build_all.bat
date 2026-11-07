@echo off
setlocal enabledelayedexpansion
title Panadiga Suite - Master Build Script

echo ====================================================================
echo    PANADIGA ENGINEERING SUITE: MASTER BUILD & PACKAGING SCRIPT
echo ====================================================================
echo.

cd /d "%~dp0"

echo [1/4] Executando suite de testes automatizados Rust...
cargo test --workspace
if errorlevel 1 (
    echo [ERRO] Falha nos testes de integridade da workspace.
    pause
    exit /b 1
)
echo [OK] 100%% dos testes de integridade passaram!
echo.

echo [2/4] Compilando CLI Panadiga (Release)...
cargo build --release -p panadiga
if errorlevel 1 (
    echo [ERRO] Falha ao compilar crate panadiga.
    pause
    exit /b 1
)
echo [OK] Binario gerado: target\release\panadiga.exe
echo.

echo [3/4] Compilando Frontend Desktop (React / TypeScript / Vite)...
call npm --prefix apps\desktop run build
if errorlevel 1 (
    echo [ERRO] Falha no build do frontend desktop.
    pause
    exit /b 1
)
echo [OK] Bundle gerado em apps\desktop\dist\
echo.

echo [4/4] Compilando Aplicacao Desktop Panadiga (Tauri Rust)...
cargo build --release -p panasonic-diga-desktop
if errorlevel 1 (
    echo [ERRO] Falha ao compilar desktop app em Rust.
    pause
    exit /b 1
)
echo [OK] Aplicativo desktop compilado com sucesso!
echo.

echo ====================================================================
echo    BUILD CONCLUIDO COM SUCESSO!
echo    - CLI: target\release\panadiga.exe
echo    - Desktop: target\release\panasonic-diga-desktop.exe
echo ====================================================================
pause
