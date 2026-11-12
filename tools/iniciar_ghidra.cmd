@echo off
@chcp 65001 >nul
setlocal enabledelayedexpansion
title Panadiga - Inicializador do Ghidra

echo ====================================================================
echo    PANADIGA: ENGENHARIA REVERSA MATSUSHITA AM34 COM NSA GHIDRA
echo    Pasta de Scripts e Extensões: %~dp0
echo ====================================================================
echo.

if defined GHIDRA_HOME (
    if exist "%GHIDRA_HOME%\ghidraRun.bat" (
        echo [INFO] Iniciando Ghidra a partir de %%GHIDRA_HOME%%: %GHIDRA_HOME%
        start "" "%GHIDRA_HOME%\ghidraRun.bat"
        exit /b 0
    )
)

where ghidraRun.bat >nul 2>&1
if not errorlevel 1 (
    echo [INFO] Iniciando ghidraRun.bat encontrado no PATH do sistema...
    start "" ghidraRun.bat
    exit /b 0
)

where ghidra.exe >nul 2>&1
if not errorlevel 1 (
    echo [INFO] Iniciando ghidra.exe encontrado no PATH do sistema...
    start "" ghidra.exe
    exit /b 0
)

for %%D in (
    "C:\ghidra"
    "C:\tools\ghidra"
    "D:\ghidra"
    "%USERPROFILE%\ghidra"
    "C:\Program Files\ghidra"
) do (
    for /d %%F in ("%%~D\ghidra_*") do (
        if exist "%%~F\ghidraRun.bat" (
            echo [INFO] Ghidra detectado em %%~F! Iniciando...
            start "" "%%~F\ghidraRun.bat"
            exit /b 0
        )
    )
)

echo [AVISO] O executável principal do Ghidra (ghidraRun.bat) não foi encontrado no PATH
echo nem em %%GHIDRA_HOME%%.
echo.
echo Para abrir o Ghidra automaticamente:
echo   1. Baixe o Ghidra em https://ghidra-sre.org/ (Requer Java JDK 21+)
echo   2. Defina a variável de ambiente GHIDRA_HOME apontando para a pasta extraída.
echo.
echo Abrindo a pasta tools\ no Explorer com os 5 scripts de mapeamento de memória AM34...
echo.
start "" explorer "%~dp0"
pause
