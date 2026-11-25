#!/bin/sh
# Proves the programs of this directory in turn as one file, `edit.bmb`, over
# one cache directory, so that each run reads what the one before left:
# the base cold and warm, its layout changed, its definitions renamed, two
# moved to another unit, the base again, a body changed, a contract changed,
# a callee's contract changed, a revealed spec fn changed, a field added to a
# type, the base with --audit, with --def (a lazy load, the keys of the named
# definitions alone), with the search within small bounds (other keys), and
# the lint's first verification through the same cache. After each run the
# cache file is copied with its second line, which names the compiler
# build, as `id -`; the file left in the cache is masked the same way.
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
  run verify --cache cache --stats "$n.$step.stats" edit.bmb
done
cp "$dir/base.txt" edit.bmb
run verify --audit --cache cache --stats audit.stats edit.bmb
cp "$dir/spec.txt" edit.bmb
run verify --def a::twice --def b::width --cache cache --stats def.stats edit.bmb
run verify --def a::inc --cache cache --stats def-warm.stats edit.bmb
run verify --bounded-search --cache cache --stats bounded.stats edit.bmb
cp "$dir/callee.txt" edit.bmb
run lint --cache cache --select W0902,W0905 edit.bmb
run lint --cache cache --select W0902,W0905 edit.bmb
sed -i '2s/.*/id -/' cache/defs
