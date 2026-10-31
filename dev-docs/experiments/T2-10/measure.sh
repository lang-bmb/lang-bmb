#!/bin/sh
# T2-10: rlimit and verdicts of the vacuity check (vac) or the precondition
# check (pre) over the programs of `proofs_are_not_vacuous` and bench/game,
# or over tests/verify/fail with a third argument `fail`.
# usage: measure.sh OUT vac|pre [fail]      (run from the repository root)
# Needs the driver built: cp dev-docs/experiments/T2-10/cost_driver.rs
# compiler/tests/zz_cost.rs && cargo test --test zz_cost --no-run
# (remove the copy afterwards). Writes OUT/CHECK.log (one row per solver
# query: limit, answer, rlimit), OUT/q-CHECK/ (the query texts) and
# OUT/CHECK.out (the verdicts), CHECK being vac, pre, vacfail or prefail;
# summarize.py OUT [fail] adds them up.
set -eu
out=$(realpath -m "$1"); check=$2${3:-}
here=$(dirname "$(realpath "$0")")
mkdir -p "$out/q-$check" "$out/bin"
ln -sf "$here/z3log.py" "$out/bin/z3"
rm -f "$out/$check.log"
bin=${BIN:-$(ls -t target/debug/deps/zz_cost-* | grep -v '\.d$' | head -1)}
cd compiler
env PATH="$out/bin:$PATH" Z3_LOG="$out/$check.log" Z3_QDIR="$out/q-$check" COST_CHECK="$2" \
  ${3:+COST_FAIL=1} "$bin" --ignored --nocapture --test-threads 1 > "$out/$check.out" 2>&1
