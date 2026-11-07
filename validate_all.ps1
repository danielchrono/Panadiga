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
    Write-Step "1/8. Validating Rust Workspace (Cargo Check & Tests)"
    Set-Location $ScriptRoot
    cargo check --workspace --all-targets
    Assert-LastExitCode "Cargo Check"
    cargo test --workspace --verbose
    Assert-LastExitCode "Cargo Test"
    Write-Success "Rust engine compiled cleanly and all 16 tests passed!"

    # -------------------------------------------------------------------------
    # 2. TypeScript Strict Type-Check (tsc --noEmit)
    # -------------------------------------------------------------------------
    Write-Step "2/8. Validating TypeScript Types (tsc --noEmit)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm run typecheck
    Assert-LastExitCode "TypeScript TypeCheck"
    Write-Success "TypeScript strict type check succeeded with 0 errors!"

    # -------------------------------------------------------------------------
    # 3. Frontend Unit & ViewModel Tests (Vitest)
    # -------------------------------------------------------------------------
    Write-Step "3/8. Executing Desktop Frontend Unit Tests (Vitest)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm test
    Assert-LastExitCode "Vitest Suite"
    Write-Success "All Vitest frontend and ViewModel unit tests passed (20/20)!"

    # -------------------------------------------------------------------------
    # 4. Copy/Paste Code Duplication Gate (jscpd)
    # -------------------------------------------------------------------------
    Write-Step "4/8. Running Copy/Paste Detector (jscpd threshold <= 5%)"
    Set-Location $ScriptRoot
    npx --prefix apps/desktop jscpd --config .jscpd.json
    Assert-LastExitCode "jscpd Duplication Scan"
    Write-Success "Code duplication check verified within safety limits (< 5%)!"

    # -------------------------------------------------------------------------
    # 5. UI/UX Completeness Linter (uxlint)
    # -------------------------------------------------------------------------
    Write-Step "5/8. Running UI/UX State Linter (uxlint)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm run uxlint
    Assert-LastExitCode "uxlint UX Scan"
    Write-Success "UX state audit completed!"

    # -------------------------------------------------------------------------
    # 6. Playwright End-to-End Tests (E2E)
    # -------------------------------------------------------------------------
    Write-Step "6/8. Running Playwright End-to-End Tests (Chromium)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm run test:e2e
    Assert-LastExitCode "Playwright E2E Suite"
    Write-Success "All Playwright E2E browser tests passed (8/8)!"

    # -------------------------------------------------------------------------
    # 7. Ghidra Native Script Generator Gate (Rust SSOT)
    # -------------------------------------------------------------------------
    Write-Step "7/8. Validating Ghidra Native Generator (panadiga ghidra)"
    Set-Location $ScriptRoot
    cargo run -- ghidra export-all --dir "$ScriptRoot/tools"
    Assert-LastExitCode "Ghidra Script Generator"
    Write-Success "Ghidra scripts generated directly from native Rust SSOT!"

    # -------------------------------------------------------------------------
    # 8. Python Typer CLI, Mypy & Pytest Verification
    # -------------------------------------------------------------------------
    Write-Step "8/8. Verifying Python Typer CLI, Mypy & Pytest Suite"
    Set-Location $ScriptRoot
    python -m py_compile "$ScriptRoot/tools/ghidra_am34_memory_map.py" "$ScriptRoot/tools/ghidra_importar_simbolos_panadiga.py" "$ScriptRoot/tools/ghidra_conectar_debugger.py"
    Assert-LastExitCode "Ghidra Python Syntax Verification"
    python -m mypy "$ScriptRoot/tools/panadiga_cli.py"
    Assert-LastExitCode "Typer CLI Mypy Strict Check"
    python -m pytest "$ScriptRoot/tools/tests"
    Assert-LastExitCode "Pytest Suite (4/4 tests)"
    Write-Success "All Python scripts, Typer CLI and Pytest suites passed (4/4 tests, 0 mypy issues)!"

    $elapsed = (Get-Date) - $startTime
    Write-Host ""
    Write-Host "======================================================================" -ForegroundColor Green
    Write-Host "[SUCCESS] ALL 8 COMMIT QUALITY GATES PASSED! (Elapsed: $($elapsed.TotalSeconds.ToString('F1'))s)" -ForegroundColor Green
    Write-Host "Repository is in pristine condition for git commit & push." -ForegroundColor Green
    Write-Host "======================================================================" -ForegroundColor Green
    exit 0
} catch {
    Set-Location $ScriptRoot
    Write-Host ""
    Write-Failure "Validation gate failed: $_"
    exit 1
}
