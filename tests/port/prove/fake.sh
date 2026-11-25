#!/bin/sh
# Runs a command with the stand-in solver `fake/z3` first on PATH:
#   fake.sh "ANSWER.." COMMAND ARG..
# The first argument is the stand-in's FAKE_SEQ: its answers in turn.
here=$(cd "$(dirname "$0")" && pwd)
FAKE_SEQ=$1
FAKE_HOME=$here
export FAKE_SEQ FAKE_HOME
shift
PATH="$here/fake:$PATH" exec "$@"
