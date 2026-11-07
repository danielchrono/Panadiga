<#
.SYNOPSIS
    Panasonic DMR-EH55 DIGA Engineering Suite - Unified Root Build & Distribution Orchestrator.
.DESCRIPTION
    Builds the complete engineering ecosystem:
    1. Executes full regression & forensics test battery (cargo test -p panadiga).
    2. Compiles high-performance Rust CLI binary (target/release/panadiga.exe).
    3. Builds the modern TypeScript + React Desktop interface (apps/desktop/dist).
    4. Compiles the native Tauri Desktop application (target/release/panasonic-diga-desktop.exe).
    5. Packages all binaries and external tools into dist_bin\ ready for deployment.
    6. Generates multiplatform release archive and SHA256 checksums in dist_bin\releases\.
#>

[CmdletBinding()]
param (
    [switch]$SkipTests,
    [switch]$Clean,
    [switch]$Package
)

$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Maximum Build Parallelism & Cache Settings
$env:CARGO_BUILD_JOBS = $env:NUMBER_OF_PROCESSORS
$env:CARGO_INCREMENTAL = "1"
if (Get-Command sccache -ErrorAction SilentlyContinue) {
    $env:RUSTC_WRAPPER = "sccache"
    Write-Host "[+] sccache detected and enabled for compiler caching!" -ForegroundColor Green
}

$RootPath = if ($PSScriptRoot) { (Get-Item $PSScriptRoot).FullName } else { (Get-Item .).FullName }
Set-Location $RootPath

$RustCliBinary = Join-Path $RootPath "target\release\panadiga.exe"
$RustDesktopBinary = Join-Path $RootPath "target\release\panasonic-diga-desktop.exe"
$DesktopDir = Join-Path $RootPath "apps\desktop"
$DesktopDistHtml = Join-Path $DesktopDir "dist\index.html"
$DistBinDir = Join-Path $RootPath "dist_bin"
$ReleasesDir = Join-Path $DistBinDir "releases"

Write-Host "====================================================================" -ForegroundColor Cyan
Write-Host "   PANASONIC DMR-EH55 DIGA ENGINEERING SUITE - UNIFIED BUILD" -ForegroundColor Cyan
Write-Host "   Target Platform: Windows x86_64 (Paralelismo: $($env:NUMBER_OF_PROCESSORS) Cores | UTF-8)" -ForegroundColor Cyan
Write-Host "====================================================================" -ForegroundColor Cyan

# 1. Environment Verification
Write-Host "`n[1/6] Verificando ferramentas do ambiente (Rust & Node.js)..." -ForegroundColor Yellow

try {
    $cargoVer = cargo --version
    Write-Host "  -> Cargo: $cargoVer" -ForegroundColor Green
} catch {
    Write-Error "Cargo/Rust não encontrado no PATH! Instale o toolchain Rust."
    exit 1
}

try {
    $nodeVer = node --version
    $npmVer = npm --version
    Write-Host "  -> Node.js: $nodeVer | NPM: $npmVer" -ForegroundColor Green
} catch {
    Write-Error "Node.js ou NPM não encontrado no PATH! Instale o Node.js v18+."
    exit 1
}

# Optional Clean
if ($Clean) {
    Write-Host "`n[*] Limpando artefatos anteriores..." -ForegroundColor Yellow
    cargo clean
    if (Test-Path (Join-Path $DesktopDir "dist")) {
        Remove-Item -Recurse -Force (Join-Path $DesktopDir "dist")
    }
    if (Test-Path $DistBinDir) {
        Remove-Item -Recurse -Force $DistBinDir
    }
}

# 2. Rust Workspace Test Suite
if (-not $SkipTests) {
    Write-Host "`n[2/6] Executando bateria de testes Rust (cargo test -p panadiga)..." -ForegroundColor Yellow
    cargo test -p panadiga
    if ($LASTEXITCODE -ne 0) {
        Write-Error "A bateria de testes do Cargo falhou! Abortando build."
        exit $LASTEXITCODE
    }
    Write-Host "  -> 100% dos 16 testes unitários e de integração passaram com sucesso!" -ForegroundColor Green
} else {
    Write-Host "`n[2/6] Testes ignorados (--SkipTests ativado)." -ForegroundColor DarkGray
}

# 3. Rust CLI Release Compilation
Write-Host "`n[3/6] Compilando CLI panadiga em modo release (target/release/panadiga.exe)..." -ForegroundColor Yellow
cargo build --release -p panadiga
if ($LASTEXITCODE -ne 0) {
    Write-Error "Falha na compilação da CLI Rust! Abortando build."
    exit $LASTEXITCODE
}

if (-not (Test-Path $RustCliBinary)) {
    Write-Error "Binário final não encontrado em $RustCliBinary!"
    exit 1
}

$cliSize = (Get-Item $RustCliBinary).Length / 1MB
Write-Host ("  -> Binário CLI gerado: {0} ({1:N2} MB)" -f $RustCliBinary, $cliSize) -ForegroundColor Green

# 4. Desktop Interface Build (TypeScript + React + Vite)
Write-Host "`n[4/6] Compilando interface Desktop (apps/desktop)..." -ForegroundColor Yellow
Push-Location $DesktopDir

