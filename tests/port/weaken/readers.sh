#!/bin/sh
# `weaken --readers` (the reader's comparison): the readers of the corpus
# case `tests/weaken/readers` over its two versions and over the versions
# swapped, then reader files of its own making: a reader of a definition
# neither version has, one that names a removed definition by its old path,
# the same definition under two reveal sets (two rounds), `uses` left out
# and empty, blank lines and lines of white space, and each line the reader
# of the file refuses (no JSON, no `def`, a `def` that is no string, `uses`
# that is no array, `uses` that holds a number), a file that cannot be read
# and an empty one.
#   readers.sh BMB ROOT
bmb=$1
dir=${2}tests/weaken/readers
run() {
  "$bmb" weaken --readers "$@"
  echo "exit $?"
}
run "$dir/readers.jsonl" "$dir/old.bmb" "$dir/new.bmb"
run "$dir/readers.jsonl" "$dir/new.bmb" "$dir/old.bmb"
run "$dir/readers.jsonl" "$dir/old.bmb" "$dir/old.bmb"
printf '%s\n' '{"def":"m::dec","uses":[]}' '' '   ' '{"def":"m::dec","uses":["m::pos"]}' '{"def":"m::dec","uses":["m::pos","m::gt0","m::nope"]}' '{"def":"m::dec"}' '{"def":"m::nowhere","uses":["m::pos"]}' '{"uses":[],"def":"m::half","more":1}' > mine.jsonl
run mine.jsonl "$dir/old.bmb" "$dir/new.bmb"
run mine.jsonl "${2}tests/weaken/rename/old.bmb" "${2}tests/weaken/rename/new.bmb"
printf '%s\n' '{"def":"m::dec"}' 'not json' > bad1.jsonl
printf '%s\n' '{"def":"m::dec"}' '{"uses":[]}' > bad2.jsonl
printf '%s\n' '{"def":7}' > bad3.jsonl
printf '%s\n' '' '{"def":"m::dec","uses":"m::pos"}' > bad4.jsonl
printf '%s\n' '{"def":"m::dec","uses":["m::pos",7]}' > bad5.jsonl
printf '%s\n' '{"def":"m::dec","uses":null}' > bad6.jsonl
printf '%s\n' '["m::dec"]' > bad7.jsonl
for k in 1 2 3 4 5 6 7; do
  run bad$k.jsonl "$dir/old.bmb" "$dir/new.bmb"
done
: > none.jsonl
run none.jsonl "$dir/old.bmb" "$dir/new.bmb"
run absent.jsonl "$dir/old.bmb" "$dir/new.bmb"
run "$dir/readers.jsonl" absent.bmb "$dir/new.bmb"
