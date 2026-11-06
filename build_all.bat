@echo off
setlocal enabledelayedexpansion

echo ====================================================================
echo    PANASONIC DMR-EH55 DIGA ENGINEERING SUITE - UNIFIED BUILD
echo    Target Platform: Windows x86_64 (CLI + Desktop Applications)
echo ====================================================================

:: 1. Environment Verification
echo.
echo [1/4] Verificando ferramentas do ambiente (Rust e Node.js)...
call cargo --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERRO] Cargo/Rust nao encontrado no PATH! Instale o Rust.
    exit /b 1
)

call node --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERRO] Node.js nao encontrado no PATH! Instale o Node.js v18+.
    exit /b 1
)

call npm --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERRO] NPM nao encontrado no PATH!
    exit /b 1
)
echo [OK] Ambiente verificado com sucesso.

:: 2. Rust Workspace Test Suite
echo.
echo [2/4] Executando bateria de testes do workspace (cargo test)...
cargo test
if %ERRORLEVEL% neq 0 (
    echo [ERRO] Falha na execucao dos testes do Cargo! Abortando.
    exit /b %ERRORLEVEL%
)
echo [OK] 100%% dos testes unitarios e integrados passaram!

:: 3. Rust CLI Release Compilation
echo.
echo [3/4] Compilando CLI panadiga em modo release (target/release/panadiga.exe)...
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo [ERRO] Falha na compilacao da CLI Rust! Abortando.
    exit /b %ERRORLEVEL%
)

if not exist "target\release\panadiga.exe" (
    echo [ERRO] Binario target\release\panadiga.exe nao foi encontrado!
    exit /b 1
)
echo [OK] Binario CLI gerado: target\release\panadiga.exe

:: 4. Desktop Interface Build
echo.
echo [4/4] Compilando interface Desktop (apps/desktop)...
cd apps\desktop
if not exist "node_modules" (
    echo [*] Instalando dependencias npm...
    call npm install
    if !ERRORLEVEL! neq 0 (
        echo [ERRO] npm install falhou!
        cd ..\..
        exit /b !ERRORLEVEL!
    )
)

echo [*] Executando npm run build...
call npm run build
if !ERRORLEVEL! neq 0 (
    echo [ERRO] npm run build falhou!
    cd ..\..
    exit /b !ERRORLEVEL!
)

if not exist "dist\index.html" (
    echo [ERRO] Artefato dist\index.html nao foi gerado!
    cd ..\..
    exit /b 1
)
cd ..\..
echo [OK] Interface Desktop compilada: apps\desktop\dist\index.html

echo.
echo ====================================================================
echo    PIPELINE UNIFICADO CONCLUIDO COM 100%% DE SUCESSO!
echo ====================================================================
echo   [OK] CLI Panadiga: target\release\panadiga.exe
echo   [OK] Desktop GUI : apps\desktop\dist\index.html
echo ====================================================================
echo.

exit /b 0
