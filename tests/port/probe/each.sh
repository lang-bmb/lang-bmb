#!/bin/sh
# `probe` on the programs of the Rust compiler's own tests of it
# (`probe-11.txt`: candidates that hold, one the body does not keep, one
# false on entry, one that does not check; `probe-12.txt`: the inner loop
# by its number, a candidate out of scope at the outer one) and on loops of
# the corpus: every loop of a function by its number, candidates that leave
# in a later round, all of them leaving, more than 16 candidates, a loop
# and a definition that are not there, a `--loop` written with a sign, the
# last `--def` and `--loop` taken, a program that does not check, a file
# that cannot be read, a workspace directory in place of files and no
# solver on PATH. Nothing is written.
#   each.sh BMB ROOT
bmb=$1
root=$2
here=$(cd "$(dirname "$0")" && pwd)
run() {
  "$bmb" probe "$@"
  echo "exit $?"
}
lp=$here/probe-11.txt
ns=$here/probe-12.txt
loops=${root}tests/verify/pass/loops.bmb
run --def lp::sum --loop 0 --candidate 'i <= n' --candidate 's <= 1000 * i' --candidate 's == 0' --candidate 'i > 5' --candidate 'x > 1' "$lp"
run --def lp::sum --loop 1 --candidate true "$lp"
run --def lp::sum --candidate 'i <= n' --candidate 's <= 1000 * i' "$lp"
run --def lp::sum --loop +0 --candidate 'i <= n' "$lp"
run --def lp::sum --loop 00 --candidate 'i <= n && ' --candidate '' --candidate 'nope(' "$lp"
run --def lp::nope --candidate true "$lp"
run --def sum --candidate true "$lp"
run --def lp::sum --loop 1 --def lp::sum --loop 0 --candidate false --candidate true "$lp"
run --def lp::sum --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true --candidate true "$lp"
run --def ns::grid --loop 1 --candidate 'j <= n' --candidate 's <= n * i + j' --candidate 'j == 0' "$ns"
run --def ns::grid --loop 0 --candidate 'j <= n' "$ns"
run --def ns::grid --loop 2 --candidate true "$ns"
run --def ns::grid --loop 0 --candidate 'i <= n' --candidate 's <= n * i' --candidate 's == i' "$ns"
run --def loops::sum_to --candidate true --candidate false --candidate '1 == 1' --candidate 'i <= n' --candidate 'i < n' --candidate 'i > 5' --candidate 's >= 0' --candidate 'missing > 0' --candidate 's <= i * 1000' "$loops" "${root}tests/verify/pass/loop_exits.bmb"
run --def loops::sum_to --candidate 'i <= n' "${root}tests/verify/fail/loops.bmb"
run --def loops::sum_to --candidate 'i <= n' "${root}tests/check/err/types.bmb"
run --def loops::sum_to --candidate 'i <= n' "$here/absent.bmb"
run --def std::num::push_u64 --candidate true "${root}tests/run/std_num.bmb"
"$bmb" import ws "$loops"
echo "exit $?"
run --def loops::sum_to --candidate 'i <= n' ws
PATH=/nonexistent run --def lp::sum --candidate 'i <= n' "$lp"
