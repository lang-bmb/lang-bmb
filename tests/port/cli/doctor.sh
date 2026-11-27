#!/bin/sh
# `doctor`: the pinned tools as they are, with no tool on PATH, with a `z3`
# of another version, one that prints no version and one that prints
# nothing (the stand-in of ../answers), and with an argument after the
# command, which it takes.
#   doctor.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
"$bmb" doctor
echo "exit $?"
"$bmb" doctor extra
echo "exit $?"
PATH=/nonexistent "$bmb" doctor
echo "exit $?"
for mode in old garbage silent cut spaced; do
  sh "$here/../answers/fake.sh" "$mode" "$bmb" doctor
  echo "exit $?"
done
