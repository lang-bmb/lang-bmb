#!/usr/bin/env python3
"""Integer conversion surface, offline (backlog 2R-04c, syntax-eval-1 §5
rank 3, J1).

  python3 bench/syntax/conv2c.py            # tables (markdown)
  python3 bench/syntax/conv2c.py --json

A small corpus with conversion sites is written once in the form proposed by
ADR-0028 (`T::try_from(x).unwrap()`, a range obligation where the target
cannot hold every source value) and converted mechanically into the
alternative of syntax-eval-1 J1 (`x as T`, integer to integer only, with the
same obligation):

  try_from  `i64::try_from(n).unwrap()`
  as        `n as i64`; an operand that is not a name, literal, call, field,
            index or parenthesized expression is parenthesized: `as` binds
            tighter than the binary operators (Rust)

Counts lexical tokens (bench/syntax/measure.py) per variant, checks that the
two forms convert into each other mechanically (try_from -> as -> try_from
gives the same text), and classifies each site by whether it carries an
obligation (ADR-0028 §1: `u64`<->`usize` never does; `i64`->`u64`/`usize`
needs `x >= 0`; `u64`/`usize`->`i64` needs `x <= i64::MAX`).
Standard library only.
"""
import argparse
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from measure import lex  # noqa: E402

# Each program converts where M1 code needed a loop or could not mix types
# (dev-docs/measurements/m1.md §6, ADR-0028 Context).
CORPUS = {
    # tests/run/search.bmb `show_size` without its counting loop.
    "print_size": r'''
#[summary("print a length")]
mod out {
    #[effects(io)]
    fn show_size(n: usize) -> () {
        print_i64(i64::try_from(n).unwrap());
    }

    #[effects(io)]
    fn show(x: u64) -> () {
        print_u64(x);
    }

    #[effects(io)]
    fn lengths(a: &[i64], b: &[u64]) -> () {
        show_size(a.len());
        print_u64(u64::try_from(b.len()).unwrap());
    }
}
''',
    # A usize index and an i64 accumulator in one computation.
    "weighted_sum": r'''
#[summary("index-weighted sum")]
mod ws {
    spec fn small(a: &[i64]) -> bool {
        a.len() <= 1000 && forall|k: usize| k < a.len() ==> -1000 <= #[trigger] a[k] && a[k] <= 1000
    }

    #[effects(pure)]
    #[uses(small)]
    fn weighted(a: &[i64]) -> i64
        requires small(a),
    {
        let mut s: i64 = 0;
        let mut i: usize = 0;
        while i < a.len()
            invariant i <= a.len(), -1000000000 * i64::try_from(i).unwrap() <= s,
            decreases a.len() - i,
        {
            s = s + a[i] * i64::try_from(i).unwrap();
            i = i + 1;
        }
        s
    }
}
''',
    # Average of a slice: the length as a divisor of an i64 sum.
    "average": r'''
#[summary("average")]
mod avg {
    spec fn ok(a: &[i64]) -> bool { a.len() > 0 && a.len() <= 1000 }

    #[effects(pure)]
    #[uses(ok)]
    fn mean(a: &[i64], total: i64) -> i64
        requires ok(a),
    {
        total / i64::try_from(a.len()).unwrap()
    }

    #[effects(pure)]
    fn mean_u(total: u64, n: usize) -> u64 {
        if n == 0 { 0 } else { total / u64::try_from(n).unwrap() }
    }
}
''',
    # Signed values that are known non-negative, used as indices.
    "index_from_i64": r'''
#[summary("index from a signed value")]
mod idx {
    spec fn in_range(a: &[i64], k: i64) -> bool { 0 <= k && k < 1000 && 1000 <= a.len() }

    #[effects(pure)]
    #[uses(in_range)]
    fn at(a: &[i64], k: i64) -> i64
        requires in_range(a, k),
    {
        a[usize::try_from(k).unwrap()]
    }

    #[effects(pure)]
    #[uses(in_range)]
    fn next(a: &[i64], k: i64) -> i64
        requires in_range(a, k),
    {
        a[usize::try_from(k + 1).unwrap() % 1000]
    }
}
''',
    # A counter kept as u64 and reported as i64, and back.
    "counter": r'''
#[summary("counters")]
mod cnt {
    spec fn small(c: u64) -> bool { c <= 1000000 }

    #[effects(pure)]
    #[uses(small)]
    fn delta(c: u64, d: u64) -> i64
        requires small(c), small(d),
    {
        i64::try_from(c).unwrap() - i64::try_from(d).unwrap()
    }

    #[effects(pure)]
    fn clamp(x: i64) -> u64 {
        if x < 0 { 0 } else { u64::try_from(x).unwrap() }
    }
}
''',
}

TYPES = ("i64", "u64", "usize")
TRY = re.compile(r"\b(i64|u64|usize)::try_from\(")


def matching(s, i):
    """Index of the parenthesis closing the one at s[i]."""
    depth = 0
    for j in range(i, len(s)):
        if s[j] == "(":
            depth += 1
        elif s[j] == ")":
            depth -= 1
            if depth == 0:
                return j
    raise ValueError("unbalanced")


