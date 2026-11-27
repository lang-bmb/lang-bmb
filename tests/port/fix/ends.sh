#!/bin/sh
# The ends of `fix` that are no report of one file: the two units of the
# Rust compiler's own test of a program in several files (`fix-04.txt` and
# `fix-05.txt`, each fixed in its own file, reported, printed and written),
# a workspace directory in place of files, a file that cannot be read, a
# file that is read and one that cannot be, and the flags in every order.
#   ends.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
run() {
  "$bmb" "$@"
  echo "exit $?"
}
cp "$here/fix-04.txt" a.bmb
cp "$here/fix-05.txt" b.bmb
run fix a.bmb b.bmb
run fix --print a.bmb b.bmb
run fix b.bmb a.bmb
run fix a.bmb
run fix --confirm --induction --write a.bmb b.bmb
run fix --write a.bmb b.bmb
cp "$here/fix-01.txt" h.bmb
"$bmb" import ws "$here/../lint/fix-02.txt"
echo "exit $?"
run fix ws
run fix --write --confirm ws h.bmb
run fix "$here/absent.bmb"
run fix h.bmb "$here/absent.bmb"
run fix --print --induction --confirm h.bmb
run fix --induction --confirm --print h.bmb
