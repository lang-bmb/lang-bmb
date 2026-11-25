#!/bin/sh
# Proves the files over the cache directory CACHE, written by stage0, under
# the name of BMB's own build: the second line of its cache file is
# replaced by the one BMB writes (learned from a cache of its own over
# NAMER, a small program), so that BMB reads every entry. Every definition
# is reused; the rows, the stats and the cache file are compared, the
# file's second line masked as `id -`.
#   incr.sh BMB CACHE NAMER FILE..
bmb=$1
cache=$2
namer=$3
shift 3
"$bmb" verify --cache own "$namer" > /dev/null
mine=$(sed -n 2p own/defs)
rm -r own
mkdir cache
cp "$cache/proofs" cache/proofs
{ head -n 1 "$cache/defs"; echo "$mine"; tail -n +3 "$cache/defs"; } > cache/defs
"$bmb" verify --cache cache --stats stats "$@"
echo "exit $?"
sed -i '2s/.*/id -/' cache/defs
