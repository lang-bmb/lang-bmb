#!/bin/sh
# `fmt --check` holds each file to the units it declares, in name order,
# whatever the other files are and in whatever order they come: two files
# each in normal form (the first calls the second) in both orders, a file
# of two units in name order, a file whose two units are not in name order,
# an empty file before and after another, and a file not in normal form
# (one blank line too many) first and last among files that are. Then
# `fmt` of the two files in both orders, which prints the same text.
#   order.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
run() {
  "$bmb" fmt --check "$@"
  echo "exit $?"
}
run "$here/a.txt" "$here/b.txt"
run "$here/b.txt" "$here/a.txt"
run "$here/two.txt"
run "$here/swapped.txt"
: > empty.txt
run empty.txt "$here/two.txt"
run "$here/two.txt" empty.txt
run "$here/c.txt" "$here/a.txt" "$here/b.txt"
run "$here/b.txt" "$here/a.txt" "$here/c.txt"
"$bmb" fmt "$here/a.txt" "$here/b.txt"
echo "exit $?"
"$bmb" fmt "$here/b.txt" "$here/a.txt"
echo "exit $?"
