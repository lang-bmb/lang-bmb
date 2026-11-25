#!/bin/sh
# Emits the programs of DIR (the per-definition cache's, tests/port/incr) in
# turn as one file, `edit.bmb`, over one cache directory, with the proof gate
# (no `--all-checks`), so that each run reads the IR the one before left:
# the base cold and warm, its layout changed, its definitions renamed, two
# moved to another unit, the base again, a body changed, a contract changed,
# a callee's contract changed, a revealed spec fn changed, a field added to a
# type, the base with --audit (every reused proof and fragment computed
# again and compared), then `build` of the base over the same cache. Each
# run leaves its IR and its stats; after each run the cache file is copied
# with its second line, which names the compiler build, as `id -`, and the
# file left in the cache is masked the same way.
#   edits.sh BMB DIR
bmb=$1
dir=$2
n=0
run() {
  n=$((n + 1))
  echo "== $n $*"
  "$bmb" "$@"
  echo "exit $?"
  sed '2s/.*/id -/' cache/defs > "defs.$n" 2> /dev/null
}
for step in base base layout rename moved base body contract callee spec type; do
  cp "$dir/$step.txt" edit.bmb
  run emit-llvm --cache cache --stats "$n.$step.stats" edit.bmb -o "$n.$step.ll"
done
cp "$dir/base.txt" edit.bmb
run emit-llvm --audit --cache cache --stats audit.stats edit.bmb -o audit.ll
run build --cache cache --stats build.stats edit.bmb -o prog
sed -i '2s/.*/id -/' cache/defs
