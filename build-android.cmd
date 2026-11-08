@echo off
setlocal enabledelayedexpansion

echo ===============================================================================
echo  Panasonic DMR-EH55 DIGA Suite - Android APK Automated Build System
echo  Tauri 2.0 Mobile / Rust cargo-ndk / AArch64 (ARM64)
echo ===============================================================================

:: 1. Validate Android SDK
if "%ANDROID_HOME%"=="" (
    if exist "%LOCALAPPDATA%\Android\Sdk" (
        set "ANDROID_HOME=%LOCALAPPDATA%\Android\Sdk"
    ) else (
        echo [ERRO] ANDROID_HOME nao definido e SDK padrao nao encontrado em %%LOCALAPPDATA%%\Android\Sdk.
        echo Instale o Android SDK e configure a variavel de ambiente ANDROID_HOME.
        exit /b 1
    )
)
echo [*] Android SDK detectado em: %ANDROID_HOME%

:: 2. Validate Android NDK
if "%NDK_HOME%"=="" (
    for /d %%D in ("%ANDROID_HOME%\ndk\*") do (
        set "NDK_HOME=%%D"
    )
)
if "%NDK_HOME%"=="" (
    echo [AVISO] NDK_HOME nao configurado. Tentando compilar via npx @tauri-apps/cli...
) else (
    echo [*] Android NDK detectado em: %NDK_HOME%
    set "ANDROID_NDK_ROOT=%NDK_HOME%"
)

:: 3. Ensure Rust Target is Installed
echo [*] Verificando target aarch64-linux-android...
rustup target add aarch64-linux-android
if %ERRORLEVEL% neq 0 (
    echo [ERRO] Falha ao adicionar target rustup aarch64-linux-android.
    exit /b %ERRORLEVEL%
)

:: 4. Build React / Vite Frontend
echo [*] Compilando frontend React (apps/desktop)...
call npm --prefix apps/desktop run build
if %ERRORLEVEL% neq 0 (
    echo [ERRO] Falha no build de producao do frontend Vite.
    exit /b %ERRORLEVEL%
)

:: 5. Temporarily configure crate-type for Android shared library (.so)
echo [*] Configurando crate-type para Android cdylib...
powershell -Command "(Get-Content Cargo.toml) -replace 'crate-type = \[\"rlib\"\]', 'crate-type = [\"staticlib\", \"cdylib\", \"rlib\"]' | Set-Content Cargo.toml"

:: 6. Initialize Android if not initialized
if not exist "gen\android" if not exist "src-tauri\gen\android" (
    echo [*] Inicializando modulo nativo Android com Tauri CLI...
    call npx @tauri-apps/cli android init --ci --skip-targets-install
)

:: 7. Build Android APK
echo [*] Executando build de APK via Tauri CLI...
call npx @tauri-apps/cli android build --apk --target aarch64-linux-android
set BUILD_STATUS=%ERRORLEVEL%

:: 8. Restore desktop rlib crate-type
echo [*] Restaurando crate-type padrao para desktop (rlib)...
powershell -Command "(Get-Content Cargo.toml) -replace 'crate-type = \[\"staticlib\", \"cdylib\", \"rlib\"\]', 'crate-type = [\"rlib\"]' | Set-Content Cargo.toml"

if %BUILD_STATUS% neq 0 (
    echo [ERRO] Falha no build do APK Android.
    exit /b %BUILD_STATUS%
)

echo ===============================================================================
echo  BUILD ANDROID CONCLUIDO COM SUCESSO!
echo  O arquivo .apk foi gerado em: gen\android\app\build\outputs\apk\
echo ===============================================================================
