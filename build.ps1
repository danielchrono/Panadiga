# Panadiga Unified Engineering Suite - Master Build Script (PowerShell)
# Usage:
#   .\build.ps1              # Full build (Desktop + Android if SDK detected)
#   .\build.ps1 -Desktop     # Desktop unified release build only
#   .\build.ps1 -Android     # Android ARM64 APK build only
#   .\build.ps1 -All         # Force Desktop + Android APK release

[CmdletBinding()]
param(
    [switch]$Android,
    [switch]$Desktop,
    [switch]$All
)

$ErrorActionPreference = "Stop"
$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# Add cargo and local bins to path
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:USERPROFILE\.local\bin;$env:Path"

Write-Host "====================================================================" -ForegroundColor Cyan
Write-Host "   PANADIGA ENGINEERING SUITE: MASTER BUILD PIPELINE (PowerShell)" -ForegroundColor Cyan
Write-Host "   Configuração: Paralelismo ($($env:NUMBER_OF_PROCESSORS) Cores) + UTF-8" -ForegroundColor Cyan
Write-Host "====================================================================" -ForegroundColor Cyan

# 1. Build Desktop
function Build-Desktop {
    Write-Host "`n[1/4] Compilando Frontend Desktop (React / TypeScript / Vite)..." -ForegroundColor Yellow
    if (-not (Test-Path "apps\desktop\node_modules")) {
        Write-Host "[*] Instalando dependencias do frontend..." -ForegroundColor Gray
        npm --prefix apps\desktop ci
    }
    npm --prefix apps\desktop run build
    Write-Host "[OK] Bundle otimizado gerado em apps\desktop\dist\" -ForegroundColor Green

    Write-Host "`n[2/4] Exportando scripts Ghidra diretamente do Rust SSOT..." -ForegroundColor Yellow
    cargo run --release -p panadiga -- ghidra export-all --dir "tools" 2>$null
    Write-Host "[OK] Scripts Ghidra sincronizados em tools\" -ForegroundColor Green

    Write-Host "`n[3/4] Compilando Binário Unificado Panadiga em modo Release..." -ForegroundColor Yellow
    cargo build --release -p panadiga
    Write-Host "[OK] Binário unificado gerado: target\release\panadiga.exe" -ForegroundColor Green

    Write-Host "`n[4/4] Empacotando distribuição para dist\..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Force -Path "dist", "dist\tools", "dist\releases" | Out-Null
    Copy-Item -Force "target\release\panadiga.exe" "dist\panadiga.exe"

    Copy-Item -Recurse -Force "tools\*" "dist\tools\"
    if (Test-Path "src\pico\firmware_bins\panasonic_dwire32a_rp2040.uf2") {
        Copy-Item -Force "src\pico\firmware_bins\panasonic_dwire32a_rp2040.uf2" "dist\"
    }
    if (Test-Path "src\pico\firmware_bins\panasonic_dwire32a_rp2350.uf2") {
        Copy-Item -Force "src\pico\firmware_bins\panasonic_dwire32a_rp2350.uf2" "dist\"
    }
    # Clean redundant runners and deprecated files from dist
    Remove-Item "dist\run.cmd", "dist\run.ps1", "dist\run.sh", "dist\WebView2Loader.dll", "dist\firmware" -Recurse -Force -ErrorAction SilentlyContinue

    # Create ZIP archive (guard missing files)
    $zipPath = "dist\releases\panadiga-windows-x86_64.zip"
    if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
    $items = @(
        "dist\panadiga.exe",
        "dist\panasonic_dwire32a_rp2040.uf2",
        "dist\panasonic_dwire32a_rp2350.uf2",
        "dist\LEIA-ME.txt",
        "dist\tools",
        "README.md",
        "README.pt-BR.md",
        "LICENSE"
    ) | Where-Object { Test-Path $_ }
    if ($items.Count -gt 0) {
        Compress-Archive -Path $items -DestinationPath $zipPath -CompressionLevel Optimal
    } else {
        Write-Host "[WARN] No files to archive for Windows distribution." -ForegroundColor Yellow
    }
    $hash = (Get-FileHash -Algorithm SHA256 $zipPath).Hash
    [IO.File]::WriteAllText("$scriptDir\dist\releases\SHA256SUMS.txt", "$hash  panadiga-windows-x86_64.zip`r`n", [Text.Encoding]::UTF8)

    Write-Host "[OK] Distribuição Desktop gerada com sucesso em dist\releases\" -ForegroundColor Green
}

