#!/bin/sh
# The usage text and the argument errors of the command line: no command, a
# command that is none, and for every command the invocations that end
# before a file is read (its arguments missing, a flag without its value, a
# value that is no number, flags that exclude each other, more or fewer
# arguments than it takes), each with what it writes and its exit code.
#   usage.sh BMB FILE
# FILE is a program that checks; it stands where a command needs a file to
# get as far as the error.
bmb=$1
file=$2
run() {
  "$bmb" "$@"
  echo "exit $?"
}
run
run nothing
run --help
run check
run verify
run verify --audit
run verify --cache
run stability
run stability --seeds x "$file"
run emit-llvm
run emit-llvm "$file"
run emit-llvm "$file" -o
run build
run build "$file" -o
run fmt
run fmt --check
run fix
run fix --write
run fix --print --confirm --induction
run fix --write --print "$file"
run fix --print --write "$file"
run fix --nope "$file"
run uses
run uses --write
run uses --cache
run uses --cache dir
run uses --write --cache dir
run lint
run lint --cache
run lint --cache dir
run lint --select W0901
run lint --select W0000 "$file"
run mutate
run mutate --def
run mutate --def a::f
run mutate "$file"
run examples
run examples --count
run examples --count 0 a::f -- "$file"
run examples --count 65 a::f -- "$file"
run examples --count 3
run examples a::f
run examples a::f --
run examples -- "$file"
run probe
run probe --def a::f
run probe --def a::f --candidate
run probe --def a::f --candidate true
run probe --candidate true "$file"
run probe --def a::f --loop x --candidate true "$file"
run probe --def a::f --loop -1 --candidate true "$file"
run probe --def a::f --loop 18446744073709551616 --candidate true "$file"
run probe --def a::f --loop 0 "$file"
run probe --def a::f --loop
run weaken
run weaken "$file"
run weaken "$file" "$file" "$file"
run weaken --readers "$file" "$file"
run weaken --readers "$file" "$file" "$file" "$file"
run weaken --nope "$file" "$file" "$file"
run reads
run hash
run map
run map L1
run map --cache
run map --cache dir
run map --budget
run map --budget x "$file"
run map --limit 3
run query
run query units
run query --offset x units -- "$file"
run query units "$file"
run import
run import dir
run apply
run apply dir
run apply dir ops extra
run mcp
run mcp --nope dir
run mcp --compact --nope dir
run dump
run dump runtime extra
run dump tokens
run dump nothing "$file"
run ast
run doctor extra > /dev/null