SIMPLE = re.compile(r"[A-Za-z_][A-Za-z_0-9]*(\.[A-Za-z_][A-Za-z_0-9]*(\(\))?|\[[^\[\]]*\])*|[0-9]+")


def operand(x):
    """`x` as the operand of `as`: kept when it is a name, literal, field,
    index, method call without arguments or already parenthesized; else
    parenthesized."""
    x = x.strip()
    if SIMPLE.fullmatch(x) or (x.startswith("(") and matching(x, 0) == len(x) - 1):
        return x
    return f"({x})"


def sites(src):
    """(target type, operand text) of each `T::try_from(x).unwrap()`."""
    out = []
    for m in TRY.finditer(src):
        open_ = m.end() - 1
        close = matching(src, open_)
        assert src[close + 1:].startswith(".unwrap()"), src[m.start():close + 10]
        out.append((m.group(1), src[open_ + 1:close]))
    return out


def to_as(src):
    while True:
        m = TRY.search(src)
        if not m:
            return src
        open_ = m.end() - 1
        close = matching(src, open_)
        end = close + 1 + len(".unwrap()")
        src = src[:m.start()] + f"{operand(src[open_ + 1:close])} as {m.group(1)}" + src[end:]


AS = re.compile(r" as (i64|u64|usize)\b")


def to_try(src):
    """The inverse: the operand of `as` is the expression just before it (a
    parenthesized one, or the longest simple operand)."""
    while True:
        m = AS.search(src)
        if not m:
            return src
        k = m.start()
        if src[k - 1] == ")":
            depth, j = 0, k - 1
            while True:
                if src[j] == ")":
                    depth += 1
                elif src[j] == "(":
                    depth -= 1
                    if depth == 0:
                        break
                j -= 1
            # A call or method call keeps its name: `f(x) as T`, `a.len() as T`.
            while j > 0 and (src[j - 1].isalnum() or src[j - 1] in "_."):
                j -= 1
            start = j
        else:
            # Names, fields and whole index brackets: `a[i].x as T`, but not
            # past an unmatched `[` (`a[k as T]`).
            j = k
            while j > 0:
                c = src[j - 1]
                if c.isalnum() or c in "_.":
                    j -= 1
                elif c == "]":
                    depth = 0
                    while True:
                        j -= 1
                        if src[j] == "]":
                            depth += 1
                        elif src[j] == "[":
                            depth -= 1
                            if depth == 0:
                                break
                else:
                    break
            start = j
        x = src[start:k]
        inner = x[1:-1] if x.startswith("(") and matching(x, 0) == len(x) - 1 else x
        src = src[:start] + f"{m.group(1)}::try_from({inner}).unwrap()" + src[m.end():]


def obligation(target, source):
    """Whether converting `source` to `target` carries a range obligation."""
    if {source, target} <= {"u64", "usize"}:
        return False
    return source != target


SOURCE = {  # the operand types of the corpus sites, by (program, operand)
    ("print_size", "n"): "usize",
    ("print_size", "b.len()"): "usize",
    ("weighted_sum", "i"): "usize",
    ("average", "a.len()"): "usize",
    ("average", "n"): "usize",
    ("index_from_i64", "k"): "i64",
    ("index_from_i64", "k + 1"): "i64",
    ("counter", "c"): "u64",
    ("counter", "d"): "u64",
    ("counter", "x"): "i64",
}


def measure():
    rows, round_trip, kinds = [], True, {"obligation": 0, "free": 0}
    for name, src in CORPUS.items():
        alt = to_as(src)
        back = to_try(alt)
        round_trip &= back == src
        ss = sites(src)
        for t, x in ss:
            kinds["obligation" if obligation(t, SOURCE[(name, x)]) else "free"] += 1
        rows.append({"program": name, "sites": len(ss), "try_from": len(lex(src)),
                     "as": len(lex(alt)), "round_trip": back == src})
    total = {"try_from": sum(r["try_from"] for r in rows), "as": sum(r["as"] for r in rows),
             "sites": sum(r["sites"] for r in rows)}
    return {"rows": rows, "total": total, "round_trip": round_trip, "kinds": kinds}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    r = measure()
    if args.json:
        print(json.dumps(r, ensure_ascii=False))
        return
    print("| 프로그램 | 변환 | try_from | as |")
    print("| --- | ---: | ---: | ---: |")
    for row in r["rows"]:
        print(f"| {row['program']} | {row['sites']} | {row['try_from']} | {row['as']} |")
    t = r["total"]
    pct = 100 * (t["as"] - t["try_from"]) / t["try_from"]
    print(f"| 합 | {t['sites']} | {t['try_from']:,} | {t['as']:,}({pct:+.1f}%) |")
    per = (t["try_from"] - t["as"]) / t["sites"]
    print(f"\n변환 한 곳당 {per:.1f}토큰. 의무가 붙는 변환 {r['kinds']['obligation']}, 없는 변환 {r['kinds']['free']}. "
          f"왕복(try_from -> as -> try_from) {'같음' if r['round_trip'] else '다름'}.")
    if not r["round_trip"]:
        sys.exit("conv2c: the two forms do not convert into each other")


if __name__ == "__main__":
    main()
