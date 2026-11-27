#!/bin/sh
# The ends of `map` that are no whole map, and the window flags, on
# trust.bmb and shapes.bmb of prove/: counts the flags refuse (no digits, a
# sign, empty, above the largest usize) and take (a `+`, a leading zero), a
# flag given twice (the last is taken), every window end (an offset past the
# last row, no row asked for, a budget smaller than the first row, the last
# rows, the whole response), flags after the level and `--cache` after a
# window flag (the rest are files), a level that is none, no file, a file
# that cannot be read, a program that does not check (with a window too),
# L1 with no solver on PATH, and the caches of L1: written, found again,
# not made by a level that proves nothing, one whose proofs cannot be written and
# one under a path that is no directory. A cache file is masked on its
# second line, which names the compiler build.
#   ends.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
trust=$here/../prove/trust.bmb
shapes=$here/../prove/shapes.bmb
run() {
  "$bmb" map "$@"
  echo "exit $?"
}
run --limit x L0 "$trust"
run --budget -1 L0 "$trust"
run --offset '' L0 "$trust"
run --limit + L0 "$trust"
run --budget 18446744073709551616 L0 "$trust"
run --offset 2 --limit x L0 "$trust"
run --limit +2 L0 "$trust"
run --offset 02 --limit 03 L0 "$trust"
run --budget 18446744073709551615 L2 "$trust"
run --limit 1 --limit 2 --offset 9 --offset 1 L0 "$trust"
run --offset 1000 L0 "$trust"
run --limit 0 L0 "$trust"
run --budget 0 L2 "$shapes"
run --budget 1 --offset 2 --limit 3 L2 "$shapes"
run --budget 600 L0 "$shapes"
run --budget 600 --offset 3 L0 "$shapes"
run --offset 1 L0 "$trust"
run --limit 100000 L2 "$trust"
run --limit 5 "$trust"
run --budget 400 --offset 1 L0 "$trust" "$shapes"
run L0 --limit 2 "$trust"
run --limit 2 --cache made L0 "$trust"
run L3 "$trust"
run L0
run L2 "$here/absent.bmb"
printf '#[summary("t")]\nmod m { fn f( }\n' > bad.bmb
run L0 bad.bmb
run --limit 1 L2 bad.bmb
run bad.bmb
PATH=/nonexistent run L1 "$trust"
run --cache made L1 "$trust"
run --cache made --limit 4 "$trust"
run --cache unused L0 "$trust"
run --cache unused --offset 1 L2 "$trust"
mkdir -p held/proofs
run --cache held L1 "$trust"
run --cache /dev/null/cache L1 "$trust"
sed -i '2s/.*/id -/' made/defs held/defs 2> /dev/null
