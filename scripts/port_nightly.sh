#!/usr/bin/env bash
# The port's differential cases too long for the per-push gate
# (tests/port/nightly/cases.json, described in tests/port/README.md). CI
# runs them every night, one job per part (.github/workflows/nightly.yml);
# locally this one command runs them all, or the parts and cases its
# arguments name.
#
# The optimized compiler is the compiler under test (the cases' recorded
# seconds are its). The port is proved as the gate proves it, into a cache
# of its own (target/nightly-proof, so that a gate can run beside it), the
# programs the cases compare are built from it (target/nightly-build), and
# the cases run. The lines of the run are left in target/nightly.jsonl and
# the bytes around each difference in target/nightly-excerpts/CASE.
#
#   scripts/port_nightly.sh [--shard I/N] [--case NAME]..
set -euo pipefail
cd "$(dirname "$0")/.."
bmb=target/release/bmb
cargo build --release --quiet
rm -rf target/nightly-proof target/nightly-build target/nightly-excerpts
python3 scripts/port_proof.py --bmb "$bmb" target/nightly-proof selfhost/*.bmb
python3 scripts/difftest.py build tests/port/nightly/cases.json target/nightly-build --bmb "$bmb" --proof target/nightly-proof
python3 scripts/difftest.py run tests/port/nightly/cases.json --built target/nightly-build --bmb "$bmb" \
  --excerpts target/nightly-excerpts "$@" | tee target/nightly.jsonl
