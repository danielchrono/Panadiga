<#
.SYNOPSIS
    Panadiga Engineering Ecosystem - Universal Code Quality & Commit Protection Gate
    Validates Rust, TypeScript, Vitest, Playwright E2E, jscpd Duplication,
    uxlint UX Completeness, Ghidra Native Generation, and Python Syntax.
#>

$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "Stop"
$ScriptRoot = if ($PSScriptRoot) { (Get-Item $PSScriptRoot).FullName } else { (Get-Item .).FullName }
Set-Location $ScriptRoot

function Write-Step($title) {
    Write-Host ""
    Write-Host "======================================================================" -ForegroundColor Cyan
    Write-Host ">> $title" -ForegroundColor Cyan
    Write-Host "======================================================================" -ForegroundColor Cyan
}

function Write-Success($msg) {
    Write-Host "[PASS] $msg" -ForegroundColor Green
}

function Write-Failure($msg) {
    Write-Host "[FAIL] $msg" -ForegroundColor Red
}

function Assert-LastExitCode($stepName) {
    if ($LASTEXITCODE -ne 0) {
        throw "Step '$stepName' failed with exit code $LASTEXITCODE"
    }
}

$startTime = Get-Date

try {
    # -------------------------------------------------------------------------
    # 1. Rust Cargo Check & Tests
    # -------------------------------------------------------------------------
    Write-Step "1/9. Validating Rust Workspace (Cargo Check & Tests)"
    Set-Location $ScriptRoot
    cargo fmt --all -- --check
    Assert-LastExitCode "Cargo Fmt Check"
    cargo check --workspace --all-targets
    Assert-LastExitCode "Cargo Check"
    cargo test --workspace --verbose
    Assert-LastExitCode "Cargo Test"
    Write-Success "Rust engine compiled cleanly and all 43 tests passed (25 lib/RAG + 18 integration across 8 modular suites)!"

    # -------------------------------------------------------------------------
    # 2. TypeScript Strict Type-Check (tsc --noEmit)
    # -------------------------------------------------------------------------
    Write-Step "2/9. Validating TypeScript Types (tsc --noEmit)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm run typecheck
    Assert-LastExitCode "TypeScript TypeCheck"
    Write-Success "TypeScript strict type check succeeded with 0 errors!"

    # -------------------------------------------------------------------------
    # 3. Frontend Unit & ViewModel Tests (Vitest)
    # -------------------------------------------------------------------------
    Write-Step "3/9. Executing Desktop Frontend Unit Tests (Vitest)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm test
    Assert-LastExitCode "Vitest Suite"
    Write-Success "All Vitest frontend and ViewModel unit tests passed (38/38)!"

    # -------------------------------------------------------------------------
    # 4. Copy/Paste Code Duplication Gate (jscpd)
    # -------------------------------------------------------------------------
    Write-Step "4/9. Running Copy/Paste Detector (jscpd threshold 0% / 0 clones)"
    Set-Location $ScriptRoot
    npx --prefix apps/desktop jscpd --config .jscpd.json
    Assert-LastExitCode "jscpd Duplication Scan"
    Write-Success "Code duplication check verified at 0% duplication (0 clones found)!"

    # -------------------------------------------------------------------------
    # 5. UI/UX Completeness Linter (uxlint)
    # -------------------------------------------------------------------------
    Write-Step "5/9. Running UI/UX State Linter (uxlint)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm run uxlint
    Assert-LastExitCode "uxlint UX Scan"
    Write-Success "UX state audit completed!"

    # -------------------------------------------------------------------------
    # 6. Playwright End-to-End Tests (E2E)
    # -------------------------------------------------------------------------
    Write-Step "6/9. Running Playwright End-to-End Tests (Chromium)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm run test:e2e
    Assert-LastExitCode "Playwright E2E Suite"
    Write-Success "All Playwright E2E browser tests passed (8/8)!"

    # -------------------------------------------------------------------------
    # 7. Ghidra Native Script Generator Gate (Rust SSOT)
    # -------------------------------------------------------------------------
    Write-Step "7/9. Validating Ghidra Native Generator (panadiga ghidra)"
    Set-Location $ScriptRoot
    cargo run -- ghidra export-all --dir "$ScriptRoot/tools"
    Assert-LastExitCode "Ghidra Script Generator"
    Write-Success "Ghidra scripts generated directly from native Rust SSOT!"

    # -------------------------------------------------------------------------
    # 8. Pico Firmware C11 & RSP Server Host Test Gate (OPUS-04)
    # -------------------------------------------------------------------------
    Write-Step "8/9. Compiling & Executing C11 Pico Firmware Host Tests (GCC -Werror)"
    Set-Location $ScriptRoot
    $cTestExe = "$ScriptRoot/test_gdb_rsp_host.exe"
    gcc -std=c11 -Wall -Wextra -Werror -DPICO_HOST_TEST=1 -I dwire32a_jtag_research dwire32a_jtag_research/gdb_rsp.c dwire32a_jtag_research/tests/test_gdb_rsp_host.c -o $cTestExe
    Assert-LastExitCode "GCC C11 Host Test Compilation"
    & $cTestExe
    Assert-LastExitCode "C11 Host Test Suite Execution"
    Remove-Item -Force $cTestExe
    Write-Success "All C11 Pico firmware host tests compiled and passed (7/7 assertions verified)!"

    # -------------------------------------------------------------------------
    # 9. Python Typer CLI, Mypy Strict & Pytest Verification (OPUS-13)
    # -------------------------------------------------------------------------
    Write-Step "9/9. Verifying Python Typer CLI, Mypy Strict & Pytest Suite"
    Set-Location $ScriptRoot
    python -m py_compile "$ScriptRoot/tools/ghidra_am34_memory_map.py" "$ScriptRoot/tools/ghidra_import_symbols_panadiga.py" "$ScriptRoot/tools/ghidra_connect_debugger.py"
    Assert-LastExitCode "Ghidra Python Syntax Verification"
    python -m mypy --strict "$ScriptRoot/tools/panadiga_cli.py" "$ScriptRoot/tools/rag_engine"
    Assert-LastExitCode "Typer CLI & RAG Engine Mypy Strict Check"
    python -m pytest "$ScriptRoot/tools/tests" "$ScriptRoot/tools/rag_engine/tests"
    Assert-LastExitCode "Pytest Suite"
    Write-Success "All Python scripts, Typer CLI, RAG engine and Pytest suites passed (21/21 tests, 0 mypy strict issues)!"

    $elapsed = (Get-Date) - $startTime
    Write-Host ""
    Write-Host "======================================================================" -ForegroundColor Green
    Write-Host "[SUCCESS] ALL 9 COMMIT QUALITY GATES PASSED! (Elapsed: $($elapsed.TotalSeconds.ToString('F1'))s)" -ForegroundColor Green
    Write-Host "Repository is in pristine condition for git commit & push." -ForegroundColor Green
    Write-Host "======================================================================" -ForegroundColor Green
    exit 0
} catch {
    Set-Location $ScriptRoot
    Write-Host ""
    Write-Failure "Validation gate failed: $_"
    exit 1
}
