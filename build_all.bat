@echo off
@chcp 65001 >nul
setlocal enabledelayedexpansion
title Panadiga Suite - Master Build & Distribution Script

echo ====================================================================
echo    PANADIGA ENGINEERING SUITE: MASTER UNIFIED BUILD PIPELINE
echo    Configuração: Paralelismo Total (%NUMBER_OF_PROCESSORS% Cores) + UTF-8
echo ====================================================================
echo.

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

set CARGO_BUILD_JOBS=%NUMBER_OF_PROCESSORS%
set CARGO_INCREMENTAL=1

echo [1/6] Executando suite de testes de integridade Rust (16 testes)...
cargo test -p panadiga
if errorlevel 1 (
    echo [ERRO] Falha nos testes automatizados do crate panadiga.
    if "%1"=="" pause
    exit /b 1
)
echo [OK] 100%% dos testes de integridade passaram!
echo.

echo [2/6] Compilando CLI Panadiga em modo Release...
cargo build --release -p panadiga
if errorlevel 1 (
    echo [ERRO] Falha ao compilar crate panadiga.
    if "%1"=="" pause
    exit /b 1
)
echo [OK] Binario gerado: target\release\panadiga.exe
echo.

echo [3/6] Compilando Frontend Desktop (React / TypeScript / Vite)...
call npm --prefix apps\desktop run build
if errorlevel 1 (
    echo [ERRO] Falha no build do frontend desktop.
    if "%1"=="" pause
    exit /b 1
)
echo [OK] Bundle otimizado gerado em apps\desktop\dist\
echo.

echo [4/6] Compilando Aplicacao Desktop Panadiga (Tauri Rust com bundle integrado)...
cargo build --release -p panasonic-diga-desktop
if errorlevel 1 (
    echo [ERRO] Falha ao compilar desktop app em Rust.
    if "%1"=="" pause
    exit /b 1
)
echo [OK] Executavel desktop compilado com sucesso!
echo.

echo [5/6] Empacotando distribuicao para dist_bin\...
if not exist "dist_bin" mkdir "dist_bin"
copy /y "target\release\panadiga.exe" "dist_bin\panadiga-cli.exe" >nul
copy /y "target\release\panasonic-diga-desktop.exe" "dist_bin\panadiga-desktop.exe" >nul

if not exist "dist_bin\tools" mkdir "dist_bin\tools"
xcopy /e /i /y "tools" "dist_bin\tools" >nul

(
echo @echo off
echo @chcp 65001 ^>nul
echo start "" "%%~dp0panadiga-desktop.exe"
) > "dist_bin\Executar_Panadiga.bat"

echo [OK] Pacote consolidado gerado em dist_bin\
echo.

echo [6/6] Gerando Arquivo de Release (.zip) e Checksum SHA256...
if not exist "dist_bin\releases" mkdir "dist_bin\releases"
powershell -NoProfile -Command "$OutputEncoding = [System.Text.Encoding]::UTF8; [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $zip = 'dist_bin\releases\panadiga-windows-x86_64.zip'; if (Test-Path $zip) { Remove-Item $zip -Force }; Compress-Archive -Path 'dist_bin\panadiga-cli.exe', 'dist_bin\panadiga-desktop.exe', 'dist_bin\Executar_Panadiga.bat', 'dist_bin\tools', 'README.md', 'LICENSE' -DestinationPath $zip -CompressionLevel Optimal; $hash = (Get-FileHash -Algorithm SHA256 $zip).Hash; [IO.File]::WriteAllText('dist_bin\releases\SHA256SUMS.txt', \"$hash  panadiga-windows-x86_64.zip`r`n\", [Text.Encoding]::UTF8)"
echo [OK] Pacote de Release gerado em dist_bin\releases\panadiga-windows-x86_64.zip
echo.

echo ====================================================================
echo    CONCLUIDO COM SUCESSO!
echo    - Panadiga Desktop: dist_bin\panadiga-desktop.exe
echo    - Panadiga CLI:     dist_bin\panadiga-cli.exe
echo    - Ferramentas:      dist_bin\tools\ (GDB, OpenOCD, Ghidra, HxD)
echo    - Release Zip:      dist_bin\releases\panadiga-windows-x86_64.zip
echo    - Checksums:        dist_bin\releases\SHA256SUMS.txt
echo    - Atalho:           dist_bin\Executar_Panadiga.bat
echo ====================================================================
if "%1"=="" pause
