#!/bin/sh
# each.sh BMB EDIT FILE: diffs of a real program, read and applied by EDIT
# (`udiff`). The program is FILE and its other text is its normal form
# (`bmb fmt`, which for a file of the corpus that is not in normal form
# differs in many places) or, when it has none or is in it, the file with
# every third line left out. The diffs are those `diff` writes between the
# two, each way: with three lines of context, with one, with none (hunks of
# added lines only, placed by their header alone), under git's a/ and b/
# paths, with carriage returns, as a new file and as a deleted one, and two
# files in one diff. Each is applied to both texts: to the one it was made
# from, where it gives the other, and to the other, where it is refused.
bmb=$1
edit=$2
file=$3
cp "$file" one.txt
if ! "$bmb" fmt "$file" > two.txt 2> /dev/null || cmp -s one.txt two.txt; then
    awk 'NR % 3 != 0' one.txt > two.txt
fi
run() {
    echo "== $1"
    "$edit" udiff d.diff one.txt two.txt
}
for context in 3 1 0; do
    diff -U $context -L a/src/x.bmb -L b/src/x.bmb one.txt two.txt > d.diff
    run "context $context"
    diff -U $context -L src/x.bmb -L src/x.bmb two.txt one.txt > d.diff
    run "back, context $context"
done
diff -u -L a/src/x.bmb -L b/src/x.bmb one.txt two.txt | sed 's/$/\r/' > d.diff
run "carriage returns"
diff -u -L /dev/null -L b/src/x.bmb /dev/null one.txt > d.diff
run "a new file"
diff -u -L a/src/x.bmb -L /dev/null one.txt /dev/null > d.diff
run "a deleted file"
{
    echo "diff --git a/src/x.bmb b/src/x.bmb"
    echo "index 0000000..1111111 100644"
    diff -u -L a/src/x.bmb -L b/src/x.bmb one.txt two.txt
    echo "diff --git a/src/y.bmb b/src/y.bmb"
    diff -U 2 -L a/src/y.bmb -L b/src/y.bmb two.txt one.txt
} > d.diff
run "two files"
# Hunks without their line numbers: placed by their context alone.
diff -u -L a/src/x.bmb -L b/src/x.bmb one.txt two.txt | sed 's/^@@ .*/@@ @@/' > d.diff
run "no line numbers"
# Every line number one hundred lines off: the nearest place is taken.
diff -U 1 -L a/src/x.bmb -L b/src/x.bmb one.txt two.txt | sed 's/^@@ -\([0-9]*\)/@@ -1\1/' > d.diff
run "line numbers off"
rm -f d.diff one.txt two.txt
