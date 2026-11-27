#!/bin/sh
# edits.sh BMB EDIT ROOT: the files of a workspace replaced by other files
# as one transaction (EDIT `plan`: `relative_sources` of both workspaces,
# then `plan_files`). The workspaces are made by BMB (stage0 on both sides,
# `bmb import`) from the programs of ROOT/tests/port/incr, which are one
# program with one edit each: the base replaced by each edit (what changed
# and what was removed, the normal-form units to write), by itself, by
# nothing, and each edit replaced by the base.
bmb=$1
edit=$2
incr=$3/tests/port/incr
"$bmb" import base "$incr/base.txt"
mkdir none
for name in layout rename moved body contract callee spec type; do
    "$bmb" import "$name" "$incr/$name.txt"
    echo "== base to $name"
    "$edit" plan base "$name"
    echo "exit $?"
    echo "== $name to base"
    "$edit" plan "$name" base
    echo "exit $?"
done
echo "== base to base"
"$edit" plan base base
echo "== nothing to base"
"$edit" plan none base
echo "== base to nothing"
"$edit" plan base none
echo "== nothing to nothing"
"$edit" plan none none
rm -rf base none layout rename moved body contract callee spec type
