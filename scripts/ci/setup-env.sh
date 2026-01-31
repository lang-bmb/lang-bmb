#!/bin/bash
# BMB CI Environment Setup Script
# Part of the Bootstrap + Benchmark Cycle System
#
# Sets up the required environment for CI/CD pipelines.
# Supports: Ubuntu, macOS, Windows (MSYS2)
#
# Usage:
#   source scripts/ci/setup-env.sh [options]
#
# Options:
#   --install-llvm      Install LLVM if not present
#   --install-deps      Install all dependencies
#   --verify            Verify environment is correct
#   --export            Export environment variables

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux" ;;
        Darwin*)    echo "macos" ;;
        CYGWIN*)    echo "windows" ;;
        MINGW*)     echo "windows" ;;
        MSYS*)      echo "windows" ;;
        *)          echo "unknown" ;;
    esac
}

OS=$(detect_os)
echo "Detected OS: $OS"

# Parse arguments
INSTALL_LLVM=false
INSTALL_DEPS=false
VERIFY=false
EXPORT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --install-llvm)
            INSTALL_LLVM=true
            shift
            ;;
        --install-deps)
            INSTALL_DEPS=true
            shift
            ;;
        --verify)
            VERIFY=true
            shift
            ;;
        --export)
            EXPORT=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# =============================================================================
# LLVM Installation
# =============================================================================
install_llvm_linux() {
    echo "Installing LLVM 21 on Linux..."

    if command -v apt-get &> /dev/null; then
        # Debian/Ubuntu
        wget https://apt.llvm.org/llvm.sh
        chmod +x llvm.sh
        sudo ./llvm.sh 21
        rm llvm.sh

        export LLVM_SYS_210_PREFIX=/usr/lib/llvm-21
        export PATH="/usr/lib/llvm-21/bin:$PATH"
    elif command -v dnf &> /dev/null; then
        # Fedora
        sudo dnf install -y llvm21 clang21 lld21
        export LLVM_SYS_210_PREFIX=/usr
    elif command -v pacman &> /dev/null; then
        # Arch
        sudo pacman -S --noconfirm llvm clang lld
        export LLVM_SYS_210_PREFIX=/usr
    else
        echo "Error: Unsupported Linux distribution"
        exit 1
    fi
}

install_llvm_macos() {
    echo "Installing LLVM 21 on macOS..."

    if command -v brew &> /dev/null; then
        brew install llvm@21
        export LLVM_SYS_210_PREFIX="$(brew --prefix llvm@21)"
        export PATH="$LLVM_SYS_210_PREFIX/bin:$PATH"
    else
        echo "Error: Homebrew not found. Install it first."
        exit 1
    fi
}

install_llvm_windows() {
    echo "Installing LLVM 21 on Windows (MSYS2)..."

    if command -v pacman &> /dev/null; then
        pacman -S --noconfirm mingw-w64-ucrt-x86_64-llvm mingw-w64-ucrt-x86_64-clang
        export LLVM_SYS_210_PREFIX=/ucrt64
        export PATH="/ucrt64/bin:$PATH"
    else
        echo "Error: MSYS2 pacman not found"
        echo "Install LLVM manually from https://releases.llvm.org/"
        exit 1
    fi
}

if [ "$INSTALL_LLVM" = true ]; then
    case $OS in
        linux)      install_llvm_linux ;;
        macos)      install_llvm_macos ;;
        windows)    install_llvm_windows ;;
        *)          echo "Unsupported OS for LLVM installation" ;;
    esac
fi

# =============================================================================
# Dependencies Installation
# =============================================================================
install_deps_linux() {
    echo "Installing dependencies on Linux..."

    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y build-essential bc python3
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y gcc gcc-c++ bc python3
    fi
}

install_deps_macos() {
    echo "Installing dependencies on macOS..."

    if command -v brew &> /dev/null; then
        brew install coreutils bc python3
    fi
}

install_deps_windows() {
    echo "Installing dependencies on Windows (MSYS2)..."

    if command -v pacman &> /dev/null; then
        pacman -S --noconfirm mingw-w64-ucrt-x86_64-gcc bc python3
    fi
}

if [ "$INSTALL_DEPS" = true ]; then
    case $OS in
        linux)      install_deps_linux ;;
        macos)      install_deps_macos ;;
        windows)    install_deps_windows ;;
        *)          echo "Unsupported OS for dependency installation" ;;
    esac
fi

