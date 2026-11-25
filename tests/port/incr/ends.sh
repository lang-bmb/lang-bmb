#!/bin/sh
# The ends of the per-definition cache that are no verdict, on base.txt
# beside this script: a cache directory that cannot be made (both caches
# say so; the verdict is the same), a `defs.tmp` that cannot be replaced, a
# `defs` that is a directory (read as an empty cache, not replaced),
# definitions that are not in the program after both caches were written,
# and the same for the lint's first verification.
#   ends.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
cp "$here/base.txt" p.bmb
run() {
  "$bmb" "$@" > /dev/null
  echo "exit $?"
}
run verify --cache /dev/null/cache p.bmb
mkdir -p tmp/defs.tmp/inside held/defs
run verify --cache tmp --stats tmp.stats p.bmb
run verify --cache held --stats held.stats p.bmb
run verify --def a::nope --def a::inc --cache fresh --stats fresh.stats p.bmb
run lint --cache /dev/null/cache --select W0902,W0905 p.bmb
run lint --cache tmp --select W0905 p.bmb
sed -i "2s/.*/id -/" fresh/defs held/defs.tmp