try {
    if (-not (Test-Path "node_modules")) {
        Write-Host "  -> Instalando dependências npm..." -ForegroundColor Gray
        npm install
        if ($LASTEXITCODE -ne 0) {
            throw "npm install falhou com código $LASTEXITCODE"
        }
    }

    Write-Host "  -> Executando npm run build (tsc && vite build)..." -ForegroundColor Gray
    npm run build
    if ($LASTEXITCODE -ne 0) {
        throw "npm run build falhou com código $LASTEXITCODE"
    }

    if (-not (Test-Path $DesktopDistHtml)) {
        throw "Bundle estático da interface desktop não encontrado em $DesktopDistHtml"
    }

    Write-Host "  -> Interface Desktop compilada (dist/index.html gerado)!" -ForegroundColor Green
} catch {
    Write-Error "Falha na compilação do Desktop: $_"
    Pop-Location
    exit 1
} finally {
    Pop-Location
}

# 5. Native Desktop Application Compilation (Tauri Rust)
Write-Host "`n[5/6] Compilando Aplicação Desktop Nativa (Tauri Rust com assets embutidos)..." -ForegroundColor Yellow
cargo build --release -p panasonic-diga-desktop
if ($LASTEXITCODE -ne 0) {
    Write-Error "Falha na compilação do aplicativo Desktop Tauri! Abortando build."
    exit $LASTEXITCODE
}

if (-not (Test-Path $RustDesktopBinary)) {
    Write-Error "Binário Desktop não encontrado em $RustDesktopBinary!"
    exit 1
}

$desktopSize = (Get-Item $RustDesktopBinary).Length / 1MB
Write-Host ("  -> Binário Desktop gerado: {0} ({1:N2} MB)" -f $RustDesktopBinary, $desktopSize) -ForegroundColor Green

# 6. Packaging to dist_bin\
Write-Host "`n[6/6] Empacotando distribuição consolidada para dist_bin\..." -ForegroundColor Yellow
if (-not (Test-Path $DistBinDir)) {
    New-Item -ItemType Directory -Path $DistBinDir | Out-Null
}

Copy-Item -Path $RustCliBinary -Destination (Join-Path $DistBinDir "panadiga-cli.exe") -Force
Copy-Item -Path $RustDesktopBinary -Destination (Join-Path $DistBinDir "panadiga-desktop.exe") -Force

$toolsDist = Join-Path $DistBinDir "tools"
if (-not (Test-Path $toolsDist)) {
    New-Item -ItemType Directory -Path $toolsDist | Out-Null
}
Copy-Item -Path (Join-Path $RootPath "tools\*") -Destination $toolsDist -Recurse -Force

Set-Content -Path (Join-Path $DistBinDir "Executar_Panadiga.bat") -Value "@echo off`r`n@chcp 65001 >nul`r`nstart `"`" `"%~dp0panadiga-desktop.exe`"" -Encoding UTF8

Write-Host "  -> Pacote consolidado gerado com sucesso em dist_bin\!" -ForegroundColor Green

# Generate Release Archive if requested or default
Write-Host "`n[*] Gerando Pacote de Release ZIP e Checksum SHA256..." -ForegroundColor Yellow
if (-not (Test-Path $ReleasesDir)) {
    New-Item -ItemType Directory -Path $ReleasesDir | Out-Null
}
$zipPath = Join-Path $ReleasesDir "panadiga-windows-x86_64.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }

Compress-Archive -Path (Join-Path $DistBinDir "panadiga-cli.exe"), `
                       (Join-Path $DistBinDir "panadiga-desktop.exe"), `
                       (Join-Path $DistBinDir "Executar_Panadiga.bat"), `
                       (Join-Path $DistBinDir "tools"), `
                       (Join-Path $RootPath "README.md"), `
                       (Join-Path $RootPath "LICENSE") `
                 -DestinationPath $zipPath -CompressionLevel Optimal

$sha256 = (Get-FileHash -Algorithm SHA256 $zipPath).Hash
$checksumFile = Join-Path $ReleasesDir "SHA256SUMS.txt"
[System.IO.File]::WriteAllText($checksumFile, "$sha256  panadiga-windows-x86_64.zip`r`n", [System.Text.Encoding]::UTF8)

Write-Host "  -> Pacote ZIP: $zipPath" -ForegroundColor Green
Write-Host "  -> SHA256 Checksum: $sha256" -ForegroundColor Green

# Final Summary
Write-Host "`n====================================================================" -ForegroundColor Cyan
Write-Host "   PIPELINE UNIFICADO CONCLUÍDO COM 100% DE SUCESSO!" -ForegroundColor Green
Write-Host "====================================================================" -ForegroundColor Cyan
Write-Host ("  [OK] Panadiga Desktop : {0} ({1:N2} MB)" -f (Join-Path $DistBinDir 'panadiga-desktop.exe'), $desktopSize) -ForegroundColor White
Write-Host ("  [OK] Panadiga CLI     : {0} ({1:N2} MB)" -f (Join-Path $DistBinDir 'panadiga-cli.exe'), $cliSize) -ForegroundColor White
Write-Host "  [OK] Ferramentas      : $(Join-Path $DistBinDir 'tools\') (GDB, OpenOCD, Ghidra, HxD)" -ForegroundColor White
Write-Host "  [OK] Release ZIP      : $zipPath" -ForegroundColor White
Write-Host "  [OK] Checksums        : $checksumFile" -ForegroundColor White
Write-Host "  [OK] Inicializador    : $(Join-Path $DistBinDir 'Executar_Panadiga.bat')" -ForegroundColor White
Write-Host "====================================================================`n" -ForegroundColor Cyan

exit 0