# 2. Build Android APK
function Build-Android {
    Write-Host "`n====================================================================" -ForegroundColor Magenta
    Write-Host "   Panadiga Android APK Build Pipeline (AArch64 ARM64)" -ForegroundColor Magenta
    Write-Host "====================================================================" -ForegroundColor Magenta

    $androidSdk = $env:ANDROID_HOME
    if (-not $androidSdk) {
        $candidate = "$env:LOCALAPPDATA\Android\Sdk"
        if (Test-Path $candidate) {
            $androidSdk = $candidate
            $env:ANDROID_HOME = $candidate
        } else {
            Write-Error "ANDROID_HOME não definido e SDK padrão não encontrado em $candidate"
            return
        }
    }
    Write-Host "[*] Android SDK: $androidSdk" -ForegroundColor Gray

    $ndkCandidate = Get-ChildItem -Path "$androidSdk\ndk" -Directory -ErrorAction SilentlyContinue | Select-Object -Last 1
    if ($ndkCandidate) {
        $env:NDK_HOME = $ndkCandidate.FullName
        $env:ANDROID_NDK_ROOT = $ndkCandidate.FullName
        Write-Host "[*] Android NDK: $($ndkCandidate.FullName)" -ForegroundColor Gray
    }

    Write-Host "[*] Verificando target aarch64-linux-android..." -ForegroundColor Gray
    rustup target add aarch64-linux-android

    Write-Host "[*] Compilando frontend..." -ForegroundColor Gray
    npm --prefix apps\desktop run build

    Write-Host "[*] Configurando crate-type cdylib no Cargo.toml..." -ForegroundColor Gray
    $cargoToml = Get-Content Cargo.toml -Raw
    $modCargoToml = $cargoToml -replace 'crate-type = \["rlib"\]', 'crate-type = ["staticlib", "cdylib", "rlib"]'
    Set-Content -Path Cargo.toml -Value $modCargoToml -NoNewline

    try {
        if (-not (Test-Path "gen\android") -and -not (Test-Path "src-tauri\gen\android")) {
            Write-Host "[*] Inicializando módulo Android no Tauri..." -ForegroundColor Gray
            npx @tauri-apps/cli android init --ci --skip-targets-install
        }

        Write-Host "[*] Compilando APK via Tauri CLI..." -ForegroundColor Yellow
        npx @tauri-apps/cli android build --apk --target aarch64

        # Find and copy generated APK to dist\ and dist\releases\
        $apkFiles = Get-ChildItem -Path "gen\android\app\build\outputs\apk" -Filter "*.apk" -Recurse -ErrorAction SilentlyContinue
        if ($apkFiles) {
            $apk = $apkFiles | Select-Object -First 1
            $destName = "panadiga-android-arm64.apk"
            New-Item -ItemType Directory -Force -Path "dist", "dist\releases" | Out-Null
            Copy-Item -Force $apk.FullName "dist\$destName"
            Copy-Item -Force $apk.FullName "dist\releases\$destName"
            $apkHash = (Get-FileHash -Algorithm SHA256 "dist\$destName").Hash
            $shaFile = "$scriptDir\dist\releases\SHA256SUMS.txt"
            $lines = if (Test-Path $shaFile) { (Get-Content $shaFile) | Where-Object { $_ -notmatch 'panadiga-android-arm64\.apk' } } else { @() }
            $lines += "$apkHash  $destName"
            [IO.File]::WriteAllLines($shaFile, $lines, [System.Text.Encoding]::UTF8)
            Write-Host "[OK] APK copiado para dist\$destName e dist\releases\$destName" -ForegroundColor Green
        }
    } finally {
        Write-Host "[*] Restaurando Cargo.toml para crate-type = ['rlib']..." -ForegroundColor Gray
        Set-Content -Path Cargo.toml -Value $cargoToml -NoNewline
    }
}

$hasAndroidSdk = $false
if ($env:ANDROID_HOME -and (Test-Path $env:ANDROID_HOME)) {
    $hasAndroidSdk = $true
} elseif (Test-Path "$env:LOCALAPPDATA\Android\Sdk") {
    $hasAndroidSdk = $true
}

if ($Android) {
    Build-Android
} elseif ($Desktop) {
    Build-Desktop
} elseif ($All -or $hasAndroidSdk) {
    Build-Desktop
    Build-Android
} else {
    Build-Desktop
}

Write-Host "`n[OK] Pipeline concluída!" -ForegroundColor Green
