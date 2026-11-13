@echo off
@chcp 65001 >nul
setlocal EnableExtensions EnableDelayedExpansion

rem ============================================================================
rem Panadiga Universal Windows Launcher (Recursive Root + TUI + CI + Cache)
rem ============================================================================

set "SCRIPT_DIR=%~dp0"
if "%SCRIPT_DIR:~-1%"=="\" if not "%SCRIPT_DIR:~1,2%"==":\" set "SCRIPT_DIR=%SCRIPT_DIR:~0,-1%"

set "PANADIGA_ROOT="
call :locate_root "%CD%"
if not defined PANADIGA_ROOT call :locate_root "%SCRIPT_DIR%"
if not defined PANADIGA_ROOT (
    echo [FAIL] Nao foi possivel localizar a raiz do repositorio Panadiga ^(Cargo.toml + scripts\build.py^).
    exit /b 1
)

rem Ensure Cargo bin directory, parallel job defaults, and UTF-8 stdio are active
if exist "%USERPROFILE%\.cargo\bin" (
    echo ";%PATH%;" | findstr /I /C:";%USERPROFILE%\.cargo\bin;" >nul || set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)
set "PYTHONUTF8=1"
if not defined NUMBER_OF_PROCESSORS set "NUMBER_OF_PROCESSORS=4"
if not defined CARGO_BUILD_JOBS set "CARGO_BUILD_JOBS=%NUMBER_OF_PROCESSORS%"
if not defined CMAKE_BUILD_PARALLEL_LEVEL set "CMAKE_BUILD_PARALLEL_LEVEL=%NUMBER_OF_PROCESSORS%"

rem Detect compiler cache accelerators (sccache / ccache) unless --no-cache was requested
set "DISABLE_CACHE=0"
for %%A in (%*) do (
    if /I "%%~A"=="--no-cache" set "DISABLE_CACHE=1"
)
if "!DISABLE_CACHE!"=="0" (
    if not defined RUSTC_WRAPPER (
        for /f "delims=" %%S in ('where sccache 2^>nul') do (
            if not defined RUSTC_WRAPPER set "RUSTC_WRAPPER=%%~fS"
        )
    )
    if not defined PANADIGA_CCACHE (
        for /f "delims=" %%C in ('where ccache 2^>nul') do (
            if not defined PANADIGA_CCACHE (
                set "PANADIGA_CCACHE=%%~fC"
                if not defined CCACHE_COMPILERCHECK set "CCACHE_COMPILERCHECK=content"
            )
        )
    )
)

set "PY_EXE="
set "PY_PREFIX="
call :find_python
if not defined PY_EXE (
    echo [FAIL] Python 3.9+ nao encontrado neste sistema.
    echo        Instale com: winget install Python.Python.3.12
    echo        Ou defina PANADIGA_PYTHON apontando para python.exe.
    exit /b 1
)

pushd "%PANADIGA_ROOT%" >nul
set "EXIT_CODE=0"
set "OPENED_TUI=0"

if "%~1"=="" (
    set "OPENED_TUI=1"
    "%PY_EXE%" %PY_PREFIX% "%PANADIGA_ROOT%\scripts\build.py" --tui
    set "EXIT_CODE=!ERRORLEVEL!"
    goto :finish
)

