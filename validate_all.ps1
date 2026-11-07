<#
.SYNOPSIS
    Panadiga Engineering Ecosystem - Universal Code Quality & Commit Protection Gate
    Validates Rust, TypeScript, Frontend UX, Copy-Paste Duplication (jscpd),
    UI/UX Lints (uxlint), and Python Typer CLI before committing.
#>

$ErrorActionPreference = "Stop"

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

$startTime = Get-Date

try {
    # -------------------------------------------------------------------------
    # 1. Rust Cargo Check & Tests
    # -------------------------------------------------------------------------
    Write-Step "1/6. Validating Rust Workspace (Cargo Check & Tests)"
    cargo check --workspace --all-targets
    cargo test --workspace --verbose
    Write-Success "Rust engine compiled cleanly and all tests passed!"

    # -------------------------------------------------------------------------
    # 2. TypeScript Strict Type-Check (tsc --noEmit)
    # -------------------------------------------------------------------------
    Write-Step "2/6. Validating TypeScript Types (tsc --noEmit)"
    Push-Location apps/desktop
    try {
        npm run typecheck
        Write-Success "TypeScript strict type check succeeded with 0 errors!"
    } finally {
        Pop-Location
    }

    # -------------------------------------------------------------------------
    # 3. Frontend Unit & ViewModel Tests (Vitest)
    # -------------------------------------------------------------------------
    Write-Step "3/6. Executing Desktop Frontend Unit Tests (Vitest)"
    Push-Location apps/desktop
    try {
        npm test
        Write-Success "All Vitest frontend and ViewModel unit tests passed!"
    } finally {
        Pop-Location
    }

    # -------------------------------------------------------------------------
    # 4. Copy/Paste Code Duplication Gate (jscpd)
    # -------------------------------------------------------------------------
    Write-Step "4/6. Running Copy/Paste Detector (jscpd threshold <= 5%)"
    npx --prefix apps/desktop jscpd --config .jscpd.json
    Write-Success "Code duplication check verified within safety limits!"

    # -------------------------------------------------------------------------
    # 5. UI/UX Completeness Linter (uxlint)
    # -------------------------------------------------------------------------
    Write-Step "5/6. Running UI/UX State Linter (uxlint)"
    Push-Location apps/desktop
    try {
        npm run uxlint
        Write-Success "UX state audit completed!"
    } finally {
        Pop-Location
    }

    # -------------------------------------------------------------------------
    # 6. Python Typer CLI & Type-Checking (pytest + mypy)
    # -------------------------------------------------------------------------
    Write-Step "6/6. Validating Python Tools (Typer CLI, Pytest & Mypy)"
    python -m mypy tools/panadiga_cli.py
    python -m pytest tools/tests -v
    Write-Success "Python Typer CLI verified with strict types and passing tests!"

    $elapsed = (Get-Date) - $startTime
    Write-Host ""
    Write-Host "======================================================================" -ForegroundColor Green
    Write-Host "[SUCCESS] ALL COMMIT GATES PASSED! (Elapsed: $($elapsed.TotalSeconds.ToString('F1'))s)" -ForegroundColor Green
    Write-Host "Repository is in pristine condition for git commit & push." -ForegroundColor Green
    Write-Host "======================================================================" -ForegroundColor Green
    exit 0
} catch {
    Write-Host ""
    Write-Failure "Validation gate failed: $_"
    exit 1
}
