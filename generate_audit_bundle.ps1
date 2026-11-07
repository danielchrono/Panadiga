<#
.SYNOPSIS
    Generates a full-text, tool-free Codebase Audit Dossier for Argon & Opus 5.5.
#>

$OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ScriptRoot = if ($PSScriptRoot) { (Get-Item $PSScriptRoot).FullName } else { (Get-Item .).FullName }
Set-Location $ScriptRoot

Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host ">> Generating Self-Contained Audit Dossier for Argon & Opus 5.5" -ForegroundColor Cyan
Write-Host "======================================================================" -ForegroundColor Cyan

python "$ScriptRoot/tools/generate_audit_bundle.py" $args

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "[SUCCESS] Dossier generated at: $ScriptRoot/AUDIT_DOSSIER_FOR_ARGON_AND_OPUS.md" -ForegroundColor Green
    Write-Host "You can now copy-paste or upload this file directly to Argon or Opus 5.5." -ForegroundColor Green
} else {
    Write-Host "[FAIL] Bundle generation failed." -ForegroundColor Red
}
