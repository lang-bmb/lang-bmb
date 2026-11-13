#!/usr/bin/env bash
# Run a command while holding the gate lock (backlog INF-35). One gate at a
# time per repository, over all its worktrees: a gate under a second gate or
# a gate-power run went past the health job's time limit (load 17), and the
# verdict was the machine's, not the commit's. A second holder waits.
#
#   scripts/with_gate_lock.sh CMD [ARG..]
#
# The command runs with BMB_GATE_LOCK_HELD=1; with that already set, it runs
# at once (scripts/gate_power.py takes the lock before it starts a mutant's
# clock and runs the gate under it).
set -euo pipefail
if [ -n "${BMB_GATE_LOCK_HELD:-}" ]; then
  exec "$@"
fi
dir=$(git -C "$(dirname "$0")" rev-parse --path-format=absolute --git-common-dir 2> /dev/null) || dir=${TMPDIR:-/tmp}
lock="${BMB_GATE_LOCK:-$dir/gate.lock}"
exec 9> "$lock"
if ! flock -n 9; then
  echo "gate: another gate holds $lock; waiting" >&2
  flock 9
fi
BMB_GATE_LOCK_HELD=1 exec "$@"
