#!/bin/sh
# Proves each file over a cache directory of its own: cold, warm, warm with
# --audit, with the search within small bounds (other keys), and with --def
# of the first definition the cold run gave a status (a lazy load). The
# stats of each run are left behind; the cache file is masked on its second
# line, which names the compiler build, as `id -`.
#   runs.sh BMB FILE..
bmb=$1
shift
"$bmb" verify --cache cache --stats cold.stats "$@" > cold.out
echo "exit $?"
cat cold.out
"$bmb" verify --cache cache --stats warm.stats "$@"
echo "exit $?"
"$bmb" verify --audit --cache cache --stats audit.stats "$@"
echo "exit $?"
"$bmb" verify --bounded-search --cache cache --stats bounded.stats "$@" > /dev/null
echo "exit $?"
first=$(sed -n 's/^{"def":"\([^"]*\)".*/\1/p' cold.out | head -n 1)
if [ -n "$first" ]; then
  "$bmb" verify --def "$first" --cache cache --stats def.stats "$@"
  echo "exit $?"
fi
sed -i '2s/.*/id -/' cache/defs 2> /dev/null
