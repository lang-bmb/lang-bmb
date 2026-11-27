#!/bin/sh
# The ends of `bmb mcp` as a process: what it does with its arguments and
# with standard input and output as wholes, where the sessions of this
# directory talk to it line by line.
# - the arguments: none, a flag it does not know, a flag after the
#   directory, two directories (each is the usage, whose words are each
#   compiler's own: only the exit code is shown), a flag where the
#   directory is expected (it is the directory), every flag at once, a flag
#   twice, the empty directory name (the directory of the run);
# - the input: every request written at once, lines that end in a carriage
#   return and a line feed, a last line without its end, no input, only
#   empty lines, a closed input, a line that is not UTF-8 after one that was
#   answered, a NUL byte in a line, a long line;
# - the output: standard error beside standard output on one pipe (the
#   counts of --stats come after the answer they follow), and an output
#   whose reader is gone before the first answer (exit code 141).
#   ends.sh BMB
bmb=$1
ping='{"jsonrpc":"2.0","id":1,"method":"ping"}'
units='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"query","arguments":{"name":"units"}}}'
begin='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"tx_begin","arguments":{}}}'
unit='{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"tx_unit","arguments":{"tx":1,"path":"made","summary":"Made by a piped session"}}}'
commit='{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"tx_commit","arguments":{"tx":1}}}'
st() {
  find . | sort | while read -r f; do
    if [ -f "$f" ] && [ ! -L "$f" ]; then sha256sum "$f"; else echo "$f"; fi
  done
}
echo '== arguments'
for args in '' '--nope ws' 'ws --compact' 'ws ws' '--compact --nope ws' '-- ws' '--Compact ws' '--compact=1 ws'; do
  # shellcheck disable=SC2086
  echo "$ping" | "$bmb" mcp $args 2> /dev/null
  echo "exit $? ($args)"
done
echo "$ping" | "$bmb" mcp --compact
echo "exit $?"
printf '%s\n%s\n%s\n%s\n' "$begin" "$unit" "$commit" "$units" | "$bmb" mcp --compact
echo "exit $?"
printf '%s\n%s\n' "$ping" "$units" | "$bmb" mcp --write-tool --compact --tool-docs --file-tools --ast-tools --item-docs --stats --undecided-info --probe-tool --rename-ops --bounded --bounded-16k ws
echo "exit $?"
printf '%s\n%s\n' "$ping" "$units" | "$bmb" mcp --stats --stats --compact --compact ws
echo "exit $?"
printf '%s\n%s\n%s\n%s\n' "$begin" "$unit" "$commit" "$units" | "$bmb" mcp ""
echo "exit $?"
st
echo '== input'
printf '%s\n%s\n%s\n%s\n%s\n' "$ping" "$begin" "$unit" "$commit" "$units" | "$bmb" mcp piped
echo "exit $?"
printf '%s\r\n%s\r\n\r\n%s\r\n' "$ping" "$units" "$ping" | "$bmb" mcp piped
echo "exit $?"
printf '%s\n%s' "$ping" "$units" | "$bmb" mcp piped
echo "exit $?"
printf '%s\r' "$ping" | "$bmb" mcp piped
echo "exit $?"
printf '' | "$bmb" mcp piped
echo "exit $?"
printf '\n\n \n\t\n\r\n' | "$bmb" mcp piped
echo "exit $?"
"$bmb" mcp piped < /dev/null
echo "exit $?"
"$bmb" mcp piped <&-
echo "exit $?"
printf '%s\n' "$ping" "$units" > requests.txt
"$bmb" mcp piped < requests.txt
echo "exit $?"
printf '%s\n\377\n%s\n' "$ping" "$units" | "$bmb" mcp piped
echo "exit $?"
printf '\377' | "$bmb" mcp piped
echo "exit $?"
printf '%s\n{"id":1,"method":"pi\300ng"}\n%s\n' "$ping" "$units" | "$bmb" mcp piped
echo "exit $?"
printf '%s\n{"id":1,"method":"pi\000ng"}\n\000\n%s\n' "$ping" "$ping" | "$bmb" mcp piped
echo "exit $?"
{ printf '{"jsonrpc":"2.0","id":"'; head -c 300000 /dev/zero | tr '\0' 'x'; printf '","method":"ping"}\n%s\n' "$ping"; } | "$bmb" mcp piped | sha256sum
echo '== output'
printf '%s\n%s\n%s\n' "$ping" "$units" '{"id":3,"method":"nope"}' | "$bmb" mcp --stats piped 2>&1
echo "exit $?"
printf '%s\nnot json\n' "$units" | "$bmb" mcp --stats piped 2>&1 > /dev/null
echo "exit $?"
{ sleep 1; printf '%s\n%s\n' "$ping" "$units"; } | { "$bmb" mcp piped; echo "exit $?" >&2; } | true
st
rm requests.txt
