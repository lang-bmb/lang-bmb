#!/bin/sh
# The queries that show proof results: `status` of the whole program
# without a cache, then `tests` and `status` through the caches of
# `--cache` (an empty directory first, so the second finds every proof
# again), then of the first unit, of two definitions and of a type alone
# (a scope proves only its definitions, a type none), and `surface` of
# those definitions with and without the caches. Verify over the caches
# with its stats shows what the queries stored. The cache file is masked
# on its second line, which names the compiler build.
#   proved.sh BMB FILE..
bmb=$1
shift
ask() {
  q=$1
  shift
  "$bmb" query $q -- "$@"
  echo "exit $?"
}
ask status "$@"
ask "--cache cache tests" "$@"
ask "--cache cache status" "$@"
rows=$("$bmb" query units -- "$@")
unit=$(printf '%s\n' "$rows" | sed -n 's/^{"level":"L0","kind":"unit","path":"\([^"]*\)".*/\1/p' | sed -n 1p)
if [ -n "$unit" ]; then
  "$bmb" query --cache scoped status "$unit" -- "$@"
  echo "exit $?"
  "$bmb" query tests "$unit" -- "$@"
  echo "exit $?"
fi
printf '%s\n' "$rows" | sed -n -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"fn".*/\1/p' -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"spec fn".*/\1/p' -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"proof fn".*/\1/p' | sed -n '2p;5p' | while IFS= read -r def; do
  "$bmb" query surface "$def" -- "$@"
  echo "exit $?"
  "$bmb" query --cache one surface "$def" -- "$@"
  echo "exit $?"
  "$bmb" query --cache one status "$def" -- "$@"
  echo "exit $?"
  "$bmb" query --cache cache tests "$def" -- "$@"
  echo "exit $?"
done
printf '%s\n' "$rows" | sed -n -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"struct".*/\1/p' -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"enum".*/\1/p' | sed -n 1p | while IFS= read -r ty; do
  "$bmb" query --cache typed status "$ty" -- "$@"
  echo "exit $?"
  "$bmb" query tests "$ty" -- "$@"
  echo "exit $?"
done
"$bmb" verify --cache cache --stats after.stats "$@" > /dev/null
echo "exit $?"
sed -i '2s/.*/id -/' cache/defs scoped/defs one/defs typed/defs 2> /dev/null
