#!/bin/sh
# The ends of `lint` and `examples` that are no report, on the files beside
# this script: checks that do not exist (an empty one, one with a quote),
# the last `--select` taken, a file that cannot be read, a program that
# does not check, a proof cache written, one that cannot be written and one
# under a path that is no directory, no solver on PATH; for `examples` the
# counts it refuses and takes, no `--`, a path that is no executable
# function (after one that is, whose queries were asked), a program that
# does not check.
#   ends.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
lint=$here/lint.bmb
ex=$here/examples.bmb
run() {
  "$bmb" "$@"
  echo "exit $?"
}
run lint --select W0999 "$lint"
run lint --select W0901, "$lint"
run lint --select 'W09"01\' "$lint"
run lint --select W0918 --select W0901 "$lint"
run lint --select W0901 "$here/absent.bmb"
printf '#[summary("t")]\nmod m { fn f( }\n' > bad.bmb
run lint --select W0901 bad.bmb
run lint --cache made --select W0901,W0918 "$lint"
mkdir -p held/proofs
run lint --cache held --select W0918 "$lint"
run lint --cache /dev/null/cache --select W0901 "$lint"
PATH=/nonexistent run lint --select W0901,W0918 "$lint"
PATH=/nonexistent run lint --select W0905 "$lint"
run examples --count 0 d::twice -- "$ex"
run examples --count 65 d::twice -- "$ex"
run examples --count x d::twice -- "$ex"
run examples --count +2 d::twice -- "$ex"
run examples --count 05 d::above -- "$ex"
run examples d::twice "$ex"
run examples d::twice d::nothing -- "$ex"
run examples d::sorted -- "$ex"
run examples m::f -- bad.bmb
run examples d::twice -- "$here/absent.bmb"
PATH=/nonexistent run examples d::twice u::long -- "$ex"
