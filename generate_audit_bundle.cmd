@echo off
chcp 65001 >nul
title Panadiga Audit Dossier Generator for Independent Security Auditor

echo ======================================================================
echo ^>^> Generating Self-Contained Audit Dossier for Independent Auditor
echo ======================================================================

python "%~dp0tools\generate_audit_bundle.py" %*

if %ERRORLEVEL% equ 0 (
    echo.
    echo [SUCCESS] Dossier generated at: %~dp0AUDIT_DOSSIER_INDEPENDENT_REVIEW.md
    echo You can now copy-paste or upload this file directly to the auditor.
) else (
    echo [FAIL] Bundle generation failed.
)
pause
