<#
.SYNOPSIS
    Multi-platform build script for Panasonic DWire32A firmware (RP2040 & RP2350).

.DESCRIPTION
    Configures and compiles the panasonic_dwire32a firmware using CMake and
    the Raspberry Pi Pico SDK. Generates separate panasonic_dwire32a_rp2040.uf2
    and panasonic_dwire32a_rp2350.uf2 binaries ready for drag-and-drop flashing.

.PARAMETER Clean
    If specified, removes the build directory before compiling.

.PARAMETER SdkPath
    Optional explicit path to the Raspberry Pi Pico SDK.

.PARAMETER Board
    Target board architecture: 'pico' for RP2040, 'pico2' for RP2350, or 'all' to build both (default).

.EXAMPLE
    .\build_pico_firmware.ps1
    .\build_pico_firmware.ps1 -Board pico
    .\build_pico_firmware.ps1 -Board pico2
    .\build_pico_firmware.ps1 -Clean -Board all
#>

[CmdletBinding()]
param (
    [switch]$Clean,
    [string]$SdkPath = "",
    [ValidateSet("pico", "pico2", "all")]
    [string]$Board = "all"
)

$ErrorActionPreference = "Stop"
$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "====================================================================" -ForegroundColor Cyan
Write-Host "   PANASONIC DWIRE32A / OCD - MULTI-PLATFORM BUILD SYSTEM" -ForegroundColor Cyan
Write-Host "   Architectures Supported: RP2040 (Pico 1) & RP2350 (Pico 2)" -ForegroundColor Cyan
Write-Host "   Selected Target: $Board" -ForegroundColor Cyan
Write-Host "====================================================================" -ForegroundColor Cyan
Write-Host ""

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

# 1. Resolve Pico SDK Path
if ($SdkPath -ne "") {
    $env:PICO_SDK_PATH = $SdkPath
}

if (-not $env:PICO_SDK_PATH) {
    Write-Host "[INFO] PICO_SDK_PATH not set. Searching default locations..." -ForegroundColor Yellow
    $candidatePaths = @(
        "C:\Program Files\Raspberry Pi\Pico SDK v2.0.0\pico-sdk",
        "C:\Program Files\Raspberry Pi\Pico SDK v1.5.1\pico-sdk",
        "C:\pico\pico-sdk",
        "C:\pico-sdk",
        "$env:USERPROFILE\pico-sdk",
        "$env:USERPROFILE\.pico-sdk\sdk\2.0.0",
        "$env:USERPROFILE\.pico-sdk\sdk\1.5.1",
        (Join-Path $ScriptDir "..\pico-sdk")
    )
    foreach ($cand in $candidatePaths) {
        if (Test-Path $cand) {
            $env:PICO_SDK_PATH = (Resolve-Path $cand).Path
            break
        }
    }
}

if ($env:PICO_SDK_PATH) {
    Write-Host "[OK] Using Pico SDK at: $env:PICO_SDK_PATH" -ForegroundColor Green
} else {
    Write-Host "[WARNING] Pico SDK path not detected. CMake will attempt automatic Git fetch if online." -ForegroundColor Yellow
}

# 2. Check CMake
$cmake = Get-Command "cmake" -ErrorAction SilentlyContinue
if (-not $cmake) {
    Write-Error "CMake was not found on PATH. Please install CMake and add it to PATH."
}
Write-Host "[OK] Detected CMake: $($cmake.Source)" -ForegroundColor Green

# 3. Check ARM GCC
$armGcc = Get-Command "arm-none-eabi-gcc" -ErrorAction SilentlyContinue
if ($armGcc) {
    Write-Host "[OK] Detected ARM GCC: $($armGcc.Source)" -ForegroundColor Green
} else {
    Write-Host "[WARNING] arm-none-eabi-gcc not found on PATH. Make sure the ARM embedded toolchain is installed." -ForegroundColor Yellow
}

# 4. Detect CMake Generator
$generator = "Ninja"
if (-not (Get-Command "ninja" -ErrorAction SilentlyContinue)) {
    if (Get-Command "mingw32-make" -ErrorAction SilentlyContinue) {
        $generator = "MinGW Makefiles"
    } elseif (Get-Command "nmake" -ErrorAction SilentlyContinue) {
        $generator = "NMake Makefiles"
    }
}
Write-Host "[OK] Using CMake Generator: $generator" -ForegroundColor Green

# 5. Helper function for building a target
function Build-TargetBoard([string]$targetBoard) {
    $targetBuildDir = Join-Path $ScriptDir "build_$targetBoard"
    $outUf2Name = if ($targetBoard -eq "pico") { "panasonic_dwire32a_rp2040.uf2" } else { "panasonic_dwire32a_rp2350.uf2" }

    if ($Clean -and (Test-Path $targetBuildDir)) {
        Write-Host "[INFO] Cleaning previous build directory: $targetBuildDir..." -ForegroundColor Yellow
        Remove-Item -Recurse -Force $targetBuildDir
    }

    if (-not (Test-Path $targetBuildDir)) {
        New-Item -ItemType Directory -Path $targetBuildDir | Out-Null
    }

    Write-Host "`n>>> [1/2] Configuring CMake for Board $targetBoard in $targetBuildDir..." -ForegroundColor Cyan
    & cmake -B $targetBuildDir -G $generator -DCMAKE_BUILD_TYPE=Release "-DPICO_BOARD=$targetBoard"
    if ($LASTEXITCODE -ne 0) {
        Write-Error "CMake configuration failed for board $targetBoard."
    }

    Write-Host "`n>>> [2/2] Building firmware for $targetBoard..." -ForegroundColor Cyan
    & cmake --build $targetBuildDir --config Release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Compilation failed for board $targetBoard."
    }

    $rawUf2 = Join-Path $targetBuildDir "panasonic_dwire32a.uf2"
    $destUf2 = Join-Path $ScriptDir $outUf2Name
    if (Test-Path $rawUf2) {
        Copy-Item -Path $rawUf2 -Destination $destUf2 -Force
        $item = Get-Item $destUf2
        $hash = Get-FileHash -Path $destUf2 -Algorithm SHA256
        Write-Host "`n[SUCCESS] COMPILED: $outUf2Name" -ForegroundColor Green
        Write-Host "  Path    : $($item.FullName)" -ForegroundColor White
        Write-Host "  Size    : $($item.Length) bytes ($([math]::Round($item.Length / 1KB, 2)) KB)" -ForegroundColor White
        Write-Host "  SHA-256 : $($hash.Hash)" -ForegroundColor White
    }
}

# 6. Execute Build
$boardsToBuild = if ($Board -eq "all") { @("pico", "pico2") } else { @($Board) }
foreach ($b in $boardsToBuild) {
    Build-TargetBoard -targetBoard $b
}

Write-Host "`n====================================================================" -ForegroundColor Cyan
Write-Host "   MULTI-PLATFORM FIRMWARE BUILD COMPLETE!" -ForegroundColor Green
Write-Host "   Flash with drag-and-drop to RPI-RP2 drive." -ForegroundColor White
Write-Host "====================================================================" -ForegroundColor Cyan
