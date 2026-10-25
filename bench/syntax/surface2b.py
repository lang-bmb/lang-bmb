#!/usr/bin/env python3
"""2B surface alternatives, offline (backlog 2R-04d, syntax-eval-1 §5 rank 4).

  python3 bench/syntax/surface2b.py            # tables (markdown)
  python3 bench/syntax/surface2b.py --json

A draft 2B corpus (ADR-0026 examples and 2B task candidates: geometry,
shapes, search returning Option, validation returning Result, a bounded
stack, a state machine) is written once in the ADR-0026 canonical form and
converted mechanically into the alternatives of syntax-eval-1 I3 and I7:

  canon     ADR-0026: `Option::Some(x)`, `Result::Ok(x)`, first parameter
            named (`p: &Point`), path calls `Point::norm1(&p)`
  prelude   `Some`/`None`/`Ok`/`Err` bare (reserved built-in names, like
            `print_i64`); user enums stay `E::V`
  self      the first parameter of an `impl` function is `self: &T`
            (`self: &mut T`); calls stay path calls
  method    `p.norm1()` on top of prelude + self (Rust's method syntax); the
            canonical form restores `Point::norm1(&p)`

Counts lexical tokens (bench/syntax/measure.py) per variant, checks that the
four prelude names are defined nowhere in the repository's BMB code and the
benches' tasks (the reservation costs no existing name), and classifies each
method-call receiver by what decides its type: a parameter or local (types
are explicit, so the definition's own text decides), a field projection (the
struct declaration), or another call's result (the callee's signature). All
three are decided by the checker without inference; only the first is
decided by the definition's text alone (slice-local, K3).
Standard library only.
"""
import argparse
import glob
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
from measure import lex  # noqa: E402

