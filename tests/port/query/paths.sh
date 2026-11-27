#!/bin/sh
# The queries that take a path, on paths of the given program: its first
# unit, two of its definitions and the last of its types, as the L0 rows of
# the compiler under test list them (the rows themselves are the first
# output). Of a definition: its slice, what a change of its surface or body
# proves again, its edges (out, both ways two steps, in by kind, out by
# kind), what it reaches, who names it in a contract, the names found by
# its last segment, and scoped queries. Of a type: its surface, what a
# change proves again, its edges, and the queries that take it for a
# definition or a scope. Nothing is proved.
#   paths.sh BMB FILE..
bmb=$1
shift
rows=$("$bmb" query units -- "$@")
echo "exit $?"
printf '%s\n' "$rows"
ask() {
  q=$1
  p=$2
  shift 2
  "$bmb" query $q "$p" -- "$@"
  echo "exit $?"
}
unit=$(printf '%s\n' "$rows" | sed -n 's/^{"level":"L0","kind":"unit","path":"\([^"]*\)".*/\1/p' | sed -n 1p)
if [ -n "$unit" ]; then
  for q in units overview duplicates effects "effect io" "text let" edges; do
    ask "$q" "$unit" "$@"
  done
fi
printf '%s\n' "$rows" | sed -n -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"fn".*/\1/p' -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"spec fn".*/\1/p' -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"proof fn".*/\1/p' | sed -n '1p;6p' | while IFS= read -r def; do
  for q in slice impact effects contracts units "effect alloc" "text fn" edges; do
    ask "$q" "$def" "$@"
  done
  "$bmb" query impact "$def" body -- "$@"
  echo "exit $?"
  "$bmb" query edges "$def" both 2 -- "$@"
  echo "exit $?"
  "$bmb" query edges "$def" in 3 calls,uses,contract -- "$@"
  echo "exit $?"
  "$bmb" query edges "$def" out 9 types,spec_calls,writes,reads -- "$@"
  echo "exit $?"
  "$bmb" query find "${def##*::}" -- "$@"
  echo "exit $?"
done
printf '%s\n' "$rows" | sed -n -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"struct".*/\1/p' -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"enum".*/\1/p' -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"trait".*/\1/p' -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"const".*/\1/p' | sed -n '$p' | while IFS= read -r ty; do
  for q in surface impact units effects contracts slice "text :" edges; do
    ask "$q" "$ty" "$@"
  done
  "$bmb" query impact "$ty" body -- "$@"
  echo "exit $?"
  "$bmb" query edges "$ty" both 2 -- "$@"
  echo "exit $?"
  "$bmb" query edges "$ty" in 2 types -- "$@"
  echo "exit $?"
done
