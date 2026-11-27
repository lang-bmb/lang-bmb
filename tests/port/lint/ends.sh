#!/bin/sh
# The ends of `lint` without `--select` and of `mutate` that are no report,
# on the programs of ../sat: the checks of a plain `lint` with no solver on
# PATH (the first verification cannot run), the checks that need no solver
# with none, a check that proves again alone, the last `--select` taken
# among checks of both kinds, a workspace directory in place of files, a file that cannot be read and a program that does
# not check; for `mutate` a path that is a `spec fn`, one that is no
# definition after one that is, the same path twice, a program that does
# not check, a file that cannot be read and no file after the paths.
#   ends.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
lint=$here/../sat/lint.bmb
run() {
  "$bmb" "$@"
  echo "exit $?"
}
PATH=/nonexistent run lint "$lint"
PATH=/nonexistent run lint --select W0906,W0907,W0909,W0916 "$lint"
PATH=/nonexistent run lint --select W0903 "$lint"
PATH=/nonexistent run lint --select W1002 "$lint"
run lint --select W0904 --select W0909,W0916,W1002 "$lint"
run lint --select W0916 --select W0903 "$lint"
"$bmb" import ws "$lint"
echo "exit $?"
run lint --select W0902,W0903,W0909 ws
run lint "$here/absent.bmb"
printf '#[summary("t")]\nmod m { fn f( }\n' > bad.bmb
run lint bad.bmb
run lint --select W0907 bad.bmb
run mutate --def lint::pos "$lint"
run mutate --def lint::above --def lint::nothing "$lint"
run mutate --def lint::above --def lint::above "$lint"
run mutate --def lint::above ws
run mutate --def m::f bad.bmb
run mutate --def lint::above "$here/absent.bmb"
