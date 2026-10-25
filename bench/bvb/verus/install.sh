#!/usr/bin/env bash
# Installs the pinned original Verus release for the BVB comparison (backlog 2A-09).
#
#   install.sh [--dry-run]
#
# Needs: curl, unzip, sha256sum, rustup (Verus runs on a pinned Rust toolchain),
# and outbound HTTPS to github.com (release asset) and static.rust-lang.org
# (toolchain). Installs into $BVB_VERUS_HOME (default ~/.cache/bmb/verus/VERSION)
# and prints the path of the `verus` binary last. Idempotent: an existing
# install whose `verus --version` reports the pinned version is kept.
# --dry-run prints what would be done and changes nothing.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
# shellcheck disable=SC1091
source "$here/verus.env"

dest="${BVB_VERUS_HOME:-$HOME/.cache/bmb/verus/$VERUS_VERSION}"
url="https://github.com/verus-lang/verus/releases/download/release/${VERUS_VERSION}/${VERUS_ASSET}"
bin="$dest/verus-x86-linux/verus"

installed() {
  [ -x "$bin" ] || return 1
  local v
  v="$(RUSTUP_TOOLCHAIN="$VERUS_RUST_TOOLCHAIN" "$bin" --version 2>/dev/null)" || return 1
  [[ "$v" == *"Version: ${VERUS_VERSION}"* ]]
}

if [ "${1:-}" = "--dry-run" ]; then
  echo "verus ${VERUS_VERSION}"
  echo "  asset:     $url"
  echo "  sha256:    $VERUS_SHA256"
  echo "  install:   $dest"
  echo "  toolchain: rustup toolchain install $VERUS_RUST_TOOLCHAIN --profile minimal"
  if installed; then echo "  status:    installed ($bin)"; else echo "  status:    not installed"; fi
  exit 0
fi

if [ "$(uname -s)-$(uname -m)" != "Linux-x86_64" ]; then
  echo "the pinned asset is for Linux x86_64; set VERUS_ASSET in verus.env for this platform" >&2
  exit 1
fi
for tool in curl unzip sha256sum rustup; do
  command -v "$tool" >/dev/null || { echo "missing $tool" >&2; exit 1; }
done

[[ "$(rustup toolchain list)" == *"${VERUS_RUST_TOOLCHAIN}"* ]] \
  || rustup toolchain install "$VERUS_RUST_TOOLCHAIN" --profile minimal >&2

if ! installed; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  echo "downloading $url" >&2
  curl -sSfL -o "$tmp/verus.zip" "$url"
  echo "$VERUS_SHA256  $tmp/verus.zip" | sha256sum -c --quiet -
  rm -rf "$dest"
  mkdir -p "$dest"
  unzip -q "$tmp/verus.zip" -d "$dest"
fi
installed || { echo "verus ${VERUS_VERSION} did not install at $bin" >&2; exit 1; }
echo "$bin"
