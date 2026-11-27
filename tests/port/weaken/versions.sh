#!/bin/sh
# `weaken` between two programs of the corpus that share their definitions'
# paths and differ in their contracts: a file of `tests/verify/pass` and the
# file of the same name in `tests/verify/fail`, in both directions. Nothing
# is run for a file without such a partner.
#   versions.sh BMB FILE
bmb=$1
pass=$2
fail=$(dirname "$(dirname "$pass")")/fail/$(basename "$pass")
[ -f "$fail" ] || exit 0
"$bmb" weaken "$pass" "$fail"
echo "exit $?"
"$bmb" weaken "$fail" "$pass"
echo "exit $?"
