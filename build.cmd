@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul
if exist "%USERPROFILE%\.cargo\bin" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
title Panadiga Suite - Native Build Pipeline

echo ====================================================================
echo    PANADIGA ENGINEERING SUITE: MASTER BUILD PIPELINE (Windows)
echo    Configuracao: Paralelismo (%NUMBER_OF_PROCESSORS% Cores) + UTF-8
echo    Modo: Desktop GUI + Core CLI Engine (Binario Unificado)
echo ====================================================================
echo.

set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

:: Sanitize CI environment variable for clap/tauri-cli compatibility (accepts only true/false)
if defined CI set "CI=true"

set CARGO_BUILD_JOBS=%NUMBER_OF_PROCESSORS%
set CARGO_INCREMENTAL=1

:: Dispatch Android build if requested
if "%1"=="--android" goto :build_android
if "%1"=="-a" goto :build_android
if "%1"=="--apk" goto :build_android
if "%1"=="--all" set "BUILD_ALL=1"

:: [1/4] Build React Interface (UI)
echo [1/4] Compilando Frontend Desktop (React / TypeScript / Vite)...
if not exist "apps\desktop\node_modules" (
    echo [*] Instalando dependencias do frontend...
    call npm --prefix apps\desktop ci
    if errorlevel 1 (
        echo [ERRO] Falha ao instalar dependencias do frontend.
        goto :error_exit
    )
)
call npm --prefix apps\desktop run build
if errorlevel 1 (
    echo [ERRO] Falha no build de producao do frontend Vite.
    goto :error_exit
)
echo [OK] Bundle otimizado gerado em apps\desktop\dist\
echo.

:: [2/4] Verifying and Exporting Lab Tools (Ghidra SSOT)
echo [2/4] Exportando scripts Ghidra diretamente do Rust SSOT...
cargo run --release -p panadiga -- ghidra export-all --dir "tools" >nul 2>nul
echo [OK] Scripts Ghidra sincronizados em tools\
echo.

:: [3/4] Compiling Native Rust Unified Release Binary
echo [3/4] Compilando Binario Unificado Panadiga em modo Release (%NUMBER_OF_PROCESSORS% threads)...
cargo build --release -p panadiga
if errorlevel 1 (
    echo [ERRO] Falha ao compilar binario unificado em modo release.
    goto :error_exit
)
echo [OK] Binario unificado gerado com sucesso: target\release\panadiga.exe
echo.

:: Optional: Compile Pico Firmware only if both arm-none-eabi-gcc and cmake are available
where arm-none-eabi-gcc >nul 2>nul
if not errorlevel 1 (
    if exist "dwire32a_jtag_research\cmake.exe" (
        echo [*] CMake found locally. Using local CMake for firmware build.
        set "CMAKE_EXE=dwire32a_jtag_research\cmake.exe"
        call dwire32a_jtag_research\build_pico_firmware.cmd all --no-pause >nul 2>nul
    ) else (
        where cmake >nul 2>&1
        if not errorlevel 1 (
            echo [*] Toolchain ARM GCC e CMake detectados! Compilando firmware C11 [RP2040 + RP2350]...
            call dwire32a_jtag_research\build_pico_firmware.cmd all >nul 2>nul
        ) else (
            rem echo [*] arm-none-eabi-gcc detectado, mas CMake nao está no PATH. Usando binarios UF2 embutidos no Rust.
        )
    )
)

:: [4/4] Packaging Distribution into dist\
echo [4/4] Empacotando distribuicao consolidada para dist\...
if not exist "dist" mkdir "dist"
if not exist "dist\tools" mkdir "dist\tools"
if not exist "dist\firmware" mkdir "dist\firmware"
if not exist "dist\releases" mkdir "dist\releases"

copy /y "target\release\panadiga.exe" "dist\panadiga.exe" >nul

