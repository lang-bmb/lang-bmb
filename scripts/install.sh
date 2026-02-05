#!/bin/bash
# BMB Installation Script
# Installs BMB compiler and runtime to system directories
#
# Usage:
#   ./install.sh [options]
#
# Options:
#   --prefix PATH    Installation prefix (default: /usr/local)
#   --user           Install to ~/.local instead of system
#   --uninstall      Remove BMB installation
#   --help           Show this help

set -e

# Defaults
PREFIX="/usr/local"
USER_INSTALL=false
UNINSTALL=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "$1"; }
error() { echo -e "${RED}Error: $1${NC}" >&2; exit 1; }

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --user)
            USER_INSTALL=true
            PREFIX="$HOME/.local"
            shift
            ;;
        --uninstall)
            UNINSTALL=true
            shift
            ;;
        --help)
            head -20 "$0" | tail -15
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Detect script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd 2>/dev/null || echo "$SCRIPT_DIR")"

# Detect platform
detect_platform() {
    case "$(uname -s)" in
        Linux)
            if [ "$(uname -m)" = "aarch64" ]; then
                PLATFORM="linux-aarch64"
            else
                PLATFORM="linux-x86_64"
            fi
            ;;
        Darwin)
            PLATFORM="darwin-universal"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            PLATFORM="windows-x64"
            ;;
        *)
            error "Unsupported platform: $(uname -s)"
            ;;
    esac
}

# Find BMB binary
find_bmb_binary() {
    # Try golden directory first
    if [ -f "$PROJECT_ROOT/golden/$PLATFORM/bmb" ]; then
        BMB_BINARY="$PROJECT_ROOT/golden/$PLATFORM/bmb"
    elif [ -f "$PROJECT_ROOT/golden/$PLATFORM/bmb.exe" ]; then
        BMB_BINARY="$PROJECT_ROOT/golden/$PLATFORM/bmb.exe"
    elif [ -f "$PROJECT_ROOT/bmb" ]; then
        BMB_BINARY="$PROJECT_ROOT/bmb"
    elif [ -f "$PROJECT_ROOT/bmb.exe" ]; then
        BMB_BINARY="$PROJECT_ROOT/bmb.exe"
    else
        error "BMB binary not found. Run ./scripts/golden-bootstrap.sh first."
    fi
}

# Uninstall
do_uninstall() {
    log "${YELLOW}Uninstalling BMB from $PREFIX...${NC}"

    rm -f "$PREFIX/bin/bmb" "$PREFIX/bin/bmb.exe" 2>/dev/null || true
    rm -rf "$PREFIX/lib/bmb" 2>/dev/null || true
    rm -rf "$PREFIX/share/bmb" 2>/dev/null || true

    log "${GREEN}BMB uninstalled successfully${NC}"
}

# Install
do_install() {
    detect_platform
    find_bmb_binary

    log "${YELLOW}Installing BMB to $PREFIX...${NC}"
    log "Platform: $PLATFORM"
    log "Binary: $BMB_BINARY"

    # Create directories
    mkdir -p "$PREFIX/bin"
    mkdir -p "$PREFIX/lib/bmb/runtime"
    mkdir -p "$PREFIX/share/bmb/bootstrap"
    mkdir -p "$PREFIX/share/bmb/stdlib"

    # Install binary
    if [[ "$PLATFORM" == "windows-x64" ]]; then
        cp "$BMB_BINARY" "$PREFIX/bin/bmb.exe"
        chmod +x "$PREFIX/bin/bmb.exe"
    else
        cp "$BMB_BINARY" "$PREFIX/bin/bmb"
        chmod +x "$PREFIX/bin/bmb"
    fi

    # Install runtime
    if [ -d "$PROJECT_ROOT/bmb/runtime" ]; then
        cp -r "$PROJECT_ROOT/bmb/runtime/"* "$PREFIX/lib/bmb/runtime/"
    elif [ -d "$PROJECT_ROOT/runtime" ]; then
        cp -r "$PROJECT_ROOT/runtime/"* "$PREFIX/lib/bmb/runtime/"
    fi

    # Install bootstrap sources (for self-compilation)
    if [ -d "$PROJECT_ROOT/bootstrap" ]; then
        cp -r "$PROJECT_ROOT/bootstrap/"*.bmb "$PREFIX/share/bmb/bootstrap/" 2>/dev/null || true
    fi

    # Install stdlib
    if [ -d "$PROJECT_ROOT/stdlib" ]; then
        cp -r "$PROJECT_ROOT/stdlib/"* "$PREFIX/share/bmb/stdlib/" 2>/dev/null || true
    fi

    log ""
    log "${GREEN}BMB installed successfully!${NC}"
    log ""
    log "Installation summary:"
    log "  Binary:    $PREFIX/bin/bmb"
    log "  Runtime:   $PREFIX/lib/bmb/runtime/"
    log "  Bootstrap: $PREFIX/share/bmb/bootstrap/"
    log "  Stdlib:    $PREFIX/share/bmb/stdlib/"
    log ""

    # Check if prefix is in PATH
    if [[ ":$PATH:" != *":$PREFIX/bin:"* ]]; then
        log "${YELLOW}Note: Add $PREFIX/bin to your PATH:${NC}"
        log "  export PATH=\"$PREFIX/bin:\$PATH\""
        log ""
    fi

    # Set BMB_RUNTIME_PATH hint
    log "Set runtime path for compilation:"
    log "  export BMB_RUNTIME_PATH=\"$PREFIX/lib/bmb/runtime\""
}

# Main
if [ "$UNINSTALL" = true ]; then
    do_uninstall
else
    do_install
fi
