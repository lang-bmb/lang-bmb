#!/usr/bin/env python3
"""JSONTestSuite against std::json (backlog 4B-02 (h), T4-05).

  scripts/json_suite.py [--write FILE]

Fetches nst/JSONTestSuite at the pinned commit SUITE_COMMIT into
target/json-suite (not committed: the cases are the suite's), builds
compiler/tests/data/json_canon.bmb (fully proved: it parses standard input
with std::json and prints one token a line, strings decoded and written
again by std::json::push_string) and runs every case of test_parsing:

  y_  must parse, and the tokens must be Python's `json.loads` of the same
      text (objects as pairs in order, numbers as written, strings escaped
      as serde_json writes them)
  n_  must be rejected
  i_  implementation-defined: accepted or rejected, counted

It fails unless every y_ and n_ case agrees. --write records the counts as
one JSON line (dev-docs/measurements/json-suite/).
"""
import argparse
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SUITE = "https://github.com/nst/JSONTestSuite.git"
SUITE_COMMIT = "1ef36fa01286573e846ac449e8683f8833c5b26a"
BMB = ROOT / "target" / "debug" / "bmb"


class Num(str):
    pass


class Obj(list):
    pass


def quoted(s):
    o = ['"']
    for ch in s:
        c = ord(ch)
        if ch == '"':
            o.append('\\"')
        elif ch == "\\":
            o.append("\\\\")
        elif c == 8:
            o.append("\\b")
        elif c == 12:
            o.append("\\f")
        elif c == 10:
            o.append("\\n")
        elif c == 13:
            o.append("\\r")
        elif c == 9:
            o.append("\\t")
        elif c < 32:
            o.append("\\u%04x" % c)
        else:
            o.append(ch)
    o.append('"')
    return "".join(o)


def tokens(v, out):
    if isinstance(v, Num):
        out.append("n" + v)
    elif v is True:
        out.append("true")
    elif v is False:
        out.append("false")
    elif v is None:
        out.append("null")
    elif isinstance(v, str):
        out.append("s" + quoted(v))
    elif isinstance(v, Obj):
        out.append("{")
        for k, x in v:
            out.append("k" + quoted(k))
            tokens(x, out)
        out.append("}")
    else:
        out.append("[")
        for x in v:
            tokens(x, out)
        out.append("]")


def expected(data):
    def no_constant(c):
        raise ValueError(c)

    v = json.loads(data.decode("utf-8"), object_pairs_hook=Obj, parse_float=Num,
                   parse_int=Num, parse_constant=no_constant)
    t = []
    tokens(v, t)
    return "".join(x + "\n" for x in t)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--write")
    args = ap.parse_args()
    suite = ROOT / "target" / "json-suite"
    if not (suite / ".git").exists():
        subprocess.run(["git", "clone", "-q", SUITE, str(suite)], check=True)
    subprocess.run(["git", "-C", str(suite), "checkout", "-q", SUITE_COMMIT], check=True)
    exe = ROOT / "target" / "json-suite-canon"
    b = subprocess.run([str(BMB), "build", str(ROOT / "compiler/tests/data/json_canon.bmb"),
                        "-o", str(exe)], capture_output=True, text=True)
    if b.returncode != 0:
        sys.exit(f"json_canon.bmb does not build:\n{b.stdout}{b.stderr}")
    counts = {"y_agree": 0, "y_differ": 0, "n_rejected": 0, "n_accepted": 0,
              "i_accepted": 0, "i_rejected": 0}
    differ = []
    for f in sorted((suite / "test_parsing").glob("*.json")):
        data = f.read_bytes()
        got = subprocess.run([str(exe)], input=data, capture_output=True).stdout
        got = got.decode("utf-8", "surrogateescape")
        kind = f.name[:2]
        if kind == "y_":
            ok = got == expected(data)
            counts["y_agree" if ok else "y_differ"] += 1
        elif kind == "n_":
            ok = got == "invalid\n"
            counts["n_rejected" if ok else "n_accepted"] += 1
        else:
            counts["i_rejected" if got == "invalid\n" else "i_accepted"] += 1
            ok = True
        if not ok:
            differ.append(f.name)
    row = {"suite": SUITE_COMMIT, **counts, "differ": differ}
    print(json.dumps(row))
    if args.write:
        with open(args.write, "a") as w:
            w.write(json.dumps(row) + "\n")
    if differ:
        sys.exit(f"json_suite: {len(differ)} cases differ")


if __name__ == "__main__":
    main()
