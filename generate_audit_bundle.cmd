@echo off
chcp 65001 >nul
title Panadiga Audit Dossier Generator for Argon and Opus 5.5

echo ======================================================================
echo ^>^> Generating Self-Contained Audit Dossier for Argon and Opus 5.5
echo ======================================================================

python "%~dp0tools\generate_audit_bundle.py" %*

if %ERRORLEVEL% equ 0 (
    echo.
    echo [SUCCESS] Dossier generated at: %~dp0AUDIT_DOSSIER_FOR_ARGON_AND_OPUS.md
    echo You can now copy-paste or upload this file directly to Argon or Opus 5.5.
) else (
    echo [FAIL] Bundle generation failed.
)
pause
