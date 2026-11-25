#!/bin/sh
# The IR side of the cache file between two compiler builds. FIRST emits
# FILE with the proof gate over an empty cache (`emit-llvm --cache`), which
# leaves entries of both sides. SECOND emits it over a copy of that file: it
# reads nothing of it (every definition is encoded and lowered) and writes it
# back under its own name. Whether the two names differ is printed. Then
# FIRST's file is given SECOND's name by hand: SECOND reuses every proof and
# every fragment of IR, as the keys are the same for the two builds, and
# once more with --audit, which computes each one again and finds no
# difference. Every IR and stats file is left behind; every cache file is
# masked on its second line.
#   cross.sh FIRST SECOND FILE
first=$1
second=$2
cp "$3" p.bmb
"$first" emit-llvm --cache one --stats one.stats p.bmb -o one.ll > /dev/null
echo "exit $?"
mine=$(sed -n 2p one/defs)
cp -r one two
"$second" emit-llvm --cache two --stats two.stats p.bmb -o two.ll > /dev/null
echo "exit $?"
theirs=$(sed -n 2p two/defs)
if [ "$mine" = "$theirs" ]; then echo "the same build"; else echo "other builds"; fi
mkdir -p forged
{ head -n 1 one/defs; echo "$theirs"; tail -n +3 one/defs; } > forged/defs
cp one/proofs forged/proofs
"$second" emit-llvm --cache forged --stats forged.stats p.bmb -o forged.ll > /dev/null
echo "exit $?"
"$second" emit-llvm --audit --cache forged --stats audit.stats p.bmb -o audit.ll > /dev/null
echo "exit $?"
for f in one/defs two/defs forged/defs; do
  sed -i '2s/.*/id -/' "$f"
done
