#!/bin/sh
# The ends of `stability` that are no report, and the counts of seeds it
# reads, on trust.bmb of prove/: counts it refuses (no digits, a sign, empty,
# above the largest u64) and takes (a `+`, a leading zero, 0, which still
# runs once), `--seeds` without its file or given twice (the rest are files),
# a file that cannot be read, a program that does not check, no solver on
# PATH (every definition failed under every seed: stable).
#   ends.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
trust=$here/../prove/trust.bmb
run() {
  "$bmb" stability "$@"
  echo "exit $?"
}
run --seeds x "$trust"
run --seeds -1 "$trust"
run --seeds '' "$trust"
run --seeds + "$trust"
run --seeds 18446744073709551616 "$trust"
run --seeds +2 "$trust"
run --seeds 02 "$trust"
run --seeds 0 "$trust"
run --seeds "$trust"
run --seeds 2 --seeds 3 "$trust"
run "$here/absent.bmb"
printf '#[summary("t")]\nmod m { fn f( }\n' > bad.bmb
run bad.bmb
PATH=/nonexistent run --seeds 2 "$trust"