:: Ensure WebView2Loader.dll is packaged next to the desktop binary
if exist "target\release\WebView2Loader.dll" (
    copy /y "target\release\WebView2Loader.dll" "dist\WebView2Loader.dll" >nul
) else if exist "target\debug\WebView2Loader.dll" (
    copy /y "target\debug\WebView2Loader.dll" "dist\WebView2Loader.dll" >nul
) else (
    powershell -NoProfile -Command "Get-ChildItem -Path target -Recurse -Filter WebView2Loader.dll -ErrorAction SilentlyContinue | Where-Object { $_.DirectoryName -match 'x64' } | Select-Object -First 1 | ForEach-Object { Copy-Item $_.FullName 'dist\WebView2Loader.dll' -Force }" >nul 2>nul
)

:: Copy tools and Ghidra scripts
xcopy /e /i /y "tools" "dist\tools" >nul 2>nul

:: Copy compiled Pico firmware binaries if present
if exist "src\pico\firmware_bins\panasonic_dwire32a_rp2040.uf2" (
    copy /y "src\pico\firmware_bins\panasonic_dwire32a_rp2040.uf2" "dist\firmware\panasonic_dwire32a_rp2040.uf2" >nul
) else if exist "dwire32a_jtag_research\panasonic_dwire32a_rp2040.uf2" (
    copy /y "dwire32a_jtag_research\panasonic_dwire32a_rp2040.uf2" "dist\firmware\panasonic_dwire32a_rp2040.uf2" >nul
)
if exist "src\pico\firmware_bins\panasonic_dwire32a_rp2350.uf2" (
    copy /y "src\pico\firmware_bins\panasonic_dwire32a_rp2350.uf2" "dist\firmware\panasonic_dwire32a_rp2350.uf2" >nul
) else if exist "dwire32a_jtag_research\panasonic_dwire32a_rp2350.uf2" (
    copy /y "dwire32a_jtag_research\panasonic_dwire32a_rp2350.uf2" "dist\firmware\panasonic_dwire32a_rp2350.uf2" >nul
)

:: Clean legacy or redundant script runners from dist\
if exist "dist\run.cmd" del /f /q "dist\run.cmd" >nul 2>nul
if exist "dist\run.sh" del /f /q "dist\run.sh" >nul 2>nul
if exist "dist\run.ps1" del /f /q "dist\run.ps1" >nul 2>nul
if exist "dist\Executar_Panadiga.cmd" del /f /q "dist\Executar_Panadiga.cmd" >nul 2>nul
if exist "dist\Executar_Panadiga.bat" del /f /q "dist\Executar_Panadiga.bat" >nul 2>nul

:: Normalize timestamps older than 1980 (e.g. WebView2Loader.dll) to satisfy ZIP FAT standard
powershell -NoProfile -Command "Get-ChildItem -Path dist -Recurse | Where-Object { $_.LastWriteTime.Year -lt 1980 } | ForEach-Object { $_.LastWriteTime = (Get-Date) }" >nul 2>nul

:: Generate release archive .zip with SHA256 checksum
powershell -NoProfile -Command "$OutputEncoding = [System.Text.Encoding]::UTF8; [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $zip = 'dist\releases\panadiga-windows-x86_64.zip'; if (Test-Path $zip) { Remove-Item $zip -Force }; $items = @('dist\panadiga.exe', 'dist\WebView2Loader.dll', 'dist\tools', 'dist\firmware', 'README.md', 'README.pt-BR.md', 'LICENSE') | Where-Object { Test-Path $_ }; Compress-Archive -Path $items -DestinationPath $zip -CompressionLevel Optimal; if (Test-Path $zip) { $hash = (Get-FileHash -Algorithm SHA256 $zip).Hash; Set-Content -Path 'dist\releases\SHA256SUMS.txt' -Value \"$hash  panadiga-windows-x86_64.zip\" -Encoding UTF8 }"

echo.
echo ====================================================================
echo    [OK] BUILD CONCLUIDO COM SUCESSO!
echo    - Binario Executavel: dist\panadiga.exe
echo    - Runtime DLL:        dist\WebView2Loader.dll
echo    - Ferramentas:        dist\tools\ (Ghidra scripts, Python CLI, RAG)
echo    - Firmware Pico:      dist\firmware\ (.UF2 RP2040 e RP2350)
echo    - Pacote Release:     dist\releases\panadiga-windows-x86_64.zip
echo ====================================================================
if defined BUILD_ALL goto :build_android
if not defined CI if "%1"=="" pause
exit /b 0