# =============================================================================
# Environment Variables
# =============================================================================
setup_env() {
    # BMB Runtime Path
    if [ -z "$BMB_RUNTIME_PATH" ]; then
        if [ -f "$PROJECT_ROOT/bmb/runtime/libbmb_runtime.a" ]; then
            export BMB_RUNTIME_PATH="$PROJECT_ROOT/bmb/runtime/libbmb_runtime.a"
        fi
    fi

    # LLVM Path (auto-detect if not set)
    if [ -z "$LLVM_SYS_210_PREFIX" ]; then
        case $OS in
            linux)
                if [ -d "/usr/lib/llvm-21" ]; then
                    export LLVM_SYS_210_PREFIX="/usr/lib/llvm-21"
                elif [ -d "/usr/lib/llvm-20" ]; then
                    export LLVM_SYS_210_PREFIX="/usr/lib/llvm-20"
                fi
                ;;
            macos)
                if command -v brew &> /dev/null; then
                    LLVM_PATH="$(brew --prefix llvm@21 2>/dev/null || brew --prefix llvm 2>/dev/null)"
                    if [ -n "$LLVM_PATH" ]; then
                        export LLVM_SYS_210_PREFIX="$LLVM_PATH"
                    fi
                fi
                ;;
            windows)
                if [ -d "/ucrt64" ]; then
                    export LLVM_SYS_210_PREFIX="/ucrt64"
                elif [ -d "/mingw64" ]; then
                    export LLVM_SYS_210_PREFIX="/mingw64"
                fi
                ;;
        esac
    fi

    # Add LLVM to PATH if needed
    if [ -n "$LLVM_SYS_210_PREFIX" ] && [ -d "$LLVM_SYS_210_PREFIX/bin" ]; then
        export PATH="$LLVM_SYS_210_PREFIX/bin:$PATH"
    fi
}

setup_env

# =============================================================================
# Verification
# =============================================================================
if [ "$VERIFY" = true ]; then
    echo ""
    echo "=== Environment Verification ==="
    echo ""

    PASS=true

    # Rust
    echo -n "Rust: "
    if command -v rustc &> /dev/null; then
        rustc --version
    else
        echo "NOT FOUND"
        PASS=false
    fi

    # Cargo
    echo -n "Cargo: "
    if command -v cargo &> /dev/null; then
        cargo --version
    else
        echo "NOT FOUND"
        PASS=false
    fi

    # LLVM
    echo -n "LLVM: "
    if command -v llvm-config &> /dev/null; then
        llvm-config --version
    elif command -v llc &> /dev/null; then
        llc --version | head -2
    else
        echo "NOT FOUND (optional for native compilation)"
    fi

    # Clang
    echo -n "Clang: "
    if command -v clang &> /dev/null; then
        clang --version | head -1
    else
        echo "NOT FOUND (optional for native compilation)"
    fi

    # GCC
    echo -n "GCC: "
    if command -v gcc &> /dev/null; then
        gcc --version | head -1
    else
        echo "NOT FOUND"
    fi

    # Python
    echo -n "Python: "
    if command -v python3 &> /dev/null; then
        python3 --version
    else
        echo "NOT FOUND"
        PASS=false
    fi

    # bc
    echo -n "bc: "
    if command -v bc &> /dev/null; then
        echo "$(bc --version 2>&1 | head -1)"
    else
        echo "NOT FOUND"
    fi

    echo ""
    echo "=== Environment Variables ==="
    echo "LLVM_SYS_210_PREFIX: ${LLVM_SYS_210_PREFIX:-NOT SET}"
    echo "BMB_RUNTIME_PATH: ${BMB_RUNTIME_PATH:-NOT SET}"
    echo ""

    if [ "$PASS" = true ]; then
        echo "Environment verification: PASSED"
    else
        echo "Environment verification: FAILED"
        exit 1
    fi
fi

# =============================================================================
# Export for CI
# =============================================================================
if [ "$EXPORT" = true ]; then
    echo ""
    echo "# Add to GITHUB_ENV or shell profile:"
    echo "export LLVM_SYS_210_PREFIX=\"$LLVM_SYS_210_PREFIX\""
    echo "export BMB_RUNTIME_PATH=\"$BMB_RUNTIME_PATH\""
    echo "export PATH=\"$LLVM_SYS_210_PREFIX/bin:\$PATH\""

    # GitHub Actions format
    if [ -n "$GITHUB_ENV" ]; then
        echo "LLVM_SYS_210_PREFIX=$LLVM_SYS_210_PREFIX" >> $GITHUB_ENV
        echo "BMB_RUNTIME_PATH=$BMB_RUNTIME_PATH" >> $GITHUB_ENV
        echo "$LLVM_SYS_210_PREFIX/bin" >> $GITHUB_PATH
    fi
fi

echo ""
echo "CI environment setup complete"