CORPUS = {
    "geometry": r'''
#[summary("plane geometry")]
mod geo {
    struct Point { x: i64, y: i64 }

    spec fn small(p: Point) -> bool { p.x >= -1000 && p.x <= 1000 && p.y >= -1000 && p.y <= 1000 }

    spec fn nonneg(r: i64) -> bool { r >= 0 }

    impl Point {
        #[effects(pure)]
        #[uses(small, nonneg)]
        fn norm1(p: &Point) -> (r: i64)
            requires small(*p),
            ensures nonneg(r),
        {
            let ax: i64 = if p.x < 0 { 0 - p.x } else { p.x };
            let ay: i64 = if p.y < 0 { 0 - p.y } else { p.y };
            ax + ay
        }

        #[effects(pure)]
        #[uses(small)]
        fn shift(p: &mut Point, dx: i64) -> ()
            requires small(*p), dx >= 0 && dx <= 10,
        {
            p.x = p.x + dx;
        }
    }

    #[effects(pure)]
    #[uses(small, nonneg)]
    fn dist(a: &Point, b: &Point) -> (r: i64)
        requires small(*a), small(*b),
        ensures nonneg(r),
    {
        let d: Point = Point { x: a.x - b.x, y: a.y - b.y };
        Point::norm1(&d)
    }
}
''',
    "shapes": r'''
#[summary("shapes and areas")]
mod shapes {
    enum Shape { Dot, Square(i64), Rect(i64, i64) }

    spec fn side_ok(s: Shape) -> bool {
        match s { Shape::Dot => true, Shape::Square(a) => a >= 0 && a <= 1000, Shape::Rect(w, h) => w >= 0 && w <= 1000 && h >= 0 && h <= 1000 }
    }

    spec fn area_nonneg(r: i64) -> bool { r >= 0 }

    impl Shape {
        #[effects(pure)]
        #[uses(side_ok, area_nonneg)]
        fn area(s: &Shape) -> (r: i64)
            requires side_ok(*s),
            ensures area_nonneg(r),
        {
            match *s { Shape::Dot => 0, Shape::Square(a) => a * a, Shape::Rect(w, h) => w * h }
        }

        #[effects(pure)]
        #[uses(side_ok)]
        fn grow(s: &Shape) -> (r: Shape)
            requires side_ok(*s),
        {
            match *s { Shape::Dot => Shape::Square(1), Shape::Square(a) => Shape::Rect(a, a), Shape::Rect(w, h) => Shape::Rect(w, h) }
        }
    }

    #[effects(pure)]
    #[uses(side_ok, area_nonneg)]
    fn total(a: &Shape, b: &Shape) -> (r: i64)
        requires side_ok(*a), side_ok(*b),
        ensures area_nonneg(r),
    {
        Shape::area(a) + Shape::area(b)
    }
}
''',
    "search": r'''
#[summary("search returning Option")]
mod search {
    spec fn found_at(a: &[i64], t: i64, o: Option<usize>) -> bool {
        match o { Option::Some(i) => i < a.len() && a[i] == t, Option::None => forall|k: usize| k < a.len() ==> a[k] != t }
    }

    #[effects(pure)]
    #[uses(found_at)]
    fn find(a: &[i64], t: i64) -> (r: Option<usize>)
        ensures found_at(a, t, r),
    {
        let mut k: usize = 0;
        let mut out: Option<usize> = Option::None;
        while k < a.len()
            invariant k <= a.len(),
            decreases a.len() - k,
        {
            if a[k] == t { out = Option::Some(k); k = a.len(); } else { k = k + 1; }
        }
        out
    }

    #[effects(pure)]
    #[uses(found_at)]
    fn index_or(a: &[i64], t: i64, d: usize) -> usize {
        match find(a, t) { Option::Some(i) => i, Option::None => d }
    }

    #[effects(pure)]
    fn both(a: &[i64], s: i64, t: i64) -> Option<usize> {
        match find(a, s) { Option::Some(i) => match find(a, t) { Option::Some(_) => Option::Some(i), Option::None => Option::None }, Option::None => Option::None }
    }
}
''',
    "validate": r'''
#[summary("validation returning Result")]
mod validate {
    enum Why { Negative, TooLarge(i64) }

    spec fn valid(r: Result<i64, Why>, x: i64) -> bool {
        match r { Result::Ok(v) => v == x && x >= 0 && x <= 100, Result::Err(_) => x < 0 || x > 100 }
    }

    #[effects(pure)]
    #[uses(valid)]
    fn check(x: i64) -> (r: Result<i64, Why>)
        ensures valid(r, x),
    {
        if x < 0 { Result::Err(Why::Negative) } else { if x > 100 { Result::Err(Why::TooLarge(x)) } else { Result::Ok(x) } }
    }

    #[effects(pure)]
    fn sum_valid(a: i64, b: i64) -> Result<i64, Why> {
        match check(a) {
            Result::Ok(x) => match check(b) { Result::Ok(y) => Result::Ok(x + y), Result::Err(e) => Result::Err(e) },
            Result::Err(e) => Result::Err(e),
        }
    }

    #[effects(pure)]
    fn or_zero(x: i64) -> i64 {
        match check(x) { Result::Ok(v) => v, Result::Err(_) => 0 }
    }
}
''',
    "stack": r'''
#[summary("bounded stack")]
mod stack {
    struct Stack { items: [i64; 8], len: usize }

    spec fn ok(s: Stack) -> bool { s.len <= 8 }

    impl Stack {
        #[effects(pure)]
        fn empty() -> Stack {
            Stack { items: [0, 0, 0, 0, 0, 0, 0, 0], len: 0 }
        }

        #[effects(pure)]
        #[uses(ok)]
        fn push(s: &mut Stack, v: i64) -> (r: bool)
            requires ok(*s),
            ensures ok(*s),
        {
            if s.len < 8 { s.items[s.len] = v; s.len = s.len + 1; true } else { false }
        }

        #[effects(pure)]
        #[uses(ok)]
        fn pop(s: &mut Stack) -> (r: Option<i64>)
            requires ok(*s),
            ensures ok(*s),
        {
            if s.len > 0 { s.len = s.len - 1; Option::Some(s.items[s.len]) } else { Option::None }
        }

        #[effects(pure)]
        #[uses(ok)]
        fn peek(s: &Stack) -> Option<i64>
            requires ok(*s),
        {
            if s.len > 0 { Option::Some(s.items[s.len - 1]) } else { Option::None }
        }
    }

    #[effects(pure)]
    #[uses(ok)]
    fn roundtrip(v: i64) -> Option<i64> {
        let mut s: Stack = Stack::empty();
        let pushed: bool = Stack::push(&mut s, v);
        if pushed { Stack::pop(&mut s) } else { Option::None }
    }
}
''',
    "machine": r'''
#[summary("door state machine")]
mod door {
    enum State { Open, Closed, Locked(i64) }

    enum Event { Push, Pull, Lock(i64), Unlock(i64) }

    struct Door { state: State, moves: i64 }

    spec fn moves_ok(d: Door) -> bool { d.moves >= 0 && d.moves < 1000000 }

    impl Door {
        #[effects(pure)]
        #[uses(moves_ok)]
        fn step(d: &mut Door, e: Event) -> (r: bool)
            requires moves_ok(*d),
        {
            let next: Option<State> = match e {
                Event::Push => match d.state { State::Open => Option::Some(State::Closed), State::Closed => Option::None, State::Locked(_) => Option::None },
                Event::Pull => match d.state { State::Open => Option::None, State::Closed => Option::Some(State::Open), State::Locked(_) => Option::None },
                Event::Lock(k) => match d.state { State::Open => Option::None, State::Closed => Option::Some(State::Locked(k)), State::Locked(_) => Option::None },
                Event::Unlock(k) => match d.state { State::Open => Option::None, State::Closed => Option::None, State::Locked(c) => if c == k { Option::Some(State::Closed) } else { Option::None } },
            };
            match next { Option::Some(s) => { d.state = s; if d.moves < 999999 { d.moves = d.moves + 1; } true }, Option::None => false }
        }

        #[effects(pure)]
        fn is_open(d: &Door) -> bool {
            match d.state { State::Open => true, State::Closed => false, State::Locked(_) => false }
        }
    }

    #[effects(pure)]
    #[uses(moves_ok)]
    fn cycle(d: &mut Door) -> (r: bool)
        requires moves_ok(*d),
    {
        let a: bool = Door::step(d, Event::Pull);
        let b: bool = Door::is_open(d);
        a && b
    }
}
''',
}