:build_android
echo.
echo ====================================================================
echo   Panasonic DMR-EH55 DIGA Suite - Android APK Build Pipeline
echo   Tauri 2.0 Mobile / Rust cargo-ndk / AArch64 (ARM64)
echo ====================================================================

if "%ANDROID_HOME%"=="" (
    if exist "%LOCALAPPDATA%\Android\Sdk" (
        set "ANDROID_HOME=%LOCALAPPDATA%\Android\Sdk"
    ) else (
        echo [ERRO] ANDROID_HOME nao definido e SDK padrao nao encontrado.
        goto :error_exit
    )
)
echo [*] Android SDK detectado em: %ANDROID_HOME%

if "%NDK_HOME%"=="" (
    for /d %%D in ("%ANDROID_HOME%\ndk\*") do set "NDK_HOME=%%D"
)
if defined NDK_HOME (
    echo [*] Android NDK detectado em: %NDK_HOME%
    set "ANDROID_NDK_ROOT=%NDK_HOME%"
)

:: Ensure local.properties exists for Gradle
if not exist "gen\android" mkdir "gen\android"
powershell -NoProfile -Command "$sdk = '%ANDROID_HOME%'.Replace('\', '/'); $ndk = '%NDK_HOME%'.Replace('\', '/'); $c = 'sdk.dir=' + $sdk + [Environment]::NewLine; if ($ndk) { $c += 'ndk.dir=' + $ndk + [Environment]::NewLine }; [IO.File]::WriteAllText('gen/android/local.properties', $c, [Text.Encoding]::ASCII)"

echo [*] Verificando target aarch64-linux-android...
rustup target add aarch64-linux-android || goto :error_exit

echo [*] Compilando frontend React...
call npm --prefix apps\desktop run build || goto :error_exit

echo [*] Configurando crate-type temporario para Android cdylib...
powershell -Command "(Get-Content Cargo.toml) -replace 'crate-type = \[\"rlib\"\]', 'crate-type = [\"staticlib\", \"cdylib\", \"rlib\"]' | Set-Content Cargo.toml"

if not exist "gen\android\app" (
    echo [*] Inicializando modulo nativo Android com Tauri CLI...
    call npx @tauri-apps/cli android init --ci --skip-targets-install
)

echo [*] Executando build de APK via Tauri CLI (target: aarch64)...
call npx @tauri-apps/cli android build --apk --target aarch64
set ANDROID_STATUS=%ERRORLEVEL%

echo [*] Restaurando crate-type padrao para desktop (rlib)...
powershell -Command "(Get-Content Cargo.toml) -replace 'crate-type = \[\"staticlib\", \"cdylib\", \"rlib\"\]', 'crate-type = [\"rlib\"]' | Set-Content Cargo.toml"

if %ANDROID_STATUS% neq 0 (
    echo [ERRO] Falha no build do APK Android.
    goto :error_exit
)

:: Copy generated APK to dist\releases and update SHA256SUMS.txt
if not exist "dist\releases" mkdir "dist\releases"
powershell -NoProfile -Command "$apk = Get-ChildItem -Path 'gen\android\app\build\outputs\apk' -Recurse -Filter '*.apk' | Select-Object -First 1; if ($apk) { Copy-Item $apk.FullName 'dist\releases\panadiga-android-arm64.apk' -Force; $h = (Get-FileHash -Algorithm SHA256 'dist\releases\panadiga-android-arm64.apk').Hash; $txt = 'dist\releases\SHA256SUMS.txt'; $lines = if (Test-Path $txt) { (Get-Content $txt) | Where-Object { $_ -notmatch 'panadiga-android-arm64.apk' } } else { @() }; $lines += \"$h  panadiga-android-arm64.apk\"; Set-Content -Path $txt -Value $lines -Encoding UTF8 }"

echo ====================================================================
echo [OK] BUILD ANDROID CONCLUIDO COM SUCESSO!
echo      Arquivo APK consolidado em dist\releases\panadiga-android-arm64.apk
echo ====================================================================
if not defined CI if "%1"=="" pause
exit /b 0

:error_exit
echo.
echo ====================================================================
echo [ERRO CRITICO] O processo de build falhou ou foi interrompido.
echo ====================================================================
if not defined CI pause
exit /b 1

