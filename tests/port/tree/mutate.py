#!/usr/bin/env python3
"""mutate.py BMB EDIT FILE: documents the reader of a tree refuses, made from
a real one. The JSON tree of FILE as BMB writes it (`bmb ast`) is changed in
one place at a time: a member left out, added or given twice, a key that is no
name, a `kind` that does not fit, a value of another type, a number at and
past the ends of each range. A change inside a definition is put into a
document that holds that definition alone in its unit, and a change above
the definitions is made in the document with only the first definition of
each kind in each unit (the documents stay small). Of the
changes of one kind at one kind of place (the `kind` of the object around
the node, the member the node is, the type of its value and its own `kind`) the first FIRST are
kept, each as one line, up to BUDGET bytes, and EDIT reads them all
(`import`): the refusal of each, or the program it still describes.

The changes are a fixed list walked over the nodes in document order, so the
lines are the same on every run and for both sides.
"""
import json
import subprocess
import sys

BUDGET = 6_000_000
FIRST = 1
SECTIONS = ("consts", "traits", "types", "impls", "fns")


class Num(str):
    """A number as its source text."""


class Obj(list):
    """An object as its members in order (a key may repeat)."""


def load(text):
    return json.loads(text, object_pairs_hook=Obj, parse_int=Num, parse_float=Num)


def dump(v, out):
    if isinstance(v, Obj):
        out.append("{")
        for i, (k, x) in enumerate(v):
            if i:
                out.append(",")
            out.append(json.dumps(k, ensure_ascii=False))
            out.append(":")
            dump(x, out)
        out.append("}")
    elif isinstance(v, list):
        out.append("[")
        for i, x in enumerate(v):
            if i:
                out.append(",")
            dump(x, out)
        out.append("]")
    elif isinstance(v, Num):
        out.append(str(v))
    else:
        out.append(json.dumps(v, ensure_ascii=False))


COMMON = [None, Num("7"), "x y", [], Obj(), True]
STRINGS = ["", "9z", "a::", "_", "<a::T as b::Tr>::m", "<a::T as 9::Tr>::m", "block", "hex", "dec", "-", "fn"]
NUMBERS = [Num(n) for n in (
    "-1", "1.5", "0", "255", "256", "55295", "55296", "1114111", "1114112",
    "18446744073709551615", "18446744073709551616",
    "340282366920938463463374607431768211455", "340282366920938463463374607431768211456",
    "1e999", "0.1", "1e2", "-0.0", "4.9e-324", "1.7976931348623157e308", "1.7976931348623159e308",
)]


def changes(v):
    """Every replacement of the node v, each with the name of its kind."""
    for i, c in enumerate(COMMON):
        yield ("common", i), c
    if isinstance(v, Obj):
        for i in range(len(v)):
            yield ("without", v[i][0]), Obj(v[:i] + v[i + 1:])
        yield "more", Obj(v + [("zz", None)])
        if v:
            k, x = v[0]
            yield "twice", Obj(v + [(k, x)])
            for end in ("9bad", "#1", "#0", "#+2", "#x", "#", "#4294967296"):
                name = end if end == "9bad" else k + end
                yield ("key", end), Obj([(name, x)] + v[1:])
        for i, (k, x) in enumerate(v):
            if k == "kind":
                yield "nosuch", Obj(v[:i] + [(k, "nosuch")] + v[i + 1:])
                yield "unit", Obj(v[:i] + [(k, "unit")] + v[i + 1:])
            if k in ("value", "else", "tail", "body"):
                yield ("pair", k), Obj(v[:i] + [(k, Obj([("key", "k"), ("value", "v")]))] + v[i + 1:])
                yield ("unit", k), Obj(v[:i] + [(k, Obj([("kind", "unit")]))] + v[i + 1:])
                yield ("block", k), Obj(v[:i] + [(k, Obj([("kind", "block")]))] + v[i + 1:])
        if not any(k == "kind" for k, _ in v):
            for kind in ("unit", "block", "struct", "enum"):
                yield ("kind", kind), Obj(v + [("kind", kind)])
    elif isinstance(v, list):
        yield "longer", v + [None]
        yield "shorter", v[:-1]
        yield "double", v + v
    elif isinstance(v, Num):
        for n in NUMBERS:
            yield ("number", n), n
    elif isinstance(v, bool):
        yield "not", not v
    elif isinstance(v, str):
        for t in STRINGS:
            yield ("string", t), t


