#!/bin/sh
# proved.sh BMB EDIT FILE: the compact view (EDIT `compact`) of what the
# compiler says of a real program once it is proved: the map L1 (each
# definition's surface with its proof status), the status lines and
# diagnostics of `verify`, and the answers of `query status` and
# `query surface`. BMB (stage0 on both sides) proves once through a cache
# of its own and writes the rows; the two sides are compared on the view.
bmb=$1
edit=$2
shift 2
"$bmb" map --cache proofs L1 "$@" > l1.jsonl
"$bmb" verify --cache proofs "$@" > verify.jsonl
"$bmb" query --cache proofs status -- "$@" > status.jsonl
"$bmb" query --cache proofs surface -- "$@" > surface.jsonl
"$edit" compact l1.jsonl verify.jsonl status.jsonl surface.jsonl
echo "exit $?"
rm -rf proofs
rm -f l1.jsonl verify.jsonl status.jsonl surface.jsonl
