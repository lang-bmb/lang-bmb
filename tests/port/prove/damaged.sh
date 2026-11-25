#!/bin/sh
# Proves the files over a proof cache whose file was damaged between two
# runs: a line with stage bits above 3, a line in the form Rust reads too
# (a `+` before a number, words after the fourth, a carriage return at the
# end), a key that is no SHA-256, a number that is none, a line cut short;
# every fifth line is left as it was. Then over the same file under the
# header of an older version, which is an empty cache, and over the file of
# the warm run with a carriage return before each line feed and after its
# last line, where no line feed follows (Rust's lines keeps that one, so the
# last line is not read).
#   damaged.sh BMB FILE..
bmb=$1
shift
"$bmb" verify --cache cache --emit-smt cold "$@" > /dev/null
awk 'NR == 1 { print; next }
  NR % 6 == 0 { print $1, $2, $3, "7"; next }
  NR % 6 == 1 { printf "%s +%s %s %s more words\r\n", $1, $2, $3, $4; next }
  NR % 6 == 2 { print "md5:" $1, $2, $3, $4; next }
  NR % 6 == 3 { print $1, "x" $2, $3, $4; next }
  NR % 6 == 4 { print $1, $2; next }
  { print }' cache/proofs > damaged
cp damaged cache/proofs
"$bmb" verify --cache cache --emit-smt warm --stats warm.stats "$@"
echo "exit $?"
cp cache/proofs proofs.warm
sed '1s/v5/v4/' damaged > cache/proofs
"$bmb" verify --cache cache --emit-smt older --stats older.stats "$@"
echo "exit $?"
sed 's/$/\r/' proofs.warm | head -c -1 > cache/proofs
"$bmb" verify --cache cache --emit-smt crlf --stats crlf.stats "$@"
echo "exit $?"