def nodes(v, path, place, out):
    """Every node as (path, place): the place is the kind of the object
    around it, the member it is, the type of its value and its own kind."""
    kind = "-"
    if isinstance(v, Obj):
        kind = next((x for k, x in v if k == "kind" and isinstance(x, str)), "-")
    out.append((path, place + (type(v).__name__, kind)))
    if isinstance(v, Obj):
        for i, (k, x) in enumerate(v):
            # Under a keyed section the member is a name: the section stands for it.
            inside = place[1] + "{}" if place[1] in SECTIONS + ("units", "fields", "variants") else k
            nodes(x, path + (i,), (kind, inside), out)
    elif isinstance(v, list):
        for i, x in enumerate(v):
            nodes(x, path + (i,), (place[0], place[1] + "[]"), out)


def at(v, path):
    for i in path:
        v = v[i][1] if isinstance(v, Obj) else v[i]
    return v


def put(v, path, new):
    if not path:
        return new
    i = path[0]
    if isinstance(v, Obj):
        return Obj(v[:i] + [(v[i][0], put(v[i][1], path[1:], new))] + v[i + 1:])
    return v[:i] + [put(v[i], path[1:], new)] + v[i + 1:]


def alone(doc, path, new):
    """The document with the node at path replaced; when the node is inside
    a definition, only that definition in its unit."""
    if len(path) >= 4 and isinstance(doc, Obj):
        unit_name, unit = doc[path[0]]
        if isinstance(unit, Obj):
            section_name, section = unit[path[1]]
            if section_name in SECTIONS and isinstance(section, Obj):
                name, definition = section[path[2]]
                small = Obj([(k, x) for k, x in unit if k == "summary"])
                small.append((section_name, Obj([(name, put(definition, path[3:], new))])))
                return Obj([(unit_name, small)])
    return put(doc, path, new)


def skeleton(v):
    """The document with the first member only of each keyed section and the
    first trait impl of each unit: what a change above the definitions is
    made in."""
    if not isinstance(v, Obj):
        return v
    out = Obj()
    for unit_name, unit in v:
        if isinstance(unit, Obj):
            small = Obj()
            for k, x in unit:
                if k in SECTIONS and isinstance(x, Obj):
                    x = Obj(x[:1])
                elif k == "trait_impls" and isinstance(x, list):
                    x = x[:1]
                elif k == "units":
                    x = skeleton(x)
                small.append((k, x))
            unit = small
        out.append((unit_name, unit))
    return out


def main():
    bmb, edit, file = sys.argv[1:4]
    made = subprocess.run([bmb, "ast", file], capture_output=True, text=True)
    if made.returncode != 0:
        print("no tree")
        return
    doc = load(made.stdout)
    bare = skeleton(doc)
    seen = {}
    size = 0
    with open("docs.jsonl", "w", encoding="utf-8") as f:
        # Above the definitions in the skeleton, inside them in the document.
        for base, inside in ((bare, False), (doc, True)):
            found = []
            nodes(base, (), ("-", "-"), found)
            if inside:
                # The small definitions first: a kind of place is met in
                # the smallest document that has it.
                weight = {}
                for path, _ in found:
                    if len(path) == 3:
                        out = []
                        dump(at(base, path), out)
                        weight[path] = sum(len(x) for x in out)
                found.sort(key=lambda node: weight.get(node[0][:3], 0))
            for path, place in found:
                if (len(path) >= 4) != inside:
                    continue
                for n, new in changes(at(base, path)):
                    if seen.get((place, n), 0) >= FIRST or size > BUDGET:
                        continue
                    seen[(place, n)] = seen.get((place, n), 0) + 1
                    out = []
                    dump(alone(base, path, new), out)
                    line = "".join(out)
                    size += len(line)
                    f.write(line + "\n")
    sys.stdout.flush()
    r = subprocess.run([edit, "import", "docs.jsonl"])
    sys.exit(r.returncode)


if __name__ == "__main__":
    main()
