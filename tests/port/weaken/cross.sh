#!/bin/sh
# `weaken` between versions of different cases of the corpus `tests/weaken`:
# the old version of each case against the new version of every other one.
# Most cases name their unit `m` and share some names of definitions, so the
# pairs meet contracts that differ in every way, signatures that differ,
# definitions matched by identity under another name, and items the two
# programs cannot share.
#   cross.sh BMB ROOT
bmb=$1
dir=${2}tests/weaken
for a in "$dir"/*/; do
  for b in "$dir"/*/; do
    if [ "$a" != "$b" ]; then
      "$bmb" weaken "${a}old.bmb" "${b}new.bmb"
      echo "exit $?"
    fi
  done
done
