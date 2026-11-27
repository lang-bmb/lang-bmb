#!/bin/sh
# The ends of `apply`, on a workspace made of three corpus programs copied
# as they are into `ws/src` (tests/run/method_order.bmb, tests/run/consts.bmb
# and tests/verify/pass/structs.bmb: together they have two `main`s, so the
# workspace does not check until a transaction removes one). After each
# transaction: what it printed, its exit code, and every file of the
# workspace with its digest.
# - how the arguments are read: an operations file that is missing, not
#   UTF-8 or not a JSON array of operations (every message of the reader),
#   standard input in its place, an empty transaction;
# - the directory: one that is missing (made by the commit), under a file,
#   with `src` a file, a link or a dangling link, with a linked, unreadable
#   or non-UTF-8 source file, with a syntax error; files that are not
#   sources stay, a source file no unit is named after goes, a link planted
#   where a temporary file is written is replaced and not followed;
# - each operation, alone and several in one transaction: `unit` (new,
#   nested, a new summary, no parent, a path that is none), `put` (each kind
#   of item, merged `impl` functions, code that does not parse, that holds
#   a unit, that closes the unit, that is empty, an empty `impl`, a `use`,
#   a missing unit), `remove` (each kind of address and each refusal),
#   `remove_unit`, `rename` (a function with callers and `#[uses]`, a type,
#   an `impl` function called as a method, a trait, a trait's method, a
#   unit, and each refusal: a keyword, a path, a short path, an unknown
#   name, an `impl`'s function of a trait method, a name that captures, a
#   unit that exists), `move` (and each refusal), `expect` (met, not met,
#   unknown, on a workspace that does not check), the standard library;
# - a commit rejected as a whole: by a type error with the operations that
#   were applied, by a failed operation beside ones that would apply;
# - the other commands on a directory in place of files: the workspace
#   (`src`), a directory of sources without `src`, a directory beside a
#   file, an empty one, one whose `src` is a link;
# - `import`: a program into a new workspace and over an old one (the files
#   of units that are gone go), a workspace into another, a program that
#   does not check, a file that cannot be read, a directory that cannot be
#   written.
#   ends.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
root=$here/../../..
st() {
  (cd "$1" 2> /dev/null && find . | sort | while read -r f; do
    if [ -f "$f" ] && [ ! -L "$f" ]; then sha256sum "$f"; else echo "$f"; fi
  done)
}
at() {
  dir=$1
  shift
  printf '%s' "$1" > ops.json
  "$bmb" apply "$dir" ops.json
  echo "exit $?"
  st "$dir"
}
tx() {
  at ws "$1"
}
mkdir -p ws/src
cp "$root/tests/run/method_order.bmb" "$root/tests/run/consts.bmb" "$root/tests/verify/pass/structs.bmb" ws/src/
echo note > ws/src/notes.txt
echo 'not a program' > ws/src/.bmb
echo outside > outside.txt

echo '== arguments'
"$bmb" apply ws absent.json
echo "exit $?"
printf '[\377]' > raw.json
"$bmb" apply ws raw.json
echo "exit $?"
printf '[\377]' | "$bmb" apply ws -
echo "exit $?"
printf '[{"op":"explode"}]' | "$bmb" apply ws -
echo "exit $?"
for bad in '' '[' '{"op":"put"}' '"put"' '7' 'null' 'true' '[1]' '[[]]' '[{"op":1}]' '[{}]' \
  '[{"op":"explode"}]' '[{"op":"unit"}]' '[{"op":"unit","path":"a"}]' '[{"op":"unit","path":"a","summary":2}]' \
  '[{"op":"put","code":"x"}]' '[{"op":"put","unit":"a"}]' '[{"op":"remove"}]' '[{"op":"remove_unit","def":"a"}]' \
  '[{"op":"rename","def":"a::f"}]' '[{"op":"rename","to":"g"}]' '[{"op":"expect","def":"a::f"}]' '[{"op":"move","def":"a::f"}]' \
  '[{"op":"unit","path":"a","summary":"s"},{"op":"move","to":"b"}]' '[] x' '[{"op":"unit","path":"a","summary":"\ud800"}]' \
  '[{"op":"unit","path":"a","summary":"\u00"}]' '[{"op":"unit" "path":"a"}]' '[{"op":"unit","op":"put","unit":"a","code":"c"}]'; do
  tx "$bad" | head -3
