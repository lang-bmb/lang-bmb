#!/bin/sh
# `uses` on a copy of one program: the search without a cache (the report,
# nothing written), through the caches of `--cache DIR` from an empty
# directory, `--write` over the caches that run left, and once more over
# what was written. The cache file is masked on its second line, which
# names the compiler build.
#   write.sh BMB FILE
bmb=$1
cp "$2" in.bmb
"$bmb" uses in.bmb
echo "exit $?"
"$bmb" uses --cache cache in.bmb
echo "exit $?"
"$bmb" uses --write --cache cache in.bmb
echo "exit $?"
"$bmb" uses --cache cache --write in.bmb
echo "exit $?"
sed -i '2s/.*/id -/' cache/defs 2> /dev/null
