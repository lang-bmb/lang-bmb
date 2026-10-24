#!/bin/bash
# Cloud session setup (INF-04): pinned toolchain + warm build cache.
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

cd "$CLAUDE_PROJECT_DIR"
./scripts/setup.sh
cargo build --quiet --all-targets
