#!/bin/bash
# BMB Version Sync Check
# Verifies that workspace Cargo.toml and bootstrap/version.bmb agree.
#
# Exits 0 if versions match, 1 otherwise.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CARGO_VERSION=$(awk '/^\[workspace.package\]/{f=1} f && /^version = /{gsub(/"/,"",$3); print $3; exit}' "$PROJECT_ROOT/Cargo.toml")
BMB_VERSION=$(awk -F'"' '/fn bmb_version\(\) -> String =/{print $2; exit}' "$PROJECT_ROOT/bootstrap/version.bmb")

if [ -z "$CARGO_VERSION" ]; then
    echo "ERROR: failed to read workspace.package.version from Cargo.toml" >&2
    exit 1
fi
if [ -z "$BMB_VERSION" ]; then
    echo "ERROR: failed to read bmb_version() from bootstrap/version.bmb" >&2
    exit 1
fi

if [ "$CARGO_VERSION" != "$BMB_VERSION" ]; then
    echo "VERSION MISMATCH:" >&2
    echo "  Cargo.toml workspace.package.version = $CARGO_VERSION" >&2
    echo "  bootstrap/version.bmb bmb_version()  = $BMB_VERSION" >&2
    echo "Run scripts/update-version.sh <version> to align." >&2
    exit 1
fi

echo "version sync OK: $CARGO_VERSION"
