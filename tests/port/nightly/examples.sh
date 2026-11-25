#!/bin/sh
# Runs `examples` over every executable function of FILE, as LISTER's
# `reads` names them (the same list for both sides), with the options
# given before them. A file with no executable function, or one that does
# not check, says so.
#   examples.sh LISTER BMB FILE [OPTION..]
lister=$1
bmb=$2
file=$3
shift 3
"$lister" reads "$file" 2> /dev/null | sed -n 's/^{"def":"\([^"]*\)","kind":"fn".*/\1/p' > names
if [ ! -s names ]; then
  echo "no executable function"
  rm -f names
  exit 0
fi
while IFS= read -r name; do
  set -- "$@" "$name"
done < names
rm -f names
"$bmb" examples "$@" -- "$file"
echo "exit $?"
