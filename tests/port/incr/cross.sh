#!/bin/sh
# The cache file between two compiler builds. FIRST proves FILE cold; the
# second line of its cache file names FIRST's build. SECOND proves FILE
# over that file: it reads nothing of it (every definition is encoded) and
# writes it back under its own name. Whether the two names differ is
# printed. Then FIRST's file is given SECOND's name by hand: SECOND reuses
# every entry, as the keys are the same for the two builds. Last, STAGE0
# writes entries of the IR side too (`emit-llvm --cache`); under FIRST's
# name its file is read by FIRST, which has nothing to write, then with
# every proof entry's digest damaged, which FIRST writes back as it was
# before the damage, IR entries included. Every cache file left is masked on its second line.
#   cross.sh FIRST SECOND STAGE0 FILE
first=$1
second=$2
stage0=$3
cp "$4" p.bmb
"$first" verify --cache one --stats one.stats p.bmb > /dev/null
echo "exit $?"
mine=$(sed -n 2p one/defs)
cp -r one two
"$second" verify --cache two --stats two.stats p.bmb > /dev/null
echo "exit $?"
theirs=$(sed -n 2p two/defs)
if [ "$mine" = "$theirs" ]; then echo "the same build"; else echo "other builds"; fi
mkdir -p forged
{ head -n 1 one/defs; echo "$theirs"; tail -n +3 one/defs; } > forged/defs
"$second" verify --cache forged --stats forged.stats p.bmb > /dev/null
echo "exit $?"
"$stage0" emit-llvm --cache ir --stats ir.stats p.bmb -o p.ll > /dev/null
echo "exit $?"
grep -c '^I ' ir/defs
{ head -n 1 ir/defs; echo "$mine"; tail -n +3 ir/defs; } > ir/forged
cp ir/forged ir/defs
"$first" verify --cache ir --stats ir-first.stats p.bmb > /dev/null
echo "exit $?"
cmp -s ir/defs ir/forged && echo "kept as it was" || echo "written back"
sed '/^P /{s/ sha256:[0-9a-f]* / sha256:00 /2;}' ir/forged > ir/defs
"$first" verify --cache ir --stats ir-damaged.stats p.bmb > /dev/null
echo "exit $?"
cmp -s ir/defs ir/forged && echo "written back as it was" || echo "written back otherwise"
grep -c '^I ' ir/defs
rm -f ir/forged p.ll
for f in one/defs two/defs forged/defs ir/defs; do
  sed -i '2s/.*/id -/' "$f"
done
