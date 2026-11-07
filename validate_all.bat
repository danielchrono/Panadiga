@echo off
@chcp 65001 >nul
setlocal enabledelayedexpansion

echo ======================================================================
echo Panadiga Universal Code Quality and Commit Protection Gate (8 Gates)
echo ======================================================================

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

echo.
echo [1/8] Validating Rust Workspace...
cargo check --workspace --all-targets || goto :error
cargo test --workspace || goto :error

echo.
echo [2/8] Validating TypeScript Types...
cd /d "%SCRIPT_DIR%\apps\desktop"
call npm run typecheck || goto :error
cd /d "%SCRIPT_DIR%"

echo.
echo [3/8] Executing Desktop Frontend Unit Tests (Vitest)...
cd /d "%SCRIPT_DIR%\apps\desktop"
call npm test || goto :error
cd /d "%SCRIPT_DIR%"

echo.
echo [4/8] Running Copy/Paste Detector (jscpd)...
cd /d "%SCRIPT_DIR%"
call npx --prefix apps\desktop jscpd --config .jscpd.json || goto :error

echo.
echo [5/8] Running UI/UX State Linter (uxlint)...
cd /d "%SCRIPT_DIR%\apps\desktop"
call npm run uxlint || goto :error
cd /d "%SCRIPT_DIR%"

echo.
echo [6/8] Running Playwright End-to-End Tests (E2E)...
cd /d "%SCRIPT_DIR%\apps\desktop"
call npm run test:e2e || goto :error
cd /d "%SCRIPT_DIR%"

echo.
echo [7/8] Validating Ghidra Native Generator (Rust SSOT)...
cd /d "%SCRIPT_DIR%"
cargo run -- ghidra export-all --dir tools || goto :error

echo.
echo [8/8] Verifying Python Typer CLI, Mypy and Pytest Suite...
cd /d "%SCRIPT_DIR%"
python -m py_compile tools\ghidra_am34_memory_map.py tools\ghidra_importar_simbolos_panadiga.py tools\ghidra_conectar_debugger.py || goto :error
python -m mypy tools\panadiga_cli.py || goto :error
python -m pytest tools\tests tools\rag_engine\tests || goto :error

echo.
echo ======================================================================
echo [SUCCESS] ALL 8 COMMIT QUALITY GATES PASSED!
echo ======================================================================
exit /b 0

:error
cd /d "%SCRIPT_DIR%"
echo.
echo ======================================================================
echo [FAIL] Validation gate failed! Please resolve errors before committing.
echo ======================================================================
exit /b 1
