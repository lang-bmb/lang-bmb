#!/bin/sh
# The ends of `weaken` that are no comparison of two files: the two versions
# as workspace directories (as the Rust compiler's own test makes them from
# the corpus case `spec`), a workspace against a file, a version that does
# not check (the old one, the new one, both: the first one's diagnostics),
# a version that cannot be read (the old one, the new one, both: each is
# said), a program against a program of other units (everything removed and
# added), a program against one with a library unit, and no solver on PATH
# (a comparison that needs it is undecided).
#   ends.sh BMB ROOT
bmb=$1
dir=${2}tests/weaken
run() {
  "$bmb" weaken "$@"
  echo "exit $?"
}
mkdir -p a/src b/src
cp "$dir/spec/old.bmb" a/src/m.bmb
cp "$dir/spec/new.bmb" b/src/m.bmb
run a b
run b a
run a "$dir/spec/new.bmb"
printf 'mod m {}\n' > bad.bmb
run bad.bmb "$dir/spec/new.bmb"
run "$dir/spec/old.bmb" bad.bmb
run bad.bmb bad.bmb
run absent.bmb "$dir/spec/new.bmb"
run "$dir/spec/old.bmb" absent.bmb
run absent.bmb missing.bmb
run "$dir/spec/old.bmb" "$dir/shapes/new.bmb"
run "$dir/methods/old.bmb" "${2}tests/run/std_num.bmb"
run "${2}tests/run/std_num.bmb" "${2}tests/run/std_num.bmb"
PATH=/nonexistent run "$dir/ensures/old.bmb" "$dir/ensures/new.bmb"
PATH=/nonexistent run "$dir/same/old.bmb" "$dir/same/new.bmb"
