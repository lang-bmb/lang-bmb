#!/bin/sh
# `fix --write` on a copy of one program: the report and the file it
# leaves, a second run over what the first wrote (nothing left to apply,
# unless a round limit or a repeated text ended the first), then `--confirm
# --write` and `--print` over that. The copy is the file the runs leave.
#   write.sh BMB FILE
bmb=$1
cp "$2" in.bmb
"$bmb" fix --write in.bmb
echo "exit $?"
"$bmb" fix --write in.bmb
echo "exit $?"
"$bmb" fix --confirm --write in.bmb
echo "exit $?"
"$bmb" fix --print in.bmb
echo "exit $?"
