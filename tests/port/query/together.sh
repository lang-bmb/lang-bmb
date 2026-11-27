#!/bin/sh
# Queries on many files as one program: the overview, the effect view
# (the searches over the calls of every definition), the groups of equal
# identities, a text that long lines hold (the part around the first
# occurrence is shown), and a window on the rows of another.
#   together.sh BMB FILE..
bmb=$1
shift
ask() {
  q=$1
  shift
  "$bmb" query $q -- "$@"
  echo "exit $?"
}
ask overview "$@"
ask effects "$@"
ask duplicates "$@"
ask "text Some(" "$@"
ask "--budget 32768 --offset 40 text =>" "$@"
