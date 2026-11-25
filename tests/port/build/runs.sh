#!/bin/sh
# Builds FILE with the proof gate (no `--all-checks`): without a cache, then
# cold and warm over one cache directory. Each run's exit code and stats are
# left behind; a program that was built is run (its output and exit code)
# and removed, and the program the cache built is compared with the one
# built without it. The cache file is masked on its second line, which
# names the compiler build, as `id -`.
#   runs.sh BMB FILE
bmb=$1
cp "$2" p.bmb
"$bmb" build --stats plain.stats p.bmb -o prog
echo "exit $?"
if [ -f prog ]; then
  ./prog < /dev/null
  echo "program exit $?"
fi
"$bmb" build --cache cache --stats cold.stats p.bmb -o cold > /dev/null
echo "exit $?"
"$bmb" build --cache cache --stats warm.stats p.bmb -o warm > /dev/null
echo "exit $?"
for f in cold warm; do
  if [ -f $f ]; then
    cmp -s prog $f && echo "$f: the same program" || echo "$f: another program"
    rm -f $f
  fi
done
rm -f prog
sed -i '2s/.*/id -/' cache/defs 2> /dev/null
