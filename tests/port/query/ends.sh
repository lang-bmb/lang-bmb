#!/bin/sh
# The ends of `query`, on trust.bmb and shapes.bmb of prove/: how the
# arguments are read (no `--`, no query, no file, `--` twice, `--cache`
# after a window flag, a window flag that takes no count), every message of
# a query that cannot be answered (an unknown query, a missing or unknown
# path, direction, depth, edge kind, effect, text, change), the queries
# whose answer depends on what the first argument names (`effects` of one
# definition, of two and of a unit; `surface`, `impact` and `edges` of a
# type), a scope that holds no definition, windows on answers (also one
# that proves), a file that cannot be read, a program that does not check,
# an `assume` where the walk over a body for escapes does not go (in an
# argument of a call: the rows of `verify` and of `status` count none),
# `status` with no solver on PATH, and the caches: written, found again,
# not made by a query that proves nothing or cannot be answered, one whose
# proofs cannot be written and one under a path that is no directory. A
# cache file is masked on its second line, which names the compiler build.
#   ends.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
trust=$here/../prove/trust.bmb
shapes=$here/../prove/shapes.bmb
run() {
  "$bmb" query "$@"
  echo "exit $?"
}
run units "$trust"
run -- "$trust"
run overview --
run units -- "$trust" -- "$shapes"
run --limit x units -- "$trust"
run --budget -1 units -- "$trust"
run --offset 2 --limit '' units -- "$trust"
run --limit 2 --cache made units -- "$trust"
run --cache made --limit 2 --limit 3 --offset 1 units -- "$trust"
run nosuch -- "$trust"
run surface -- "$trust"
run surface trust::nope -- "$trust"
run slice -- "$trust"
run slice trust -- "$trust"
run impact -- "$trust"
run impact trust::top neither -- "$trust"
run impact trust::nope neither -- "$trust"
run impact shapes::Point neither -- "$shapes"
run impact shapes::Point -- "$shapes"
run impact shapes::Point body -- "$shapes"
run surface shapes::Shape -- "$shapes"
run edges -- "$trust"
run edges trust -- "$trust"
run edges trust::top sideways -- "$trust"
run edges trust::top out deep -- "$trust"
run edges trust::top out -1 -- "$trust"
run edges trust::top out +2 -- "$trust"
run edges trust::top out 18446744073709551615 -- "$trust"
run edges trust::top out 18446744073709551616 -- "$trust"
run edges trust::top out 0 -- "$trust"
run edges trust::top both 02 calls,calls -- "$trust"
run edges trust::top in 2 calls,called -- "$trust"
run edges trust::top out 1 '' -- "$trust"
run edges trust::top out 1 calls, -- "$trust"
run edges shapes::Point both 2 -- "$shapes"
run edges shapes::Point in 1 types extra -- "$shapes"
run find -- "$trust"
run find trust::top -- "$trust"
run find '' -- "$trust"
run find Point -- "$trust" "$shapes"
run contracts -- "$trust"
run contracts trust::small -- "$trust"
run contracts small -- "$trust"
run effect -- "$trust"
run effect loud -- "$trust"
run effect loud trust::nope -- "$trust"
run effect panic trust::nope -- "$trust"
run effect panic trust::top trust::mid trust::top -- "$trust"
run effect pure -- "$trust"
run effects trust::top -- "$trust"
run effects trust::top trust::mid -- "$trust"
run effects trust -- "$trust"
run effects trust::nope -- "$trust"
run effects shapes::Point -- "$shapes"
run text -- "$trust"
run text '' trust -- "$trust"
run text 'r >= 0 && r' -- "$trust"
run text assume trust::assumes trust::nope -- "$trust"
run text Point shapes::Point shapes::rect -- "$shapes"
run units trust::nope -- "$trust"
run units shapes::Point shapes -- "$shapes"
run overview shapes::Point -- "$shapes"
run duplicates trus -- "$trust"
run duplicates shapes::Point -- "$trust" "$shapes"
run --offset 1000 units -- "$trust"
run --limit 0 overview -- "$trust"
run --budget 300 --offset 1 edges trust::top both 3 -- "$trust"
run --budget 0 slice trust::above -- "$trust"
run --budget 900 --limit 4 status trust -- "$trust"
run --limit 1 nosuch -- "$trust"
run units -- "$here/absent.bmb"
printf '#[summary("t")]\nmod m { fn f( }\n' > bad.bmb
run units -- bad.bmb
run --limit 1 status -- bad.bmb
printf '#[summary("An assume in an argument of a call")]\nmod nested {\n    spec fn small(x: i64) -> bool {\n        x < 100\n    }\n\n    #[effects(panic)]\n    fn same(x: i64) -> (r: i64)\n        ensures r == x,\n    {\n        x\n    }\n\n    #[effects(panic)]\n    #[uses(small)]\n    fn through(x: i64) -> (r: i64)\n        ensures r < 100,\n    {\n        same({\n            #[rationale(kind = boundary, "the caller keeps x small")]\n            assume(small(x));\n            x\n        })\n    }\n}\n' > nested.bmb
"$bmb" verify nested.bmb
echo "exit $?"
run status -- nested.bmb
run tests nested::through -- nested.bmb
run status nope -- "$trust"
run tests trust::nope -- "$trust"
run status shapes::Point -- "$shapes"
run tests shapes::Point shapes::Shape -- "$shapes"
PATH=/nonexistent run status trust::top trust::alone -- "$trust"
PATH=/nonexistent run surface trust::alone -- "$trust"
run --cache made status -- "$trust"
run --cache made tests trust::top -- "$trust"
run --cache made surface trust::top -- "$trust"
run --cache unused overview -- "$trust"
run --cache unused status trust::nope -- "$trust"
run --cache unused surface trust::nope -- "$trust"
run --cache unused surface shapes::Point -- "$shapes"
run --cache typed status shapes::Point -- "$shapes"
mkdir -p held/proofs
run --cache held status trust::alone -- "$trust"
run --cache /dev/null/cache tests trust::alone -- "$trust"
ls made unused typed held 2>&1
sed -i '2s/.*/id -/' made/defs typed/defs held/defs 2> /dev/null
