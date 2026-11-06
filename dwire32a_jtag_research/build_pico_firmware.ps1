<#
.SYNOPSIS
    Build script for Panasonic DWire32A Raspberry Pi Pico (RP2040) firmware.

.DESCRIPTION
    Configures and compiles the panasonic_dwire32a firmware using CMake and
    the Raspberry Pi Pico SDK. Generates the final panasonic_dwire32a.uf2
    ready for drag-and-drop flashing.

.PARAMETER Clean
    If specified, removes the build directory before compiling.

.PARAMETER SdkPath
    Optional explicit path to the Raspberry Pi Pico SDK.

.EXAMPLE
    .\build_pico_firmware.ps1
    .\build_pico_firmware.ps1 -Clean
    .\build_pico_firmware.ps1 -SdkPath "C:\pico-sdk"
#>

[CmdletBinding()]
param (
    [switch]$Clean,
    [string]$SdkPath = ""
)

$ErrorActionPreference = "Stop"

Write-Host "====================================================================" -ForegroundColor Cyan
Write-Host "   PANASONIC DWIRE32A / OCD - RASPBERRY PI PICO BUILD SYSTEM" -ForegroundColor Cyan
Write-Host "   Firmware Target: panasonic_dwire32a.uf2 (RP2040)" -ForegroundColor Cyan
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
        "C:\Program Files\Raspberry Pi\Pico SDK v1.5.1\pico-sdk",
        "C:\Program Files\Raspberry Pi\Pico SDK v1.5.0\pico-sdk",
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

# 5. Handle Clean Flag
$BuildDir = Join-Path $ScriptDir "build"
if ($Clean -and (Test-Path $BuildDir)) {
    Write-Host "[INFO] Cleaning previous build directory..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $BuildDir
}

if (-not (Test-Path $BuildDir)) {
    New-Item -ItemType Directory -Path $BuildDir | Out-Null
}

# 6. Configure Project
Write-Host "`n[1/2] Configuring project with CMake..." -ForegroundColor Cyan
& cmake -B build -G $generator -DCMAKE_BUILD_TYPE=Release
if ($LASTEXITCODE -ne 0) {
    Write-Error "CMake configuration failed. Review errors above."
}

# 7. Build Target
Write-Host "`n[2/2] Building panasonic_dwire32a target..." -ForegroundColor Cyan
& cmake --build build --config Release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Compilation failed. Review compiler output above."
}

# 8. Check Artifact
$Uf2File = Join-Path $BuildDir "panasonic_dwire32a.uf2"
Write-Host "`n====================================================================" -ForegroundColor Cyan
if (Test-Path $Uf2File) {
    $item = Get-Item $Uf2File
    $hash = Get-FileHash -Path $Uf2File -Algorithm SHA256
    Write-Host "[SUCCESS] FIRMWARE COMPILED SUCCESSFULLY!" -ForegroundColor Green
    Write-Host "Binary Path : $($item.FullName)" -ForegroundColor White
    Write-Host "Binary Size : $($item.Length) bytes ($([math]::Round($item.Length / 1KB, 2)) KB)" -ForegroundColor White
    Write-Host "SHA-256     : $($hash.Hash)" -ForegroundColor White
    Write-Host ""
    Write-Host "FLASHING PROCEDURE (DRAG & DROP):" -ForegroundColor Yellow
    Write-Host "  1. Disconnect the Raspberry Pi Pico from the USB cable."
    Write-Host "  2. Hold down the white BOOTSEL button on the Pico board."
    Write-Host "  3. While holding BOOTSEL, connect the USB cable to your PC."
    Write-Host "  4. Release the BOOTSEL button."
    Write-Host "  5. A mass storage volume labeled 'RPI-RP2' will appear in File Explorer."
    Write-Host "  6. Drag and drop 'panasonic_dwire32a.uf2' onto the 'RPI-RP2' drive."
    Write-Host "  7. The Pico will flash immediately and reboot as a USB CDC Serial device."
    Write-Host "  8. Open your serial console (e.g., COM3) or the Tauri application!"
} else {
    Write-Host "[WARNING] Build completed but 'panasonic_dwire32a.uf2' was not found in 'build\'." -ForegroundColor Yellow
}
Write-Host "====================================================================" -ForegroundColor Cyan
