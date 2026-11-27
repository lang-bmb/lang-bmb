#!/bin/sh
# A plain `lint` through the caches of `--cache DIR`: from an empty
# directory, from the caches that run left (the first verification reuses
# every definition and the proofs of the re-verifications are found), one
# check that proves again alone over them, then `verify` over them with its
# stats. The cache file is masked on its second line, which names the
# compiler build.
#   cache.sh BMB FILE..
bmb=$1
shift
"$bmb" lint --cache cache "$@"
echo "exit $?"
"$bmb" lint --cache cache "$@"
echo "exit $?"
"$bmb" lint --cache cache --select W0904 "$@"
echo "exit $?"
"$bmb" verify --cache cache --stats after.stats "$@" > /dev/null
echo "exit $?"
sed -i '2s/.*/id -/' cache/defs 2> /dev/null
