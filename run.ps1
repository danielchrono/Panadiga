# PowerShell Fast Launcher for Panadiga
# Mirrors the behavior of run.cmd

[CmdletBinding()]
param(
    [switch]$Dev,
    [switch]$Build,
    [switch]$Cli,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$RemainingArgs
)

$ErrorActionPreference = "Stop"
$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$RootPath = if ($PSScriptRoot) { (Get-Item $PSScriptRoot).FullName } else { (Get-Item .).FullName }
Set-Location $RootPath

$DesktopExe = Join-Path $RootPath "dist\\panadiga-desktop.exe"
$CliExe = Join-Path $RootPath "dist\\panadiga-cli.exe"

# 1. Dev mode
if ($Dev) {
    Write-Host "[INFO] Starting Desktop development (Vite + Tauri Dev)..." -ForegroundColor Cyan
    Set-Location (Join-Path $RootPath "apps\\desktop")
    npm run tauri dev
    exit $LASTEXITCODE
}

# 2. Build mode
if ($Build) {
    Write-Host "[INFO] Performing full rebuild of the ecosystem..." -ForegroundColor Cyan
    & "$RootPath\build_all.ps1"
    if (Test-Path $DesktopExe) {
        Write-Host "[INFO] Launching Panadiga Desktop..." -ForegroundColor Green
        Start-Process -FilePath $DesktopExe
    }
    exit 0
}

# 3. CLI mode or shortcut subcommands
$cliSubcommands = @("nvram","hdd","frm","headers","scan","manuals","pico","ghidra","rag","help")
$firstArg = if ($RemainingArgs.Count -gt 0) { $RemainingArgs[0] } else { "" }

if ($Cli -or ($cliSubcommands -contains $firstArg)) {
    if (Test-Path $CliExe) {
        & $CliExe @RemainingArgs
        exit $LASTEXITCODE
    } else {
        cargo run -- @RemainingArgs
        exit $LASTEXITCODE
    }
}

# 4. Default: launch Desktop GUI
if (Test-Path $DesktopExe) {
    Write-Host "[INFO] Starting Panadiga Desktop instantly..." -ForegroundColor Green
    Start-Process -FilePath $DesktopExe
    exit 0
} else {
    Write-Host "[WARN] Desktop binary not found in dist\. Triggering build..." -ForegroundColor Yellow
    & "$RootPath\build_all.ps1"
    if (Test-Path $DesktopExe) {
        Start-Process -FilePath $DesktopExe
        exit 0
    } else {
        Write-Error "Failed to locate executable after build!"
        exit 1
    }
}
