#!/bin/sh
# The ends of `emit-llvm` and `build` with the proof gate that are no
# verdict of the program: a module, a stats file or an executable's `.ll`
# that cannot be written (a directory that does not exist), caches that
# cannot be written (a file where the cache directory would be), a program
# that does not check, `--cache` with nothing after it before `-o` (it is
# read as a file), `--all-checks` given to `build` (the same), `--all-checks`
# with `--cache` (no cache is read or written), and the tools `build` runs:
# no `llc` on PATH, an `llc` of another version and one whose version cannot
# be read, no `clang`, a `clang` that fails (its standard error is the
# reason). The stand-ins are written here, and PATH holds a directory of
# links to the real `z3` and `clang` (and `llc` where the run wants it).
#   ends.sh BMB
bmb=$1
cat > p.bmb <<'BMB'
#[summary("a program the gate proves")]
mod p {
    #[effects(io)]
    fn main() -> () {
        print_i64(add(2, 3));
    }

    #[effects(pure)]
    fn add(a: i64, b: i64) -> (r: i64)
        requires a < 100 && b < 100 && a > -100 && b > -100,
        ensures r == a + b,
    {
        a + b
    }
}
BMB
printf '#[summary("x")]\nmod q {\n    fn f() -> i64 {\n        true\n    }\n}\n' > bad.bmb
run() {
  echo "== $*"
  "$bmb" "$@"
  echo "exit $?"
}
run emit-llvm p.bmb -o missing/out.ll
run emit-llvm --stats missing/s p.bmb -o out.ll
touch file
run emit-llvm --cache file p.bmb -o out.ll
run build --cache file p.bmb -o prog-file
rm -f prog-file
run emit-llvm bad.bmb -o bad.ll
run build bad.bmb -o bad
run emit-llvm p.bmb --cache -o out.ll
run build --all-checks p.bmb -o prog
run emit-llvm --all-checks --cache unused p.bmb -o all.ll
if [ -e unused ]; then echo "unused: written"; else echo "unused: not written"; fi
run build p.bmb -o missing/prog
z3=$(command -v z3)
clang=$(command -v clang)
llc=$(command -v llc)
mkdir tools
ln -s "$z3" tools/z3
ln -s "$clang" tools/clang
echo "== no llc"
PATH="$PWD/tools" "$bmb" build p.bmb -o prog
echo "exit $?"
printf '#!/bin/sh\necho "LLVM (http://llvm.org/):"\necho "  LLVM version 17.0.6"\n' > tools/llc
chmod +x tools/llc
echo "== llc of another version"
PATH="$PWD/tools" "$bmb" build p.bmb -o prog
echo "exit $?"
printf '#!/bin/sh\necho "no version here"\n' > tools/llc
echo "== llc without a version"
PATH="$PWD/tools" "$bmb" build p.bmb -o prog
echo "exit $?"
rm tools/llc tools/clang
ln -s "$llc" tools/llc
echo "== no clang"
PATH="$PWD/tools" "$bmb" build p.bmb -o prog
echo "exit $?"
printf '#!/bin/sh\necho "clang: error: the stand-in fails" >&2\nexit 1\n' > tools/clang
chmod +x tools/clang
echo "== clang fails"
PATH="$PWD/tools" "$bmb" build p.bmb -o prog
echo "exit $?"
ls prog prog.ll 2>&1 | sed 's/^ls: //'
rm -r tools
