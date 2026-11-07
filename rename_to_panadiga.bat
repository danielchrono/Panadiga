@echo off
@chcp 65001 >nul
echo ====================================================================
echo  Renomeador de Pasta Raiz para Panadiga
echo ====================================================================
echo.
echo Certifique-se de que o Antigravity IDE ou VS Code esteja FECHADO
echo para que o Windows libere os arquivos.
echo.
pause

cd ..
if exist "Panadiga" (
    rmdir "Panadiga" 2>nul
)

rename "Panasonic Firmware HDD bin" "Panadiga"
if %ERRORLEVEL% equ 0 (
    echo.
    echo [SUCESSO] A pasta foi renomeada para: C:\Users\danie\Documents\Panadiga
    echo Agora voce pode reabrir seu editor diretamente na pasta Panadiga!
) else (
    echo.
    echo [AVISO] Ainda ha programas usando a pasta. Feche seu editor e tente novamente.
)
echo.
pause
