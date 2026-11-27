#!/bin/sh
# The proofs of L1 through the caches of `--cache`: L1 from an empty cache
# directory, the whole map from the caches that run left (every proof is
# found again, the rows are the same), then verify over them with its
# stats, which shows what the map stored. The cache file is masked on its
# second line, which names the compiler build.
#   cache.sh BMB FILE..
bmb=$1
shift
"$bmb" map --cache cache L1 "$@"
echo "exit $?"
"$bmb" map --cache cache "$@"
echo "exit $?"
"$bmb" verify --cache cache --stats after.stats "$@" > /dev/null
echo "exit $?"
sed -i '2s/.*/id -/' cache/defs 2> /dev/null
