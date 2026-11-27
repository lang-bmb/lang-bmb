#!/bin/sh
# `weaken` between two versions of a program whose definitions can reach an
# escape (`assume`, `#[external_body]`), where the gate proves in several
# passes: `tests/port/prove/trust.bmb` and `tests/run/escapes.bmb`, each
# against itself with two of its postconditions rewritten by `sed` (one
# weaker, one stronger), in both directions.
#   escapes.sh BMB ROOT
bmb=$1
trust=${2}tests/port/prove/trust.bmb
escapes=${2}tests/run/escapes.bmb
sed 's/ensures r >= 1,/ensures r >= 0,/; s/ensures r >= 0 && r <= 10,/ensures r >= 0 \&\& r <= 9,/' "$trust" > trust2.bmb
sed 's/ensures sq_small(r),/ensures sq_small(r) \&\& r >= 0,/; s/ensures small(r),/ensures true,/' "$escapes" > escapes2.bmb
cmp -s "$trust" trust2.bmb && echo "trust: nothing rewritten"
cmp -s "$escapes" escapes2.bmb && echo "escapes: nothing rewritten"
"$bmb" weaken "$trust" trust2.bmb
echo "exit $?"
"$bmb" weaken trust2.bmb "$trust"
echo "exit $?"
"$bmb" weaken "$escapes" escapes2.bmb
echo "exit $?"
"$bmb" weaken escapes2.bmb "$escapes"
echo "exit $?"
