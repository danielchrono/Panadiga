@echo off
setlocal enabledelayedexpansion
title Panadiga Suite - Master Build & Distribution Script

echo ====================================================================
echo    PANADIGA ENGINEERING SUITE: MASTER UNIFIED BUILD PIPELINE
echo ====================================================================
echo.

cd /d "%~dp0"

echo [1/5] Executando suite de testes de integridade Rust...
cargo test --workspace
if errorlevel 1 (
    echo [ERRO] Falha nos testes automatizados da workspace.
    pause
    exit /b 1
)
echo [OK] 100%% dos testes de integridade passaram!
echo.

echo [2/5] Compilando CLI Panadiga em modo Release...
cargo build --release -p panadiga
if errorlevel 1 (
    echo [ERRO] Falha ao compilar crate panadiga.
    pause
    exit /b 1
)
echo [OK] Binario gerado: target\release\panadiga.exe
echo.

echo [3/5] Compilando Frontend Desktop (React / TypeScript / Vite)...
call npm --prefix apps\desktop run build
if errorlevel 1 (
    echo [ERRO] Falha no build do frontend desktop.
    pause
    exit /b 1
)
echo [OK] Bundle otimizado gerado em apps\desktop\dist\
echo.

echo [4/5] Compilando Aplicacao Desktop Panadiga (Tauri Rust com bundle integrado)...
cargo build --release -p panasonic-diga-desktop
if errorlevel 1 (
    echo [ERRO] Falha ao compilar desktop app em Rust.
    pause
    exit /b 1
)
echo [OK] Executavel desktop compilado com sucesso!
echo.

echo [5/5] Empacotando distribuicao para dist_bin\...
if not exist "dist_bin" mkdir "dist_bin"
copy /y "target\release\panadiga.exe" "dist_bin\panadiga-cli.exe" >nul
copy /y "target\release\panasonic-diga-desktop.exe" "dist_bin\panadiga-desktop.exe" >nul

if not exist "dist_bin\tools" mkdir "dist_bin\tools"
xcopy /e /i /y "tools" "dist_bin\tools" >nul

(
echo @echo off
echo start "" "%%~dp0panadiga-desktop.exe"
) > "dist_bin\Executar_Panadiga.bat"

echo [OK] Pacote consolidado gerado em dist_bin\
echo.

echo ====================================================================
echo    CONCLUIDO COM SUCESSO!
echo    - Panadiga Desktop: dist_bin\panadiga-desktop.exe
echo    - Panadiga CLI:     dist_bin\panadiga-cli.exe
echo    - Ferramentas:      dist_bin\tools\ (GDB, OpenOCD, Ghidra, HxD)
echo    - Atalho:           dist_bin\Executar_Panadiga.bat
echo ====================================================================
if "%1"=="" pause
