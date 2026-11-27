#!/bin/sh
# The queries that take no path and prove nothing, on the given files as
# one program: the L0 rows, the overview, the groups of equal identities,
# the effect views, the lines that hold a text (one in most definitions, one
# with a blank, one that is in none), names found, and three windows.
#   each.sh BMB FILE..
bmb=$1
shift
ask() {
  q=$1
  shift
  "$bmb" query $q -- "$@"
  echo "exit $?"
}
ask units "$@"
ask overview "$@"
ask duplicates "$@"
ask effects "$@"
ask "effect alloc" "$@"
ask "effect io" "$@"
ask "effect nondet" "$@"
ask "effect panic" "$@"
ask "text fn" "$@"
"$bmb" query text ': u' -- "$@"
echo "exit $?"
ask "text §" "$@"
ask "find new" "$@"
ask "find main" "$@"
ask "--budget 1500 --offset 1 --limit 12 units" "$@"
ask "--offset 2 text (" "$@"
ask "--limit 3 --budget 100 effect alloc" "$@"
