@echo off
@chcp 65001 >nul
setlocal enabledelayedexpansion

echo ======================================================================
echo Panadiga Universal Code Quality and Commit Protection Gate (9 Gates)
echo ======================================================================

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

echo.
echo [1/9] Validating Rust Workspace...
cargo check --workspace --all-targets || goto :error
cargo test --workspace || goto :error

echo.
echo [2/9] Validating TypeScript Types...
cd /d "%SCRIPT_DIR%\apps\desktop"
call npm run typecheck || goto :error
cd /d "%SCRIPT_DIR%"

echo.
echo [3/9] Executing Desktop Frontend Unit Tests (Vitest)...
cd /d "%SCRIPT_DIR%\apps\desktop"
call npm test || goto :error
cd /d "%SCRIPT_DIR%"

echo.
echo [4/9] Running Copy/Paste Detector (jscpd)...
cd /d "%SCRIPT_DIR%"
call npx --prefix apps\desktop jscpd --config .jscpd.json || goto :error

echo.
echo [5/9] Running UI/UX State Linter (uxlint)...
cd /d "%SCRIPT_DIR%\apps\desktop"
call npm run uxlint || goto :error
cd /d "%SCRIPT_DIR%"

echo.
echo [6/9] Running Playwright End-to-End Tests (E2E)...
cd /d "%SCRIPT_DIR%\apps\desktop"
call npm run test:e2e || goto :error
cd /d "%SCRIPT_DIR%"

echo.
echo [7/9] Validating Ghidra Native Generator (Rust SSOT)...
cd /d "%SCRIPT_DIR%"
cargo run -- ghidra export-all --dir tools || goto :error

echo.
echo [8/9] Compiling & Executing C11 Pico Firmware Host Tests...
cd /d "%SCRIPT_DIR%"
gcc -std=c11 -Wall -Wextra -Werror -DPICO_HOST_TEST=1 -I dwire32a_jtag_research dwire32a_jtag_research\gdb_rsp.c dwire32a_jtag_research\tests\test_gdb_rsp_host.c -o test_gdb_rsp_host.exe || goto :error
.\test_gdb_rsp_host.exe || goto :error
del /f /q test_gdb_rsp_host.exe

echo.
echo [9/9] Verifying Python Typer CLI, Mypy Strict and Pytest Suite...
cd /d "%SCRIPT_DIR%"
python -m py_compile tools\ghidra_am34_memory_map.py tools\ghidra_import_symbols_panadiga.py tools\ghidra_connect_debugger.py || goto :error
python -m mypy --strict tools\panadiga_cli.py tools\rag_engine || goto :error
python -m pytest tools\tests tools\rag_engine\tests || goto :error

echo.
echo ======================================================================
echo [SUCCESS] ALL 9 COMMIT QUALITY GATES PASSED!
echo ======================================================================
exit /b 0

:error
cd /d "%SCRIPT_DIR%"
echo.
echo ======================================================================
echo [FAIL] Validation gate failed! Please resolve errors before committing.
echo ======================================================================
exit /b 1