done
echo '== a workspace that does not check'
tx '[]'
tx '[{"op":"expect","def":"m::P","id":"sha256:0"}]'
tx '[{"op":"rename","def":"m::P","to":"Q"},{"op":"move","def":"geo::pick","to":"user"}]'
tx '[{"op":"remove","def":"konst::main"}]'
"$bmb" hash ws > ids.jsonl
echo "exit $?"
id_of() {
  grep "\"def\":\"$1\"" ids.jsonl | sed 's/.*"id":"\([^"]*\)".*/\1/'
}
echo '== expect'
tx "[{\"op\":\"expect\",\"def\":\"m::P\",\"id\":\"$(id_of m::P)\"},{\"op\":\"expect\",\"def\":\"geo::pick\",\"id\":\"$(id_of geo::pick)\"},{\"op\":\"expect\",\"def\":\"m::T\",\"id\":\"$(id_of m::P)\"}]"
tx "[{\"op\":\"expect\",\"def\":\"m::P::f\",\"id\":\"$(id_of m::P::f)\"},{\"op\":\"expect\",\"def\":\"<m::P as m::T>::f\",\"id\":\"$(id_of '<m::P as m::T>::f')\"}]"
tx '[{"op":"expect","def":"m::nope","id":"sha256:0"},{"op":"expect","def":"std::num::push_u64","id":"x"},{"op":"expect","def":"","id":""}]'
tx "[{\"op\":\"put\",\"unit\":\"geo\",\"code\":\"spec fn small_d(d: i64) -> bool { -10 <= d && d <= 10 }\"},{\"op\":\"expect\",\"def\":\"geo::small_d\",\"id\":\"$(id_of geo::small_d)\"}]"
tx "[{\"op\":\"expect\",\"def\":\"geo::small_d\",\"id\":\"$(id_of geo::small_d)\"}]"
echo '== unit'
tx '[{"op":"unit","path":"extra","summary":"A new unit"},{"op":"unit","path":"extra::inner","summary":"Inside \"it\""},{"op":"unit","path":"extra::inner::deep","summary":""}]'
tx '[{"op":"unit","path":"extra","summary":"Another summary\nof two lines"},{"op":"unit","path":"extra::inner","summary":"é"}]'
tx '[{"op":"unit","path":"no::such","summary":"s"},{"op":"unit","path":"extra::no::such","summary":"s"}]'
tx '[{"op":"unit","path":"../evil","summary":"s"},{"op":"unit","path":"fn","summary":"s"},{"op":"unit","path":"","summary":"s"},{"op":"unit","path":"extra::","summary":"s"},{"op":"unit","path":"a:::b","summary":"s"},{"op":"unit","path":"9a","summary":"s"},{"op":"unit","path":"é","summary":"s"}]'
tx '[{"op":"unit","path":"std","summary":"s"},{"op":"unit","path":"std::mine","summary":"s"},{"op":"put","unit":"std::num","code":"fn f() -> () {}"},{"op":"remove","def":"std::num::push_u64"},{"op":"remove","def":"<<std::x as y>"},{"op":"remove_unit","path":"std"},{"op":"rename","def":"std::num","to":"n"},{"op":"move","def":"std::num::push_u64","to":"extra"},{"op":"move","def":"geo::pick","to":"std::num"},{"op":"move","def":"geo::pick","to":"std"}]'
tx '[{"op":"unit","path":"stdx","summary":"Not the library"},{"op":"remove_unit","path":"stdx"}]'
echo '== put'
tx '[{"op":"put","unit":"extra","code":"#[effects(pure)]\nfn one() -> i64 {\n    1\n}"},{"op":"put","unit":"extra::inner","code":"#[effects(pure)] fn two() -> i64 { extra::one() + 1 }\n\nstruct Pair { a: i64, b: i64 }\n\nconst LIMIT: i64 = 10;"}]'
tx '[{"op":"put","unit":"extra","code":"#[effects(pure)]\nfn one() -> i64 { 2 }\n#[effects(pure)]\nfn one() -> i64 { 3 }"}]'
tx '[{"op":"put","unit":"extra::inner","code":"impl Pair { #[effects(pure)] fn sum(self: &Pair) -> i64 { self.a.saturating_add(self.b) } #[effects(pure)] fn first(self: &Pair) -> i64 { self.a } }"}]'
tx '[{"op":"put","unit":"extra::inner","code":"impl Pair { #[effects(pure)] fn first(self: &Pair) -> i64 { self.b } #[effects(pure)] fn new(a: i64) -> Pair { Pair { a: a, b: a } } }\n\nstruct Pair { a: i64, b: i64, c: i64 }"}]'
tx '[{"op":"put","unit":"extra::inner","code":"impl Pair { #[effects(pure)] fn first(self: &Pair) -> i64 { self.b } #[effects(pure)] fn new(a: i64) -> Pair { Pair { a: a, b: a, c: a } } }\n\nstruct Pair { a: i64, b: i64, c: i64 }"}]'
tx '[{"op":"put","unit":"extra::inner","code":"const LIMIT: i64 = 11;\n\ntrait Sized2 { #[effects(pure)] fn size(self: &Self) -> i64; }\n\nimpl Sized2 for Pair { #[effects(pure)] fn size(self: &Pair) -> i64 { 2 } }\n\nimpl m::T for Pair { #[effects(pure)] fn f(self: &Pair) -> i64 { 1 } #[effects(pure)] fn g(self: &Pair) -> i64 { 2 } #[effects(pure)] fn len(self: &Pair) -> i64 { LIMIT } }"}]'
tx '[{"op":"put","unit":"extra::inner","code":"impl Sized2 for Pair { #[effects(pure)] fn size(self: &Pair) -> i64 { 3 } }\n\ntrait Sized2 { #[effects(pure)] fn size(self: &Self) -> i64; #[effects(pure)] fn twice(self: &Self) -> i64 { self.size().saturating_mul(2) } }"}]'
tx '[{"op":"put","unit":"extra","code":"fn broken( -> i64 { 1 }"},{"op":"put","unit":"extra","code":"\n\n  fn also(x: i64) -> i64 { x +  }"}]'
tx '[{"op":"put","unit":"extra","code":"mod nested { fn f() -> () {} }"}]'
tx '[{"op":"put","unit":"extra","code":"fn f() -> () {} } #[summary(\"s\")] mod evil { fn g() -> () {}"}]'
tx '[{"op":"put","unit":"extra","code":"}"},{"op":"put","unit":"extra","code":""},{"op":"put","unit":"extra","code":"// only a comment"},{"op":"put","unit":"extra","code":"impl Gone { }"},{"op":"put","unit":"extra","code":"fn broken( impl Gone { }"}]'
tx '[{"op":"put","unit":"nowhere","code":"fn f() -> () {}"},{"op":"put","unit":"extra::nowhere","code":"fn f() -> () {}"},{"op":"put","unit":"a b","code":"fn f() -> () {}"},{"op":"put","unit":"extra","code":"use extra::inner::Pair;"}]'
tx '[{"op":"put","unit":"extra","code":"#[effects(pure)]\nfn three() -> i64 {\n    extra::inner::two() + nope()\n}"},{"op":"put","unit":"extra","code":"#[effects(pure)]\nfn four() -> bool {\n    4\n}"}]'
tx '[{"op":"put","unit":"extra","code":"struct one { x: i64 }"}]'
tx '[{"op":"put","unit":"extra","code":"#[effects(pure)]\nfn sized(p: extra::inner::Pair) -> i64 { p.size().saturating_add(p.sum()).saturating_add(extra::inner::Pair::first(&p)).saturating_add(m::T::len(&p)) }\n\n#[effects(pure)]\nfn zeros() -> [u8; konst::SIZE] { [0; konst::SIZE] }"}]'
echo '== remove'
tx '[{"op":"remove","def":"extra::one"}]'
tx '[{"op":"remove","def":"extra::inner::Pair::sum"}]'
tx '[{"op":"remove","def":"extra::sized"},{"op":"remove","def":"extra::inner::Pair::sum"}]'
tx '[{"op":"remove","def":"extra::inner::Pair::first"}]'
tx '[{"op":"remove","def":"extra::inner::Pair::new"}]'
tx '[{"op":"remove","def":"extra::inner::Sized2::size"},{"op":"remove","def":"<extra::inner::Pair as extra::inner::Sized2>::size"},{"op":"remove","def":"<extra::inner::Pair as extra::inner::Nope>"},{"op":"remove","def":"<extra::inner::Pair"},{"op":"remove","def":"<"}]'
tx '[{"op":"remove","def":"<extra::inner::Pair as extra::inner::Sized2>"},{"op":"remove","def":"extra::inner::Sized2"},{"op":"remove","def":"<extra::inner::Pair as m::T>"},{"op":"remove","def":"extra::inner::LIMIT"}]'
tx '[{"op":"remove","def":"extra::inner"},{"op":"remove","def":"extra::nope"},{"op":"remove","def":"nope::f"},{"op":"remove","def":"one"},{"op":"remove","def":"extra::inner::Pair::nope"},{"op":"remove","def":"a::b::c::d"},{"op":"remove","def":"extra::fn"}]'
tx '[{"op":"remove","def":"geo::Point"}]'
tx '[{"op":"remove","def":"extra::inner::Pair"},{"op":"remove","def":"extra::zeros"},{"op":"remove","def":"extra::inner::two"}]'
tx '[{"op":"remove","def":"konst::SIZE"}]'
echo '== remove_unit'
tx '[{"op":"remove_unit","path":"extra::inner::deep"},{"op":"remove_unit","path":"extra::inner::deep"},{"op":"remove_unit","path":"nope"},{"op":"remove_unit","path":"nope::deep"},{"op":"remove_unit","path":"a-b"}]'
tx '[{"op":"remove_unit","path":"extra::inner::deep"}]'
tx '[{"op":"remove_unit","path":"geo"}]'
tx '[{"op":"remove_unit","path":"speclit"},{"op":"unit","path":"speclit","summary":"Again"},{"op":"put","unit":"speclit","code":"#[effects(pure)] fn kept() -> i64 { 0 }"}]'
echo '== rename'
tx '[{"op":"rename","def":"geo::small","to":"tiny"}]'
tx '[{"op":"rename","def":"geo::Point","to":"Pt"},{"op":"rename","def":"geo::Pt::sum","to":"total"}]'
tx '[{"op":"rename","def":"m::P::f","to":"own"}]'
tx '[{"op":"rename","def":"m::T","to":"Tr"},{"op":"rename","def":"m::Tr::len","to":"length"}]'
tx '[{"op":"rename","def":"<m::P as m::Tr>::g","to":"h"},{"op":"rename","def":"m::P::g","to":"h"},{"op":"rename","def":"<u64 as m::Tr>::g","to":"h"}]'
tx '[{"op":"rename","def":"m::Tr::g","to":"h"},{"op":"rename","def":"m::W::wrapping_add","to":"plus"}]'
tx '[{"op":"rename","def":"m::Tr::h","to":"f"}]'
tx '[{"op":"rename","def":"konst::SIZE","to":"N"}]'
tx '[{"op":"rename","def":"konst","to":"k"}]'
tx '[{"op":"rename","def":"konst::masked","to":"low"},{"op":"rename","def":"other","to":"another"},{"op":"rename","def":"another::show","to":"print"}]'
tx '[{"op":"rename","def":"geo::tiny","to":"fn"},{"op":"rename","def":"geo::tiny","to":"a::b"},{"op":"rename","def":"geo::tiny","to":""},{"op":"rename","def":"tiny","to":"t"},{"op":"rename","def":"geo::nope","to":"t"},{"op":"rename","def":"geo::Pt::nope","to":"t"},{"op":"rename","def":"geo::","to":"t"},{"op":"rename","def":"print_i64","to":"t"},{"op":"rename","def":"u64","to":"t"}]'
tx '[{"op":"rename","def":"geo::tiny","to":"small_d"}]'
tx '[{"op":"rename","def":"geo::Pt","to":"Seg"}]'
tx '[{"op":"rename","def":"arrs::Point","to":"Cell"},{"op":"rename","def":"user","to":"users"}]'
tx '[{"op":"rename","def":"user","to":"geo"},{"op":"rename","def":"geo","to":"geo"},{"op":"rename","def":"extra::inner","to":"inner"}]'
tx '[{"op":"rename","def":"user","to":"users"},{"op":"rename","def":"extra::inner","to":"within"},{"op":"rename","def":"extra::nope","to":"x"}]'
tx '[{"op":"rename","def":"user","to":"users"},{"op":"rename","def":"extra::inner","to":"within"}]'
tx '[{"op":"put","unit":"extra","code":"#[effects(pure)] fn bad() -> bool { 1 }"},{"op":"rename","def":"geo::tiny","to":"little"},{"op":"move","def":"geo::tiny","to":"extra"}]'
tx '[{"op":"put","unit":"speclit","code":"#[effects(pure)] fn caller() -> i64 { kept() }\n\nspec fn sp(x: i64) -> bool { x == 0 }\n\n#[effects(pure)]\n#[uses(sp)]\nfn guarded(x: i64) -> i64\n    requires sp(x),\n{ x }"},{"op":"rename","def":"speclit::kept","to":"held"},{"op":"rename","def":"speclit::held","to":"kept2"},{"op":"rename","def":"speclit::sp","to":"zero"}]'
tx '[{"op":"put","unit":"users","code":"#[effects(io)]\nfn show() -> () {\n    print_i64(1);\n}\n\n#[effects(io)]\nfn emit(v: i64) -> () {}"}]'
tx '[{"op":"rename","def":"users::emit","to":"print_i64"}]'
tx '[{"op":"put","unit":"users","code":"#[effects(pure)]\nfn kept2() -> i64 { 7 }"},{"op":"rename","def":"speclit::caller","to":"kept2"}]'
echo '== move'
tx '[{"op":"move","def":"geo::pick","to":"users"}]'
tx '[{"op":"move","def":"geo::tiny","to":"extra::within"},{"op":"move","def":"speclit::caller","to":"extra"},{"op":"move","def":"speclit::zero","to":"extra"}]'
tx '[{"op":"move","def":"geo::tiny","to":"extra::within"},{"op":"move","def":"speclit::kept2","to":"extra"}]'
tx '[{"op":"move","def":"geo::count","to":"nowhere"},{"op":"move","def":"geo::count","to":"geo"},{"op":"move","def":"geo::Pt","to":"users"},{"op":"move","def":"geo::Pt::total","to":"users"},{"op":"move","def":"<m::P as m::Tr>::h","to":"users"},{"op":"move","def":"m::Tr::h","to":"users"},{"op":"move","def":"count","to":"users"},{"op":"move","def":"geo::nope","to":"users"},{"op":"move","def":"geo::count","to":"a b"},{"op":"move","def":"geo::count","to":""},{"op":"move","def":"print_i64","to":"users"}]'
tx '[{"op":"put","unit":"users","code":"#[effects(pure)] fn count() -> i64 { 0 }"},{"op":"move","def":"geo::count","to":"users"}]'
tx '[{"op":"move","def":"konst::low","to":"another"}]'
tx '[{"op":"put","unit":"extra","code":"#[effects(io)]\nfn print_i64(v: i64) -> () {}"},{"op":"move","def":"users::show","to":"extra"}]'
tx '[{"op":"unit","path":"fresh","summary":"Made in the transaction"},{"op":"move","def":"geo::count","to":"fresh"},{"op":"rename","def":"fresh::count","to":"tally"},{"op":"move","def":"fresh::tally","to":"geo"},{"op":"remove_unit","path":"fresh"}]'
echo '== the directory'
ln -s ../../outside.txt ws/src/.geo.bmb.tmp
echo 'kept too' > ws/src/a.b.txt
printf '#[summary("Named after no unit")]\nmod loose {\n    #[effects(pure)]\n    fn f() -> i64 { 1 }\n}\n' > ws/src/x.y.bmb
tx '[{"op":"put","unit":"geo","code":"#[effects(pure)] fn added() -> i64 { 1 }"}]'
cat outside.txt
printf '#[summary("t")]\nmod m { fn f( }\n' > ws/src/zz.bmb
tx '[{"op":"put","unit":"geo","code":"#[effects(pure)] fn more() -> i64 { 2 }"},{"op":"expect","def":"geo::added","id":"x"},{"op":"rename","def":"geo::added","to":"b"}]'
rm ws/src/zz.bmb
mkdir ws/src/dir.bmb
tx '[]'
rmdir ws/src/dir.bmb
printf 'mod \377' > ws/src/raw.bmb
tx '[]'
rm ws/src/raw.bmb
ln -s ../../outside.txt ws/src/link.bmb
tx '[]'
rm ws/src/link.bmb
at missing/deep '[{"op":"unit","path":"first","summary":"The first unit"}]'
at missing/deep/ '[{"op":"unit","path":"second","summary":"The second unit"}]'
at outside.txt/ws '[]'
at empty '[]'
mkdir -p filed linked dangling plain
echo x > filed/src
at filed '[]'
ln -s ../ws/src linked/src
at linked '[]'
ln -s nowhere dangling/src
at dangling '[]'
at "" '[{"op":"unit","path":"here","summary":"In the directory of the run"}]'
ls src
echo '== other commands on a directory'
cp "$root/tests/run/consts.bmb" "$root/tests/check/ok/search.bmb" plain/
run() {
  "$bmb" "$@"
  echo "exit $?"
}
run check ws
run check plain
run check missing/deep "$root/tests/run/swap.bmb"
run check empty
run check linked
run check dangling
run check filed
run fmt --check ws
run fmt missing/deep/
run hash missing/deep
run dump parse plain
run map L0 ws
run query units -- plain missing/deep
run verify --def geo::added ws
mkdir plain/src
run check plain
ln -s ../../outside.txt plain/src/l.bmb
run check plain
echo '== import'
imp() {
  dir=$1
  "$bmb" import "$@"
  echo "exit $?"
  st "$dir"
}
imp twice "$root/tests/run/method_order.bmb" "$root/tests/run/consts.bmb" "$root/tests/verify/pass/structs.bmb"
imp imported "$root/tests/verify/pass/structs.bmb" "$root/tests/run/method_order.bmb"
imp imported "$root/tests/run/consts.bmb"
imp copy ws
imp copy ws "$root/tests/run/swap.bmb"
imp copy absent.bmb
imp linked "$root/tests/run/swap.bmb"
imp outside.txt "$root/tests/run/swap.bmb"
imp "" imported
ls src
rm dangling/src
