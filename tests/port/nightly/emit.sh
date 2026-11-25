#!/bin/sh
# Emits the files as one program with the proof gate (no `--all-checks`)
# over a cache directory that holds the proof file of CACHE: every proof is
# a hit, every definition is lowered and stored. The status rows, the IR,
# the stats and the cache file are compared; the cache file's second line,
# which names the compiler build, is masked as `id -`.
#   emit.sh BMB CACHE FILE..
bmb=$1
cache=$2
shift 2
mkdir cache
cp "$cache/proofs" cache/proofs
"$bmb" emit-llvm --cache cache --stats stats "$@" -o out.ll
echo "exit $?"
sed -i '2s/.*/id -/' cache/defs 2> /dev/null
