@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul
title Panadiga Suite - Unified Native Build Pipeline

echo ====================================================================
echo    PANADIGA ENGINEERING SUITE: MASTER UNIFIED BUILD PIPELINE
echo    Configuracao: Paralelismo Total (%NUMBER_OF_PROCESSORS% Cores) + UTF-8
echo    Modo: Desktop GUI + Core CLI + Firmware Pico RP2040/RP2350
echo ====================================================================
echo.

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

set CARGO_BUILD_JOBS=%NUMBER_OF_PROCESSORS%
set CARGO_INCREMENTAL=1

:: [1/5] Desktop Frontend (React / TypeScript / Vite)
echo [1/5] Compilando Frontend Desktop (React / TypeScript / Vite)...
if not exist "apps\desktop\node_modules" (
    echo [*] Instalando dependencias do frontend...
    call npm --prefix apps\desktop ci
)
call npm --prefix apps\desktop run build
if errorlevel 1 (
    echo [ERRO] Falha no build do frontend desktop.
    if "%1"=="" pause
    exit /b 1
)
echo [OK] Bundle otimizado gerado em apps\desktop\dist\
echo.

:: [2/5] Rust Workspace Tests (Paralelismo Total)
echo [2/5] Executando suite de testes do Workspace Rust (%NUMBER_OF_PROCESSORS% threads)...
cargo test --workspace
if errorlevel 1 (
    echo [ERRO] Falha nos testes automatizados do workspace.
    if "%1"=="" pause
    exit /b 1
)
echo [OK] 100%% dos testes do workspace passaram!
echo.

:: [3/5] Rust Workspace Release Compilation (Paralelismo Total: CLI + Desktop GUI)
echo [3/5] Compilando Workspace Rust em modo Release (%NUMBER_OF_PROCESSORS% threads)...
cargo build --release --workspace
if errorlevel 1 (
    echo [ERRO] Falha ao compilar workspace em modo release.
    if "%1"=="" pause
    exit /b 1
)
echo [OK] Binarios gerados com sucesso:
echo      - target\release\panadiga.exe
echo      - target\release\panasonic-diga-desktop.exe
echo.

:: [4/5] Raspberry Pi Pico Firmware (RP2040 & RP2350)
echo [4/5] Verificando compilador de firmware do Raspberry Pi Pico...
where arm-none-eabi-gcc >nul 2>nul
if not errorlevel 1 (
    echo [*] arm-none-eabi-gcc detectado! Compilando firmware C11 [RP2040 + RP2350 via Ninja]...
    call dwire32a_jtag_research\build_pico_firmware.bat all
) else (
    echo [INFO] arm-none-eabi-gcc nao encontrado no PATH local.
    echo        Os binarios oficiais .UF2 [RP2040 e RP2350] sao gerados automaticamente
    echo        pelo pipeline de CI/CD do GitHub Actions em orgchrono/Panadiga.
)
echo.

:: [5/5] Packaging Distribution in dist\ and dist_bin\
echo [5/5] Empacotando distribuicao consolidada para dist\...
if not exist "dist" mkdir "dist"
if not exist "dist\tools" mkdir "dist\tools"
if not exist "dist\firmware" mkdir "dist\firmware"
if not exist "dist\releases" mkdir "dist\releases"

copy /y "target\release\panadiga.exe" "dist\panadiga-cli.exe" >nul
copy /y "target\release\panasonic-diga-desktop.exe" "dist\panadiga-desktop.exe" >nul

:: Ensure WebView2Loader.dll is packaged next to the desktop binary
if exist "target\release\WebView2Loader.dll" (
    copy /y "target\release\WebView2Loader.dll" "dist\WebView2Loader.dll" >nul
) else if exist "target\debug\WebView2Loader.dll" (
    copy /y "target\debug\WebView2Loader.dll" "dist\WebView2Loader.dll" >nul
)

:: Export Ghidra scripts directly from native Rust SSOT
cargo run -- ghidra export-all --dir "dist\tools" >nul 2>nul
xcopy /e /i /y "tools" "dist\tools" >nul 2>nul

:: Copy compiled firmware binaries if present
if exist "dwire32a_jtag_research\panasonic_dwire32a_rp2040.uf2" (
    copy /y "dwire32a_jtag_research\panasonic_dwire32a_rp2040.uf2" "dist\firmware\panasonic_dwire32a_rp2040.uf2" >nul
)
if exist "dwire32a_jtag_research\panasonic_dwire32a_rp2350.uf2" (
    copy /y "dwire32a_jtag_research\panasonic_dwire32a_rp2350.uf2" "dist\firmware\panasonic_dwire32a_rp2350.uf2" >nul
)

:: Launcher script inside dist\
(
echo @echo off
echo @chcp 65001 ^>nul
echo start "" "%%~dp0panadiga-desktop.exe"
) > "dist\Executar_Panadiga.bat"

:: Synchronize dist_bin\ for backwards compatibility
if not exist "dist_bin" mkdir "dist_bin"
xcopy /e /i /y "dist" "dist_bin" >nul 2>nul

:: Generate release archive .zip with SHA256 checksum
powershell -NoProfile -Command "$OutputEncoding = [System.Text.Encoding]::UTF8; [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $zip = 'dist\releases\panadiga-windows-x86_64.zip'; if (Test-Path $zip) { Remove-Item $zip -Force }; Compress-Archive -Path 'dist\panadiga-cli.exe', 'dist\panadiga-desktop.exe', 'dist\WebView2Loader.dll', 'dist\Executar_Panadiga.bat', 'dist\tools', 'dist\firmware', 'README.md', 'LICENSE' -DestinationPath $zip -CompressionLevel Optimal; $hash = (Get-FileHash -Algorithm SHA256 $zip).Hash; [IO.File]::WriteAllText('dist\releases\SHA256SUMS.txt', \"$hash  panadiga-windows-x86_64.zip`r`n\", [Text.Encoding]::UTF8)"

echo.
echo ====================================================================
echo    [OK] BUILD CONCLUIDO COM SUCESSO!
echo    - Panadiga Desktop: dist\panadiga-desktop.exe
echo    - Panadiga CLI:     dist\panadiga-cli.exe
echo    - Runtime DLL:      dist\WebView2Loader.dll
echo    - Ferramentas:      dist\tools\ (GDB, Ghidra scripts, HxD)
echo    - Firmware Pico:    dist\firmware\ (.UF2 RP2040 e RP2350)
echo    - Pacote Release:   dist\releases\panadiga-windows-x86_64.zip
echo    - Atalho Rapido:    dist\Executar_Panadiga.bat
echo ====================================================================
if "%1"=="" pause
