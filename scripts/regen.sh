#!/usr/bin/env bash
# Write every generated document and block from the working tree, then check them with scripts/fresh.sh:
#
#   docs/diagnostics.md, the reference index, the surface fingerprint (while
#   the surface is not frozen), the conformance table, and the port ledger
#   when BMB_PORT_LEDGER names it.
#
#   scripts/regen.sh              # all of the above
#
# They are generated from the working tree: what is committed must be the
# tree they were made from. Uncommitted changes are listed at the end, and
# the gate checks the committed tree again (scripts/ship.sh).
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build --quiet
# The port's proof (the ledger reads it) runs in the optimized build: after a change of the compiler every definition is encoded again.
cargo build --release --quiet
export BMB_PROOF_BMB="$PWD/target/release/bmb"
bmb=target/debug/bmb
"$bmb" codes --md > docs/diagnostics.md
python3 scripts/reference.py index --write
if grep -q '"frozen": false' tests/surface.json; then
  python3 scripts/surface.py record
fi
python3 scripts/conformance.py table --write
if [ -n "${BMB_PORT_LEDGER:-}" ]; then
  python3 scripts/components.py ledger --write
fi

rc=0
BMB_FRESH_NO_BUILD=1 scripts/fresh.sh || rc=$?
dirty=$(git status --porcelain)
if [ -n "$dirty" ]; then
  echo "regen: generated from a working tree with uncommitted changes; commit them together:"
  printf '%s\n' "$dirty" | sed 's/^/  /'
fi
exit $rc
