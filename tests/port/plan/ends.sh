#!/bin/sh
# ends.sh BMB EDIT ROOT: the ends of replacing a workspace's files (EDIT
# `plan`) that are no plain commit. The workspace is ROOT's
# tests/port/incr/base.txt laid out by BMB (`bmb import`); the new files
# are that workspace's, with one more file under a name given on the
# command line:
# - a file whose name is not its unit's (the name with a directory, without
#   `src/`, without an extension, with two, starting with a dot, with dots
#   and slashes around it), a file that holds two units, one that holds
#   none, one that holds the unit of another file;
# - a file with a syntax error (its diagnostics alone, whatever it holds),
#   one with a type error, one that is the standard library's unit;
# - a workspace before that does not parse or does not check (nothing to
#   compare with: everything is changed);
# - workspaces that cannot be read: `src` that is a file, a link, a
#   directory with a link in it or a directory inside it.
bmb=$1
edit=$2
root=$3
"$bmb" import base "$root/tests/port/incr/base.txt"
mkdir none
unit=$(sed -n 's/^mod \([a-z_0-9]*\) {.*/\1/p' "$root/tests/port/emit/d2-structs.txt" | sed -n 1p)
ask() {
    echo "== $*"
    "$edit" plan "$@"
    echo "exit $?"
}
ok=$root/tests/port/emit/d2-structs.txt
for name in "src/$unit.bmb" "$unit.bmb" "deep/er/$unit.bmb" "src/$unit" "src/$unit.old.bmb" "src/.bmb" "src/.$unit" "src/$unit.bmb/" "src/$unit.bmb/." "src/./$unit.bmb" "src/../$unit.bmb" ".." "src/.." "" "/" "src/$unit.bmb.txt" "src/other.bmb" "src/a.bmb"; do
    ask base base "$name" "$ok"
done
ask base base src/base.bmb "$root/tests/port/incr/base.txt"
ask base base src/a.bmb "$root/tests/port/incr/base.txt"
ask base base src/empty.bmb "$root/tests/port/udiff/empty.txt"
ask base base src/text.bmb "$root/tests/port/udiff/file.txt"
ask base base src/x.bmb "$root/tests/port/parser/err-items.txt"
ask base base src/x.bmb "$root/tests/port/parser/err-items.txt" "src/$unit.bmb" "$ok" src/wrong.bmb "$ok"
ask base base src/wrong.bmb "$ok" src/x.bmb "$root/tests/port/parser/err-items.txt"
for bad in "$root"/tests/check/err/data_types.bmb "$root"/tests/check/err/data_values.bmb "$root"/tests/port/types/whole.txt; do
    if [ -f "$bad" ]; then
        name=$(sed -n 's/^mod \([a-z_0-9]*\) {.*/\1/p' "$bad" | sed -n 1p)
        ask base base "src/$name.bmb" "$bad"
        ask none none "src/$name.bmb" "$bad"
    fi
done
ask base base src/std.bmb "$root/lib/std/num.bmb"
ask base base src/num.bmb "$root/lib/std/num.bmb"
# A workspace before that does not parse, and one that does not check.
mkdir -p broken/src unchecked/src
cp "$root/tests/port/parser/err-items.txt" broken/src/x.bmb
cp "$root/tests/check/err/data_types.bmb" unchecked/src/shapes.bmb
ask broken base
ask unchecked base
ask base broken
ask base unchecked
# Workspaces that cannot be read.
mkdir file link held inside inside/src
: > file/src
ln -s ../base/src link/src
mkdir held/src
ln -s ../../base/src/a.bmb held/src/a.bmb
mkdir inside/src/dir.bmb
cp base/src/a.bmb inside/src/a.bmb
cp base/src/a.bmb inside/src/notes.txt
cp base/src/a.bmb "inside/src/.bmb"
for ws in file link held inside nowhere; do
    ask "$ws" base
    ask base "$ws"
done
rm -rf base none broken unchecked file link held inside
