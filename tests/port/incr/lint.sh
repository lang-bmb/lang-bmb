#!/bin/sh
# The lint's first verification through the per-definition cache: the four
# satisfiability checks from an empty cache, the two that need the first
# verification from the cache that run left (every proof it stored is
# reused), then verify over it with its stats. The cache file is masked on
# its second line, which names the compiler build.
#   lint.sh BMB FILE..
bmb=$1
shift
"$bmb" lint --cache cache --select W0901,W0902,W0905,W0918 "$@"
echo "exit $?"
"$bmb" lint --cache cache --select W0902,W0905 "$@"
echo "exit $?"
"$bmb" verify --cache cache --stats after.stats "$@" > /dev/null
echo "exit $?"
sed -i '2s/.*/id -/' cache/defs 2> /dev/null
