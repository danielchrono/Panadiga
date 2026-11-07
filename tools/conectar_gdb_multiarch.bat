@echo off
setlocal enabledelayedexpansion
title Panadiga - Cliente GNU Debugger (AM34 / MN10300)

echo ====================================================================
echo    PANADIGA: CONEXAO GNU DEBUGGER (GDB) AO CHIP PANASONIC AM34
echo    Target: Panasonic UniPhier MN2DS0016 (Matsushita AM34 / MN103S)
echo    Porta Alvo: 127.0.0.1:3333 (Panadiga GDB RSP Bridge)
echo ====================================================================
echo.

where gdb-multiarch >nul 2>&1
if not errorlevel 1 (
    set "GDB_CMD=gdb-multiarch"
    goto :RUN_GDB
)

where mn10300-elf-gdb >nul 2>&1
if not errorlevel 1 (
    set "GDB_CMD=mn10300-elf-gdb"
    goto :RUN_GDB
)

where gdb >nul 2>&1
if not errorlevel 1 (
    set "GDB_CMD=gdb"
    goto :RUN_GDB
)

echo [ERRO] Nenhum executavel GDB (gdb-multiarch ou mn10300-elf-gdb) encontrado no PATH do sistema.
echo.
echo Para instalar:
echo   - No Linux/WSL: sudo apt install gdb-multiarch
echo   - No Windows: Baixe a toolchain GCC MN10300 ou use o Ghidra Debugger / IDA Pro apontando para 127.0.0.1:3333
echo.
pause
exit /b 1

:RUN_GDB
echo [INFO] Iniciando depurador com !GDB_CMD!...
echo [INFO] Configurando arquitetura Matsushita MN10300 e conectando em 127.0.0.1:3333...
echo.

!GDB_CMD! -ex "set architecture mn10300" -ex "target remote 127.0.0.1:3333"

pause
