#!/bin/sh
# roundtrip.sh BMB EDIT FILE..: the JSON tree of the files as BMB writes it
# (`bmb ast`), read back by EDIT (`import`): the normal form of the program
# the document describes and whether that program's tree is the document
# again. A file without a tree (a syntax error, or a form the tree does not
# hold) prints `no tree`: the case `ast` compares what `bmb ast` says of it.
bmb=$1
edit=$2
shift 2
if "$bmb" ast "$@" > doc.json 2> /dev/null; then
    "$edit" import doc.json
else
    echo "no tree"
fi
rm -f doc.json