set "FIRST_ARG=%~1"
set "MAPPED_FLAG="
if /I "!FIRST_ARG!"=="tui" (
    set "MAPPED_FLAG=--tui"
    set "OPENED_TUI=1"
)
if /I "!FIRST_ARG!"=="all" set "MAPPED_FLAG=--all"
if /I "!FIRST_ARG!"=="all-ci" set "MAPPED_FLAG=--all --ci"
if /I "!FIRST_ARG!"=="ci" set "MAPPED_FLAG=--ci"
if /I "!FIRST_ARG!"=="desktop" set "MAPPED_FLAG=--desktop"
if /I "!FIRST_ARG!"=="frontend" set "MAPPED_FLAG=--frontend"
if /I "!FIRST_ARG!"=="pico" set "MAPPED_FLAG=--pico"
if /I "!FIRST_ARG!"=="android" set "MAPPED_FLAG=--android"
if /I "!FIRST_ARG!"=="run" set "MAPPED_FLAG=--run"
if /I "!FIRST_ARG!"=="rebuild" set "MAPPED_FLAG=--rebuild-run"
if /I "!FIRST_ARG!"=="cache" set "MAPPED_FLAG=--cache-status"
if /I "!FIRST_ARG!"=="clean-cache" set "MAPPED_FLAG=--clean-cache"
if /I "!FIRST_ARG!"=="clean-all" set "MAPPED_FLAG=--clean-all-caches"
if /I "!FIRST_ARG!"=="help" set "MAPPED_FLAG=--help"
if /I "!FIRST_ARG!"=="cli" set "MAPPED_FLAG=--cli"

if defined MAPPED_FLAG (
    call :gather_rest %*
    "%PY_EXE%" %PY_PREFIX% "%PANADIGA_ROOT%\scripts\build.py" !MAPPED_FLAG! !REST_ARGS!
    set "EXIT_CODE=!ERRORLEVEL!"
) else (
    "%PY_EXE%" %PY_PREFIX% "%PANADIGA_ROOT%\scripts\build.py" %*
    set "EXIT_CODE=!ERRORLEVEL!"
)

:finish
popd >nul
if not "!EXIT_CODE!"=="0" if "!OPENED_TUI!"=="1" pause
exit /b %EXIT_CODE%

rem ----------------------------------------------------------------------------
:locate_root
set "_PROBE=%~1"
if not defined _PROBE exit /b 1
if "%_PROBE:~-1%"=="\" if not "%_PROBE:~1,2%"==":\" set "_PROBE=%_PROBE:~0,-1%"
:_walk_up
if exist "%_PROBE%\Cargo.toml" if exist "%_PROBE%\scripts\build.py" (
    set "PANADIGA_ROOT=%_PROBE%"
    exit /b 0
)
for %%I in ("%_PROBE%\..") do set "_PARENT=%%~fI"
if /I "%_PARENT%"=="%_PROBE%" exit /b 1
set "_PROBE=%_PARENT%"
goto :_walk_up

rem ----------------------------------------------------------------------------
:find_python
if defined PANADIGA_PYTHON (
    call :check_python_cmd "%PANADIGA_PYTHON%" "" && exit /b 0
)
if exist "%PANADIGA_ROOT%\.venv\Scripts\python.exe" (
    call :check_python_cmd "%PANADIGA_ROOT%\.venv\Scripts\python.exe" "" && exit /b 0
)
if exist "%PANADIGA_ROOT%\venv\Scripts\python.exe" (
    call :check_python_cmd "%PANADIGA_ROOT%\venv\Scripts\python.exe" "" && exit /b 0
)
for /f "delims=" %%P in ('where python 2^>nul') do (
    call :check_python_cmd "%%~fP" "" && exit /b 0
)
for /f "delims=" %%P in ('where python3 2^>nul') do (
    call :check_python_cmd "%%~fP" "" && exit /b 0
)
for /f "delims=" %%P in ('where py 2^>nul') do (
    call :check_python_cmd "%%~fP" "-3" && exit /b 0
)
exit /b 1

:check_python_cmd
set "_CAND_EXE=%~1"
set "_CAND_ARG=%~2"
if not exist "%_CAND_EXE%" exit /b 1
"%_CAND_EXE%" %_CAND_ARG% -c "import sys; raise SystemExit(0 if sys.version_info >= (3, 9) else 1)" >nul 2>&1
if errorlevel 1 exit /b 1
set "PY_EXE=%_CAND_EXE%"
set "PY_PREFIX=%_CAND_ARG%"
exit /b 0

rem ----------------------------------------------------------------------------
:gather_rest
set "REST_ARGS="
shift
:_next_rest
if "%~1"=="" exit /b 0
set "REST_ARGS=!REST_ARGS! "%~1""
shift
goto :_next_rest
