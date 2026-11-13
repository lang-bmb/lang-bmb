#!/usr/bin/env bash
# Write every generated document and block from the working tree (backlog
# INF-34), then check them with scripts/fresh.sh:
#
#   docs/diagnostics.md, the reference index, the surface fingerprint (while
#   the surface is not frozen), the conformance table, the port ledger, the
#   backlog archive, the roadmap's status block.
#
#   scripts/regen.sh              # all of the above
#   scripts/regen.sh --progress   # also dev-docs/reports/progress.md (weekly)
#
# They are generated from the working tree: what is committed must be the
# tree they were made from. Uncommitted changes are listed at the end, and
# the gate checks the committed tree again (scripts/ship.sh).
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build --quiet
bmb=target/debug/bmb
"$bmb" codes --md > docs/diagnostics.md
python3 scripts/backlog.py archive
python3 scripts/reference.py index --write
if grep -q '"frozen": false' dev-docs/health/surface.json; then
  python3 scripts/surface.py record
fi
python3 scripts/conformance.py table --write
python3 scripts/components.py ledger --write
python3 scripts/progress.py --write-roadmap
if [ "${1:-}" = "--progress" ]; then
  python3 scripts/progress.py --write dev-docs/reports/progress.md
fi

rc=0
BMB_FRESH_NO_BUILD=1 scripts/fresh.sh || rc=$?
dirty=$(git status --porcelain)
if [ -n "$dirty" ]; then
  echo "regen: generated from a working tree with uncommitted changes; commit them together:"
  printf '%s\n' "$dirty" | sed 's/^/  /'
fi
exit $rc
