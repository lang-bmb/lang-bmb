#!/bin/sh
# each.sh BMB EDIT FILE..: the compact view (EDIT `compact`) of what the
# compiler says of a real program without proving anything: the maps L0 and
# L2 with their header, the diagnostics of `check`, the hashes, the answers
# of the queries that take no path, and the slice, edges and surface
# queries of its first and sixth definitions. BMB (stage0 on both sides)
# writes the rows; the two sides are compared on the view of them.
bmb=$1
edit=$2
shift 2
"$bmb" map L0 "$@" > l0.jsonl
"$bmb" map L2 "$@" > l2.jsonl
"$bmb" check "$@" > check.jsonl
"$bmb" hash "$@" > hash.jsonl
for q in units overview duplicates effects; do
    "$bmb" query $q -- "$@"
done > queries.jsonl
sed -n -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"fn".*/\1/p' -e 's/^{"level":"L0","kind":"def","path":"\([^"]*\)","def_kind":"spec fn".*/\1/p' l0.jsonl | sed -n '1p;6p' | while IFS= read -r def; do
    "$bmb" query slice "$def" -- "$@"
    "$bmb" query edges "$def" both 2 -- "$@"
    "$bmb" query impact "$def" -- "$@"
    "$bmb" query contracts "$def" -- "$@"
done > paths.jsonl
"$edit" compact l0.jsonl l2.jsonl check.jsonl hash.jsonl queries.jsonl paths.jsonl
echo "exit $?"
rm -f l0.jsonl l2.jsonl check.jsonl hash.jsonl queries.jsonl paths.jsonl
