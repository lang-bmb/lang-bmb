#!/usr/bin/env bash
# The product gate: build, tests, proofs, differential tests and the fixed
# point. It passes with this repository alone. Local runs and CI use this same
# script. Judge it by its exit code. The benchmarks and the self-checks of
# their harnesses are in the bmb-bench repository
# (https://github.com/lang-bmb/bmb-bench), whose own check runs against a
# checkout of this one.
#
# The quick sequential steps stop at the first failure. After the one build,
# the test executables and the script checks are independent and run
# concurrently (scripts/gate_jobs.py): each runs to the end,
# a failing job's output is printed with its name, and any failure makes the
# exit code non-zero.
#
# One gate runs at a time: a second one waits for the lock. What goes
# stale is checked first, in seconds (scripts/fresh.sh). A run that
# leaves a file in the tree fails at the end.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -z "${BMB_GATE_LOCK_HELD:-}" ]; then
  exec scripts/with_gate_lock.sh "$0" "$@"
fi

step() { printf '== %s\n' "$1"; }

tree=$(mktemp)
built=$(mktemp)
trap 'rm -f "$tree" "$built"' EXIT
python3 scripts/tree_state.py snapshot > "$tree"

step "format";  cargo fmt --all --check
# The generated documents, the surface fingerprint, the
# gate-power mutants and the solver-free health metrics.
# A gate-power run applies one mutant: the documents that count source lines
# are stale by construction and are no evidence about the mutant, so of these
# only the surface fingerprint runs, as a job.
power_jobs=()
if [ -z "${BMB_GATE_POWER_RUN:-}" ]; then
  step "fresh"; scripts/fresh.sh
else
  power_jobs=(--job "surface fingerprint=python3 scripts/surface.py check")
fi
step "lint";    cargo clippy --all-targets --quiet -- -D warnings
step "build"   # every test executable and target/debug/bmb, which the jobs below use
cargo test --no-run --quiet --message-format=json-render-diagnostics > "$built"
# The port is proved once, into an empty cache: the jobs that read its proof
# (the port's differential tests, components, health S7) reuse it instead of each proving the
# whole port again. Its definitions are proved in as many
# processes as the machine has cores, each from an empty cache of its own,
# and the single run below gives the verdict from the merged cache. The optimized build of the same sources proves it: no solver time
# is saved, but building the queries is about ten times faster than in the
# debug build, which took most of this step (the tests above
# and the jobs below still run the debug build).
step "port proof"
cargo build --release --quiet
export BMB_PROOF_BMB="$PWD/target/release/bmb"
rm -rf target/port-proof
python3 scripts/port_proof.py --bmb "$BMB_PROOF_BMB" target/port-proof selfhost/*.bmb > /dev/null
if ! "$BMB_PROOF_BMB" verify --cache target/port-proof selfhost/*.bmb > "$built.port"; then
  grep -v '"status":"verified"' "$built.port" | head -20
  rm -f "$built.port"
  exit 1
fi
# The bounds of the port's self-proof (ADR-0162): no row with a trusted
# fact, an escape or termination left unproved, which the exit code above
# does not tell, and the places trusted by reading where the record lists
# them.
if ! python3 scripts/trusted_steps.py rows "$built.port" || ! python3 scripts/trusted_steps.py steps; then
  rm -f "$built.port"
  exit 1
fi
rm -f "$built.port"

# The programs the port's differential cases compare (stage1 and the
# others in tests/port/cases.json), built once by the compiler under test
# for the jobs below, as many at once as there are cores, each BMB program
# from a copy of the proof cache above. A build that fails leaves its error next to where the
# program would be and does not stop the gate here: the jobs whose cases need
# it fail with it, so the failure is a job's.
step "port build"
rm -rf target/port-build
python3 scripts/difftest.py build tests/port/cases.json target/port-build > /dev/null || true

# The cases run as port_shards jobs, each a part balanced by the cases'
# recorded seconds (tests/port/costs.json); every case is in exactly one
# (scripts/test_difftest.py).
port_shards=6
port_jobs=()
for i in $(seq 1 "$port_shards"); do
  port_jobs+=(--job "port difftest $i/$port_shards=python3 scripts/difftest.py run tests/port/cases.json --built target/port-build --shard $i/$port_shards")
done

step "test, surface, port checks, port difftest 1..$port_shards, bootstrap fixed point, gate scripts, gate power rules, solver stability rules, conformance suite, components and port ledger (concurrent)"
python3 scripts/gate_jobs.py --cargo-json "$built" "${power_jobs[@]}" "${port_jobs[@]}" \
  --job "test doc=cargo test --doc --quiet" \
  --job "surface=python3 scripts/test_surface.py" \
  --job "port checks=python3 scripts/test_difftest.py && python3 scripts/test_port_proof.py > /dev/null && python3 scripts/test_trusted_steps.py 2> /dev/null && python3 scripts/std_units.py check > /dev/null && python3 scripts/codegen_text.py check > /dev/null && python3 scripts/builtins.py check > /dev/null && target/debug/bmb fmt --check selfhost/*.bmb && target/debug/bmb fmt --check lib/std/*.bmb" \
  --job "bootstrap fixed point=python3 scripts/bootstrap.py run > /dev/null" \
  --job "gate scripts=python3 scripts/test_gate_scripts.py" \
  --job "gate power rules=python3 scripts/test_gate_power.py" \
  --job "solver stability rules=python3 scripts/test_solver_stability.py" \
  --job "conformance suite=python3 scripts/test_conformance.py && python3 scripts/conformance.py run" \
  --job "components and port ledger=python3 scripts/test_components.py > /dev/null"
step "tree";    python3 scripts/tree_state.py left "$tree"
echo "all checks passed"
