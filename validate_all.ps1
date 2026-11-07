<#
.SYNOPSIS
    Panadiga Engineering Ecosystem - Universal Code Quality & Commit Protection Gate
    Validates Rust, TypeScript, Frontend UX, Copy-Paste Duplication (jscpd),
    UI/UX Lints (uxlint), and Python Typer CLI before committing.
#>

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
    Write-Step "1/6. Validating Rust Workspace (Cargo Check & Tests)"
    Set-Location $ScriptRoot
    cargo check --workspace --all-targets
    Assert-LastExitCode "Cargo Check"
    cargo test --workspace --verbose
    Assert-LastExitCode "Cargo Test"
    Write-Success "Rust engine compiled cleanly and all tests passed!"

    # -------------------------------------------------------------------------
    # 2. TypeScript Strict Type-Check (tsc --noEmit)
    # -------------------------------------------------------------------------
    Write-Step "2/6. Validating TypeScript Types (tsc --noEmit)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm run typecheck
    Assert-LastExitCode "TypeScript TypeCheck"
    Write-Success "TypeScript strict type check succeeded with 0 errors!"

    # -------------------------------------------------------------------------
    # 3. Frontend Unit & ViewModel Tests (Vitest)
    # -------------------------------------------------------------------------
    Write-Step "3/6. Executing Desktop Frontend Unit Tests (Vitest)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm test
    Assert-LastExitCode "Vitest Suite"
    Write-Success "All Vitest frontend and ViewModel unit tests passed!"

    # -------------------------------------------------------------------------
    # 4. Copy/Paste Code Duplication Gate (jscpd)
    # -------------------------------------------------------------------------
    Write-Step "4/6. Running Copy/Paste Detector (jscpd threshold <= 5%)"
    Set-Location $ScriptRoot
    npx --prefix apps/desktop jscpd --config .jscpd.json
    Assert-LastExitCode "jscpd Duplication Scan"
    Write-Success "Code duplication check verified within safety limits!"

    # -------------------------------------------------------------------------
    # 5. UI/UX Completeness Linter (uxlint)
    # -------------------------------------------------------------------------
    Write-Step "5/6. Running UI/UX State Linter (uxlint)"
    Set-Location "$ScriptRoot/apps/desktop"
    npm run uxlint
    Assert-LastExitCode "uxlint UX Scan"
    Write-Success "UX state audit completed!"

    # -------------------------------------------------------------------------
    # 6. Ghidra Native Script Generator Gate (panadiga ghidra)
    # -------------------------------------------------------------------------
    Write-Step "6/6. Validating Ghidra Native Generator (panadiga ghidra)"
    Set-Location $ScriptRoot
    cargo run -p panadiga -- ghidra export-all --dir tools
    Assert-LastExitCode "Ghidra Script Generator"
    Write-Success "Ghidra scripts generated directly from native Rust SSOT!"

    $elapsed = (Get-Date) - $startTime
    Write-Host ""
    Write-Host "======================================================================" -ForegroundColor Green
    Write-Host "[SUCCESS] ALL COMMIT GATES PASSED! (Elapsed: $($elapsed.TotalSeconds.ToString('F1'))s)" -ForegroundColor Green
    Write-Host "Repository is in pristine condition for git commit & push." -ForegroundColor Green
    Write-Host "======================================================================" -ForegroundColor Green
    exit 0
} catch {
    Set-Location $ScriptRoot
    Write-Host ""
    Write-Failure "Validation gate failed: $_"
    exit 1
}
