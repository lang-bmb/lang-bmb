#!/usr/bin/env bash
# What goes stale, checked in seconds (backlog INF-34, INF-37): the generated
# documents and blocks, the backlog's form, the surface fingerprint, the
# gate-power mutants and the health metrics that need no solver. The gate
# runs this first: these used to fail forty minutes into a run, three times
# in one plan.
#
#   scripts/fresh.sh           # build target/debug/bmb, then check
#   scripts/regen.sh           # write what this finds stale
#
# Every check runs; the exit code is non-zero when any fails.
# BMB_FRESH_NO_BUILD=1 takes target/debug/bmb as it is (the script's test).
set -uo pipefail
cd "$(dirname "$0")/.."

if [ -z "${BMB_FRESH_NO_BUILD:-}" ]; then
  cargo build --quiet || exit 1
fi
bmb=target/debug/bmb
[ -x "$bmb" ] || { echo "fresh: $bmb is missing" >&2; exit 1; }

bad=0
check() {
  local name=$1 out
  shift
  if ! out=$("$@" 2>&1); then
    printf 'fresh: %s\n' "$name" >&2
    printf '%s\n' "$out" | tail -8 | sed 's/^/  /' >&2
    bad=1
  fi
}
diagnostics() {
  "$bmb" codes --md | diff -q docs/diagnostics.md - > /dev/null \
    || { echo "docs/diagnostics.md is stale: run scripts/regen.sh"; return 1; }
}

check "diagnostic codes document" diagnostics
check "backlog" python3 scripts/check_backlog.py
check "language reference index" python3 scripts/reference.py check
check "surface fingerprint" python3 scripts/surface.py check
check "conformance table" python3 scripts/conformance.py check
check "components and port ledger" python3 scripts/components.py check
check "roadmap status block" python3 scripts/progress.py --check-roadmap
# A gate-power run applies one mutant, so its mutants do not all apply.
if [ -z "${BMB_GATE_POWER_RUN:-}" ]; then
  check "gate-power mutants" python3 scripts/gate_power.py check
fi
check "health (quick)" python3 scripts/health.py quick
exit $bad
