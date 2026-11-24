#!/bin/sh
# Runs a command with the stand-in solver `fake/z3` first on PATH:
#   fake.sh MODE COMMAND ARG..
# MODE is what the stand-in is (its FAKE_Z3; `plain` for the usual one).
# For `vanish` the command's PATH is a directory of its own, in the working
# directory, that holds a copy of the stand-in and nothing else.
here=$(cd "$(dirname "$0")" && pwd)
FAKE_Z3=$1
FAKE_HOME=$here
export FAKE_Z3 FAKE_HOME
shift
if [ "$FAKE_Z3" = vanish ]; then
  mkdir only
  cp "$here/fake/z3" only/z3
  FAKE_PATH=$PATH
  export FAKE_PATH
  PATH="$PWD/only" exec "$@"
fi
PATH="$here/fake:$PATH" exec "$@"
