#!/bin/sh
# The ends of `uses` that are no search of one file: a program that does
# not check, a program in two files, a workspace directory
# in place of files (searched and written), a file that cannot be read, a
# program that names a library unit (the library's definitions are not
# searched), no solver on PATH, and `--cache` with a directory that cannot
# be made (the search runs, the caches are not written).
#   ends.sh BMB ROOT
bmb=$1
root=$2
here=$(cd "$(dirname "$0")" && pwd)
run() {
  "$bmb" "$@"
  echo "exit $?"
}
printf 'mod m {}\n' > bad.bmb
run uses bad.bmb
run uses --write bad.bmb
cp "$here/uses-02.txt" p.bmb
cp "$here/uses-03.txt" a.bmb
run uses p.bmb a.bmb
run uses --write a.bmb p.bmb
"$bmb" import ws "$here/uses-01.txt"
echo "exit $?"
run uses ws
run uses --write ws
run uses "$here/absent.bmb"
run uses "${root}tests/run/std_num.bmb"
PATH=/nonexistent run uses "$here/uses-02.txt"
: > held
run uses --cache held/cache "$here/uses-02.txt"
