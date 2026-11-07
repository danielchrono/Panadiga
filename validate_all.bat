@echo off
setlocal enabledelayedexpansion

echo ======================================================================
echo Panadiga Universal Code Quality & Commit Protection Gate
echo ======================================================================

echo.
echo [1/6] Validating Rust Workspace...
cargo check --workspace --all-targets || goto :error
cargo test --workspace || goto :error

echo.
echo [2/6] Validating TypeScript Types...
cd apps\desktop
call npm run typecheck || goto :error
cd ..\..

echo.
echo [3/6] Executing Desktop Frontend Unit Tests (Vitest)...
cd apps\desktop
call npm test || goto :error
cd ..\..

echo.
echo [4/6] Running Copy/Paste Detector (jscpd)...
call npx --prefix apps\desktop jscpd --config .jscpd.json || goto :error

echo.
echo [5/6] Running UI/UX State Linter (uxlint)...
cd apps\desktop
call npm run uxlint || goto :error
cd ..\..

echo.
echo [6/6] Validating Python Tools (Typer CLI, Pytest & Mypy)...
python -m mypy tools\panadiga_cli.py || goto :error
python -m pytest tools\tests || goto :error

echo.
echo ======================================================================
echo [SUCCESS] ALL COMMIT QUALITY GATES PASSED!
echo ======================================================================
exit /b 0

:error
echo.
echo ======================================================================
echo [FAIL] Validation gate failed! Please resolve errors before committing.
echo ======================================================================
exit /b 1
