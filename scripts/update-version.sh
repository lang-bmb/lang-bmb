#!/bin/bash
# BMB Version Update Script
# Updates version across all relevant files
#
# Usage:
#   ./scripts/update-version.sh <new-version>
#   ./scripts/update-version.sh 0.61.0
#
# Files updated:
#   - VERSION
#   - golden/VERSION
#   - bootstrap/version.bmb
#   - Cargo.toml (if exists)

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.61.0"
    exit 1
fi

NEW_VERSION="$1"
DATE=$(date -u +%Y-%m-%d)

# Parse version components
IFS='.' read -ra VERSION_PARTS <<< "$NEW_VERSION"
MAJOR="${VERSION_PARTS[0]:-0}"
MINOR="${VERSION_PARTS[1]:-0}"
PATCH="${VERSION_PARTS[2]:-0}"

echo "Updating BMB version to $NEW_VERSION"
echo "  Major: $MAJOR"
echo "  Minor: $MINOR"
echo "  Patch: $PATCH"
echo "  Date:  $DATE"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Update VERSION file
echo "$NEW_VERSION" > "$PROJECT_ROOT/VERSION"
echo "Updated VERSION"

# Update golden/VERSION
if [ -d "$PROJECT_ROOT/golden" ]; then
    echo "v$NEW_VERSION" > "$PROJECT_ROOT/golden/VERSION"
    echo "$DATE" >> "$PROJECT_ROOT/golden/VERSION"
    echo "Updated golden/VERSION"
fi

# Update bootstrap/version.bmb
cat > "$PROJECT_ROOT/bootstrap/version.bmb" << EOF
// BMB Version Information
// This file is automatically updated by build scripts

fn bmb_version() -> String = "$NEW_VERSION";
fn bmb_version_major() -> i64 = $MAJOR;
fn bmb_version_minor() -> i64 = $MINOR;
fn bmb_version_patch() -> i64 = $PATCH;

fn bmb_build_date() -> String = "$DATE";
EOF
echo "Updated bootstrap/version.bmb"

# Update Cargo.toml if exists
if [ -f "$PROJECT_ROOT/Cargo.toml" ]; then
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$PROJECT_ROOT/Cargo.toml"
    rm -f "$PROJECT_ROOT/Cargo.toml.bak"
    echo "Updated Cargo.toml"
fi

# Update bmb/Cargo.toml if exists
if [ -f "$PROJECT_ROOT/bmb/Cargo.toml" ]; then
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$PROJECT_ROOT/bmb/Cargo.toml"
    rm -f "$PROJECT_ROOT/bmb/Cargo.toml.bak"
    echo "Updated bmb/Cargo.toml"
fi

echo ""
echo "Version updated to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Commit: git commit -am 'chore: bump version to $NEW_VERSION'"
echo "  3. Tag: git tag v$NEW_VERSION"
echo "  4. Push: git push && git push --tags"
