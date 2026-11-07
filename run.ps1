<#
.SYNOPSIS
    Panadiga Engineering Suite - Fast Launcher & Runner.
.DESCRIPTION
    Quickly launches Panadiga Desktop UI, Dev Mode, or CLI commands with instant binary caching.
.PARAMETER Dev
    Starts Tauri Desktop application in live development mode with hot-reloading.
.PARAMETER Build
    Forces a complete rebuild using maximum parallelism before launching.
.PARAMETER Cli
    Runs the native CLI tool with any arguments passed.
#>

[CmdletBinding()]
param (
    [switch]$Dev,
    [switch]$Build,
    [switch]$Cli,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$RemainingArgs
)

$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "Stop"

$RootPath = if ($PSScriptRoot) { (Get-Item $PSScriptRoot).FullName } else { (Get-Item .).FullName }
Set-Location $RootPath

$DesktopExe = Join-Path $RootPath "dist_bin\panadiga-desktop.exe"
$CliExe = Join-Path $RootPath "dist_bin\panadiga-cli.exe"

# 1. Dev Mode
if ($Dev) {
    Write-Host "[INFO] Iniciando ambiente de desenvolvimento Desktop (Vite + Tauri Dev)..." -ForegroundColor Cyan
    Set-Location (Join-Path $RootPath "apps\desktop")
    npm run tauri dev
    exit $LASTEXITCODE
}

# 2. Build Mode
if ($Build) {
    Write-Host "[INFO] Executando rebuild completo com paralelismo acelerado..." -ForegroundColor Cyan
    & "$RootPath\build_all.ps1"
    if (Test-Path $DesktopExe) {
        Write-Host "[INFO] Iniciando Panadiga Desktop..." -ForegroundColor Green
        Start-Process -FilePath $DesktopExe
    }
    exit 0
}

# 3. CLI Mode or Subcommand Shortcut
$cliSubcommands = @("nvram", "hdd", "frm", "headers", "scan", "manuals", "pico", "ghidra", "rag", "help")
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

# 4. Default: Launch Desktop
if (Test-Path $DesktopExe) {
    Write-Host "[INFO] Iniciando Panadiga Desktop instantaneamente..." -ForegroundColor Green
    Start-Process -FilePath $DesktopExe
    exit 0
} else {
    Write-Host "[AVISO] Binário Desktop não encontrado em dist_bin\. Compilando agora..." -ForegroundColor Yellow
    & "$RootPath\build_all.ps1"
    if (Test-Path $DesktopExe) {
        Write-Host "[INFO] Iniciando Panadiga Desktop..." -ForegroundColor Green
        Start-Process -FilePath $DesktopExe
        exit 0
    } else {
        Write-Error "Falha ao localizar executável após o build!"
        exit 1
    }
}
