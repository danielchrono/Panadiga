#!/usr/bin/env sh
set -eu

# ==============================================================================
# Panadiga Universal POSIX Launcher (Linux / macOS / BSD / WSL / Git Bash)
# Pure POSIX sh (strictly LF, no bashisms) with recursive root discovery, TUI,
# parallel/serial DAG build orchestration, CI gates & compiler/hash cache setup.
# ==============================================================================

resolve_script_dir() {
    src="$1"
    while [ -L "$src" ]; do
        dir=$(CDPATH= cd -- "$(dirname -- "$src")" && pwd -P)
        if command -v readlink >/dev/null 2>&1; then
            link=$(readlink "$src")
        else
            break
        fi
        case "$link" in
            /*) src="$link" ;;
            *) src="$dir/$link" ;;
        esac
    done
    CDPATH= cd -- "$(dirname -- "$src")" && pwd -P
}

locate_repo_root() {
    probe="$1"
    while [ -n "$probe" ]; do
        if [ -f "$probe/Cargo.toml" ] && [ -f "$probe/scripts/build.py" ]; then
            printf '%s\n' "$probe"
            return 0
        fi
        parent=$(dirname -- "$probe")
        if [ "$parent" = "$probe" ]; then
            return 1
        fi
        probe="$parent"
    done
    return 1
}

detect_cpu_count() {
    if command -v nproc >/dev/null 2>&1; then
        nproc 2>/dev/null && return 0
    fi
    if command -v sysctl >/dev/null 2>&1; then
        sysctl -n hw.ncpu 2>/dev/null && return 0
    fi
    if command -v getconf >/dev/null 2>&1; then
        getconf _NPROCESSORS_ONLN 2>/dev/null && return 0
    fi
    printf '4\n'
}

check_python_bin() {
    cand="$1"
    if [ -x "$cand" ] || command -v "$cand" >/dev/null 2>&1; then
        if "$cand" -c "import sys; raise SystemExit(0 if sys.version_info >= (3, 9) else 1)" >/dev/null 2>&1; then
            printf '%s\n' "$cand"
            return 0
        fi
    fi
    return 1
}

find_python() {
    root_dir="$1"
    if [ -n "${PANADIGA_PYTHON:-}" ] && check_python_bin "$PANADIGA_PYTHON"; then
        return 0
    fi
    for venv_py in \
        "$root_dir/.venv/bin/python" \
        "$root_dir/.venv/Scripts/python.exe" \
        "$root_dir/venv/bin/python" \
        "$root_dir/venv/Scripts/python.exe"
    do
        if check_python_bin "$venv_py"; then
            return 0
        fi
    done
    for sys_py in python3 python; do
        if check_python_bin "$sys_py"; then
            return 0
        fi
    done
    if command -v py >/dev/null 2>&1 && py -3 -c "import sys; raise SystemExit(0 if sys.version_info >= (3, 9) else 1)" >/dev/null 2>&1; then
        printf 'py3\n'
        return 0
    fi
    return 1
}

SCRIPT_DIR=$(resolve_script_dir "$0")
PANADIGA_ROOT=""
if root_cand=$(locate_repo_root "$PWD"); then
    PANADIGA_ROOT="$root_cand"
elif root_cand=$(locate_repo_root "$SCRIPT_DIR"); then
    PANADIGA_ROOT="$root_cand"
else
    printf '[FAIL] Nao foi possivel localizar a raiz Panadiga (Cargo.toml + scripts/build.py).\n' >&2
    exit 1
fi

if [ -d "$HOME/.cargo/bin" ]; then
    case ":${PATH:-}:" in
        *":$HOME/.cargo/bin:"*) ;;
        *) PATH="$HOME/.cargo/bin:${PATH:-}" ;;
    esac
    export PATH
fi

CPU_JOBS=$(detect_cpu_count)
export PYTHONUTF8="${PYTHONUTF8:-1}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-$CPU_JOBS}"
export CMAKE_BUILD_PARALLEL_LEVEL="${CMAKE_BUILD_PARALLEL_LEVEL:-$CPU_JOBS}"
export MAKEFLAGS="${MAKEFLAGS:--j$CPU_JOBS}"

DISABLE_CACHE=0
for arg in "$@"; do
    if [ "$arg" = "--no-cache" ]; then
        DISABLE_CACHE=1
        break
    fi
done

if [ "$DISABLE_CACHE" -eq 0 ]; then
    if [ -z "${RUSTC_WRAPPER:-}" ] && command -v sccache >/dev/null 2>&1; then
        export RUSTC_WRAPPER
        RUSTC_WRAPPER=$(command -v sccache)
    fi
    if [ -z "${PANADIGA_CCACHE:-}" ] && command -v ccache >/dev/null 2>&1; then
        export PANADIGA_CCACHE
        PANADIGA_CCACHE=$(command -v ccache)
        export CCACHE_COMPILERCHECK="${CCACHE_COMPILERCHECK:-content}"
    fi
fi

if ! PY_SEL=$(find_python "$PANADIGA_ROOT"); then
    printf '[FAIL] Python 3.9+ nao encontrado. Instale python3 ou defina PANADIGA_PYTHON.\n' >&2
    exit 1
fi

run_build_py() {
    cd "$PANADIGA_ROOT"
    if [ "$PY_SEL" = "py3" ]; then
        exec py -3 "$PANADIGA_ROOT/scripts/build.py" "$@"
    else
        exec "$PY_SEL" "$PANADIGA_ROOT/scripts/build.py" "$@"
    fi
}

if [ "$#" -eq 0 ]; then
    if [ -t 0 ] && [ -t 1 ]; then
        run_build_py --tui
    else
        run_build_py --help
    fi
fi

case "$1" in
    tui)         shift; run_build_py --tui "$@" ;;
    all)         shift; run_build_py --all "$@" ;;
    all-ci)      shift; run_build_py --all --ci "$@" ;;
    ci)          shift; run_build_py --ci "$@" ;;
    desktop)     shift; run_build_py --desktop "$@" ;;
    frontend)    shift; run_build_py --frontend "$@" ;;
    pico)        shift; run_build_py --pico "$@" ;;
    android)     shift; run_build_py --android "$@" ;;
    run)         shift; run_build_py --run "$@" ;;
    rebuild)     shift; run_build_py --rebuild-run "$@" ;;
    cache)       shift; run_build_py --cache-status "$@" ;;
    clean-cache) shift; run_build_py --clean-cache "$@" ;;
    clean-all)   shift; run_build_py --clean-all-caches "$@" ;;
    help)        shift; run_build_py --help "$@" ;;
    cli)         shift; run_build_py --cli "$@" ;;
    *)           run_build_py "$@" ;;
esac
