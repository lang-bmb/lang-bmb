#!/bin/sh
# Proves FILE over a cache directory whose file is damaged between runs, and
# under other headers. The damage, entry by entry after a cold run (nine
# kinds in turn): fields missing (and an entry of another kind for the same
# path, kept as it is), a path that does not unescape, a digest that does
# not hold (then a line whose digest holds with a node past the end: the
# last line whose digest holds wins), a payload that does not unescape (then
# node indices with a `+`, which Rust reads), three lines for one slot (a
# digest that does not hold, the entry, and one under another key), a
# result that differs from the solver's (reused as stored; an audit tells
# it), numbers with a `+`, a word more, and a code the compiler does not
# have. Then the same file read
# lazily (--def, which counts as damaged only the lines it could not split),
# with carriage returns before its line feeds and none after its last line,
# under another first line, another compiler, a first line alone, empty, and
# not UTF-8. Every cache file left is masked on its second line.
#   damaged.sh BMB FILE
bmb=$1
file=$2
cp "$file" p.bmb
mask() { sed -i '2s/.*/id -/' "$1" 2> /dev/null; }
digest() { printf '%s\n%s\n%s\n%s' "$1" "$2" "$3" "$4" | sha256sum | sed 's/ .*//; s/^/sha256:/'; }
"$bmb" verify --cache cache --stats cold.stats p.bmb > /dev/null
echo "exit $?"
cp cache/defs cold.defs
head -n 2 cold.defs > damaged
# Field 1 kind, 2 path, 3 key, 4 digest, 5.. payload (obligations, total,
# max, script digest, nodes).
tail -n +3 cold.defs | {
  i=0
  while IFS=' ' read -r kind path key sum o t m s nodes; do
    i=$((i + 1))
    p=$(printf '%s' "$path" | sed 's/\\s/ /g')
    case $((i % 9)) in
      1) echo "$kind $path $key"
         echo "X $path $key $(digest X "$p" "$key" "$o") $o" ;;
      2) echo "$kind $path\\x $key $sum $o $t $m $s $nodes" ;;
      3) echo "$kind $path $key sha256:00 $o $t $m $s $nodes"
         pl="$o $t $m $s E0802@100000"
         echo "$kind $path $key $(digest "$kind" "$p" "$key" "$pl") $pl" ;;
      4) pl="$o $t $m $s $nodes\\q"
         echo "$kind $path $key $(digest "$kind" "$p" "$key" "$pl") $pl"
         pl="$o $t $m $s $(printf '%s' "$nodes" | sed 's/@/@+/g')"
         echo "$kind $path $key $(digest "$kind" "$p" "$key" "$pl") $pl" ;;
      5) echo "$kind $path $key sha256:11 $o $t $m $s $nodes"
         echo "$kind $path $key $sum $o $t $m $s $nodes"
         echo "$kind $path other $(digest "$kind" "$p" other "$o $t $m $s $nodes") $o $t $m $s $nodes" ;;
      7) pl="+$o +$t $m $s $nodes"
         echo "$kind $path $key $(digest "$kind" "$p" "$key" "$pl") $pl" ;;
      8) pl="$o $t $m $s $nodes more"
         echo "$kind $path $key $(digest "$kind" "$p" "$key" "$pl") $pl" ;;
      0) pl="$o $t $m $s E9999@1"
         echo "$kind $path $key $(digest "$kind" "$p" "$key" "$pl") $pl" ;;
      *) pl="$o 7 7 $s $nodes"
         echo "$kind $path $key $(digest "$kind" "$p" "$key" "$pl") $pl" ;;
    esac
  done
} >> damaged
cp damaged cache/defs
"$bmb" verify --cache cache --stats warm.stats p.bmb
echo "exit $?"
cp cache/defs warm.defs
"$bmb" verify --audit --cache cache --stats audit.stats p.bmb > /dev/null
echo "exit $?"
cp damaged cache/defs
"$bmb" verify --audit --cache cache --stats damaged-audit.stats p.bmb > /dev/null
echo "exit $?"
first=$(tail -n +3 cold.defs | head -n 1 | sed 's/^[^ ]* \([^ ]*\) .*/\1/; s/\\s/ /g')
cp damaged cache/defs
"$bmb" verify --def "$first" --cache cache --stats lazy.stats p.bmb
echo "exit $?"
cp cache/defs lazy.defs
sed 's/$/\r/' cold.defs | head -c -1 > cache/defs
"$bmb" verify --cache cache --stats crlf.stats p.bmb > /dev/null
echo "exit $?"
for h in magic id alone empty bytes; do
  case $h in
    magic) sed '1s/v3/v2/' cold.defs > cache/defs ;;
    id) sed '2s/.*/id other/' cold.defs > cache/defs ;;
    alone) head -n 1 cold.defs > cache/defs ;;
    empty) : > cache/defs ;;
    bytes) printf 'bmb incremental cache v3\n\377\n' > cache/defs ;;
  esac
  "$bmb" verify --cache cache --stats "$h.stats" p.bmb > /dev/null
  echo "exit $?"
  cp cache/defs "$h.defs"
done
for f in cold.defs damaged warm.defs lazy.defs magic.defs id.defs alone.defs empty.defs bytes.defs cache/defs; do
  mask "$f"
done
