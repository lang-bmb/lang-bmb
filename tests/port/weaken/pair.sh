#!/bin/sh
# `weaken` on one case of the corpus `tests/weaken/CASE/{old,new}.bmb`: the
# old version against the new one, the new against the old, and each
# version against itself (every definition matched by path with the same
# contract: no query).
#   pair.sh BMB OLD
# OLD is the case's `old.bmb`; the new version is the `new.bmb` beside it.
bmb=$1
old=$2
new=$(dirname "$old")/new.bmb
"$bmb" weaken "$old" "$new"
echo "exit $?"
"$bmb" weaken "$new" "$old"
echo "exit $?"
"$bmb" weaken "$old" "$old"
echo "exit $?"
"$bmb" weaken "$new" "$new"
echo "exit $?"