PRELUDE = ("Some", "None", "Ok", "Err")
IMPL = re.compile(r"impl (\w+) \{")


def impl_types(src):
    return set(IMPL.findall(src))


def to_prelude(src):
    return re.sub(r"\b(?:Option|Result)::(Some|None|Ok|Err)\b", r"\1", src)


def impl_spans(src):
    """(start, end, type) of every `impl T { .. }` block."""
    out = []
    for m in IMPL.finditer(src):
        depth, i = 0, m.end() - 1
        while True:
            if src[i] == "{":
                depth += 1
            elif src[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        out.append((m.start(), i, m.group(1)))
    return out


def receivers(src):
    """{(type, fn): receiver kind} for impl functions whose first parameter is
    `&T` / `&mut T` (`&`/`&mut` of the impl's own type)."""
    out = {}
    for lo, hi, t in impl_spans(src):
        for m in re.finditer(r"fn (\w+)\((\w+): (&(?:mut )?)" + t + r"\b", src[lo:hi]):
            out[(t, m.group(1))] = (m.group(2), m.group(3).strip())
    return out


def to_self(src):
    """First `&T` parameter of an impl function -> `self: &T`; the body's uses
    of that name -> `self`."""
    recv = receivers(src)
    pieces, last = [], 0
    for lo, hi, t in impl_spans(src):
        block = src[lo:hi]
        for (tt, f), (name, ref) in recv.items():
            if tt != t:
                continue
            m = re.search(r"fn " + f + r"\(" + name + r": ", block)
            start = m.start()
            nxt = re.search(r"\n        (#\[|fn )", block[m.end():])
            end = m.end() + (nxt.start() if nxt else len(block) - m.end())
            fn_text = block[start:end]
            fn_text = re.sub(r"\b" + name + r"\b", "self", fn_text)
            block = block[:start] + fn_text + block[end:]
        pieces.append(src[last:lo] + block)
        last = hi
    return "".join(pieces) + src[last:]


def to_method(src):
    """`T::f(&x, ..)`, `T::f(&mut x, ..)`, `T::f(x, ..)` (x already a reference)
    for impl functions with a receiver -> `x.f(..)`. Returns the text and the
    kind of every converted receiver."""
    recv = receivers(src)
    kinds = []

    def sub(m):
        t, f, arg, rest = m.group(1), m.group(2), m.group(3), m.group(4)
        if (t, f) not in recv:
            return m.group(0)
        base = re.sub(r"^&(mut )?", "", arg.strip())
        kinds.append(("variable" if re.fullmatch(r"\w+", base) else
                      "field" if re.fullmatch(r"[\w.]+", base) else "call"))
        return f"{base}.{f}({rest.lstrip(', ').strip()})"
    out = re.sub(r"\b(\w+)::(\w+)\(([^,()]+(?:\([^()]*\))?)((?:,[^()]*)?)\)", sub, src)
    return out, kinds


def variants(src):
    p = to_prelude(src)
    s = to_self(p)
    m, kinds = to_method(s)
    return {"canon": src, "prelude": p, "self": to_self(src), "method": m}, kinds


def defined_names():
    """Names defined (fn, spec fn, struct, enum, variant-free) in the
    repository's BMB code and the benches' task solutions."""
    names = set()
    files = glob.glob(os.path.join(ROOT, "**", "*.bmb"), recursive=True)
    texts = [open(f).read() for f in files if "/target/" not in f]
    for f in glob.glob(os.path.join(ROOT, "bench", "*", "tasks", "*.json")):
        t = json.load(open(f))
        for k in ("reference", "weak_reference"):
            for op in t.get(k) or []:
                texts.append(op.get("code", ""))
    for t in texts:
        names |= set(re.findall(r"\b(?:fn|struct|enum|mod)\s+(\w+)", t))
    return names


def measure():
    rows, kinds = [], []
    for name, src in CORPUS.items():
        vs, ks = variants(src)
        kinds += ks
        rows.append({"program": name, **{k: len(lex(v)) for k, v in vs.items()}})
    total = {k: sum(r[k] for r in rows) for k in ("canon", "prelude", "self", "method")}
    clash = sorted(defined_names() & set(PRELUDE))
    return {"rows": rows, "total": total, "receivers": {k: kinds.count(k) for k in ("variable", "field", "call")},
            "prelude_clashes": clash}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--show", help="print one program's four variants")
    args = ap.parse_args()
    if args.show:
        vs, _ = variants(CORPUS[args.show])
        for k, v in vs.items():
            print(f"===== {k}\n{v}")
        return
    r = measure()
    if args.json:
        print(json.dumps(r))
        return
    t = r["total"]
    print("| 프로그램 | canon | prelude | self | method |")
    print("| --- | ---: | ---: | ---: | ---: |")
    for row in r["rows"]:
        print(f"| {row['program']} | {row['canon']} | {row['prelude']} | {row['self']} | {row['method']} |")
    print(f"| 합 | {t['canon']} | {t['prelude']} ({100 * (t['prelude'] / t['canon'] - 1):+.1f}%) | "
          f"{t['self']} ({100 * (t['self'] / t['canon'] - 1):+.1f}%) | {t['method']} ({100 * (t['method'] / t['canon'] - 1):+.1f}%) |")
    print()
    print(f"메서드 호출 수신자: {r['receivers']}")
    print(f"prelude 이름과 겹치는 기존 정의: {r['prelude_clashes'] or '없음'}")


if __name__ == "__main__":
    main()
