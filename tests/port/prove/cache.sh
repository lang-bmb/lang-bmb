#!/bin/sh
# Proves the files three times over one proof cache in the working
# directory: from an empty cache, from the cache that run left (every proof
# is a hit and the scripts are those of the run that stored them), and with
# the search within small bounds, whose queries have other keys.
#   cache.sh BMB FILE..
bmb=$1
shift
"$bmb" verify --cache cache --emit-smt cold --stats cold.stats "$@"
echo "exit $?"
cp cache/proofs proofs.cold 2> /dev/null
"$bmb" verify --audit --cache cache --emit-smt warm --stats warm.stats "$@"
echo "exit $?"
"$bmb" verify --bounded-search --cache cache --emit-smt bounded --stats bounded.stats "$@"
echo "exit $?"
