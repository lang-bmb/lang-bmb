#!/bin/sh
# Proves the files with --emit-smt from a warm proof cache: the proof file
# of CACHE (made by the compiler under test, which both sides read, as the
# keys name no build), every proof a hit. The scripts, the statistics and
# the proof file left are compared.
#   warm.sh BMB CACHE FILE..
bmb=$1
cache=$2
shift 2
mkdir cache
cp "$cache/proofs" cache/proofs
"$bmb" verify --cache cache --emit-smt smt --stats stats "$@"
echo "exit $?"
