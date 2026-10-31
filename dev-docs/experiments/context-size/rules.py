#!/usr/bin/env python3
"""What the language-rules section of the frozen prompt teaches, and what the
compiler would tell the agent anyway (backlog 2D-28).

  python3 dev-docs/experiments/context-size/rules.py [--bmb PATH] > dev-docs/experiments/context-size/rules.md

Offline: no model is called. Each bullet of "## Language rules" in
`bench/bvb-write/prompt.md` (the same section as every m1-family prompt) gets
one or more probes: a minimal unit that breaks the rule the way a Rust or
Verus writer would. Each probe goes through `bmb check` (rules the type
checker enforces) or `bmb verify` (rules the proof gate enforces), and the
table records the diagnostics: code, message, and the `fix` the diagnostic
carries (a message, with a replacement when it is mechanical). A probe that
yields no diagnostic means the compiler accepts the construct: the rule is
stale (the language grew after the prompt was frozen, ADR-0021) or does not
need stating.

Classes (the column "분류"):
  - 진단: the violation is rejected and the diagnostic names the form to write
    (its message or fix). The rule can be learned from the first rejection.
  - 증명: the proof gate reports it with a counterexample and a fix that names
    what to add. Same as above, one `write` later.
  - 받음: no diagnostic, the construct is accepted now.
  - 미리: the agent cannot learn it from a diagnostic: what exists (a name it
    would not guess), or a rejection without a way forward.
"""
import argparse
import json
import os
import subprocess
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))

HEAD = '#[summary("probe")]\nmod p {\n'
TAIL = "}\n"
PRED = "    spec fn pos(x: i64) -> bool { x > 0 }\n"


def unit(body):
    return HEAD + body + TAIL


def fn(sig, body, attrs="#[effects(pure)]"):
    return f"    {attrs}\n    {sig}\n    {{\n        {body}\n    }}\n"


# (rule bullet, probe name, mode, source). The bullet is the first words of the
# prompt line it tests; the source is a whole file.
PROBES = [
    ("units", "item outside `mod`", "check", "#[effects(pure)]\nfn f() -> i64 { 1 }\n"),
    ("units", "`mod` without summary", "check", "mod p {\n" + fn("fn f() -> i64", "1") + "}\n"),
    ("units", "other unit without path", "check",
     unit(fn("fn f() -> i64", "g()")) + '#[summary("q")]\nmod q {\n' + fn("fn g() -> i64", "1") + "}\n"),
    ("units", "`crate::` path", "check",
     unit(fn("fn f() -> i64", "crate::q::g()")) + '#[summary("q")]\nmod q {\n' + fn("fn g() -> i64", "1") + "}\n"),
    ("units", "`use`", "check",
     '#[summary("probe")]\nmod p {\n    use q::g;\n' + fn("fn f() -> i64", "q::g()") + "}\n"
     + '#[summary("q")]\nmod q {\n' + fn("fn g() -> i64", "1") + "}\n"),
    ("units", "`pub fn`", "check", unit(fn("pub fn f() -> i64", "1"))),
    ("units", "`struct`", "check",
     unit("    struct P { x: i64 }\n" + fn("fn f(p: &P) -> i64", "p.x"))),
    ("units", "`enum` and `match`", "check",
     unit("    enum E { A, B }\n" + fn("fn f(e: &E) -> i64", "match e { E::A => 1, E::B => 2 }"))),
    ("units", "generics", "check", unit(fn("fn f<T>(x: T) -> T", "x"))),
    ("units", "closure", "check", unit(fn("fn f() -> i64", "let c: i64 = (|x: i64| x)(1);\n        c"))),
    ("units", "`assert!`", "check", unit(fn("fn f() -> i64", "assert!(true);\n        1"))),
    ("units", "`vec!`", "check", unit(fn("fn f() -> i64", "let v: Vec<i64> = vec![1];\n        v[0]"))),
    ("types", "`i32`", "check", unit(fn("fn f(x: i32) -> i32", "x"))),
    ("types", "returned reference", "check", unit(fn("fn f(a: &[i64]) -> &[i64]", "a"))),
    ("types", "`&mut` alias", "check",
     unit(fn("fn g(a: &mut [i64], b: &mut [i64]) -> ()", "()")
          + fn("fn f() -> ()", "let mut a: [i64; 2] = [0, 0];\n        g(&mut a, &mut a)"))),
    ("let/effects", "`let` without type", "check", unit(fn("fn f() -> i64", "let x = 1;\n        x"))),
    ("let/effects", "no return type", "check", unit(fn("fn f()", "()"))),
    ("let/effects", "no effect", "check", unit(fn("fn f() -> i64", "1", attrs="// none"))),
    ("let/effects", "`pure` prints", "check", unit(fn("fn f() -> ()", "print_i64(1)"))),
    ("no return..", "`return`", "check", unit(fn("fn f(x: i64) -> i64", "if x > 0 { return 1; }\n        0"))),
    ("no return..", "`for` over a range", "check",
     unit(fn("fn f(n: u64) -> u64", "let mut s: u64 = 0;\n        for i in 0..n {\n            s = i;\n        }\n        s"))),
    ("no return..", "`loop` and `break`", "check",
     unit(fn("fn f() -> i64", "let mut i: i64 = 0;\n        loop {\n            i = i + 1;\n            break;\n        }\n        i"))),
    ("no return..", "`while` with `continue`", "check",
     unit(fn("fn f() -> i64", "let mut i: i64 = 0;\n        while i < 3\n            invariant i <= 3,\n"
                              "            decreases 3 - i,\n        {\n            i = i + 1;\n"
                              "            continue;\n        }\n        i"))),
    ("no return..", "`as`", "check", unit(fn("fn f(x: u64) -> i64", "x as i64"))),
    ("no return..", "`?`", "check", unit(fn("fn f(x: Option<i64>) -> i64", "x?"))),
    ("no return..", "`.unwrap()`", "check", unit(fn("fn f(x: Option<i64>) -> i64", "x.unwrap()"))),
    ("arithmetic", "overflow", "verify", unit(fn("fn f(x: i64) -> i64", "x + 1"))),
    ("arithmetic", "division", "verify", unit(fn("fn f(x: i64, y: i64) -> i64", "x / y"))),
    ("arithmetic", "indexing", "verify", unit(fn("fn f(a: &[i64], i: usize) -> i64", "a[i]"))),
    ("contracts", "expression clause", "check", unit(fn("fn f(x: i64) -> i64\n        requires x > 0,", "x"))),
    ("contracts", "`ensures` before `requires`", "check",
     unit(PRED + fn("fn f(x: i64) -> (r: i64)\n        ensures pos(r),\n        requires pos(x),", "x"))),
    ("contracts", "unnamed result", "check",
     unit(PRED + fn("fn f(x: i64) -> i64\n        requires pos(x),\n        ensures pos(result),", "x"))),
    ("contracts", "`old` on a value", "check",
     unit("    spec fn same(a: i64, b: i64) -> bool { a == b }\n"
          + fn("fn f(x: i64) -> (r: i64)\n        ensures same(r, old(x)),", "x", attrs="#[effects(pure)]\n    #[uses(same)]"))),
    ("opaque", "body needed, no `#[uses]`", "verify", unit(PRED + fn("fn f() -> (r: i64)\n        ensures pos(r),", "1"))),
    ("loops", "`while` without `decreases`", "check",
     unit(fn("fn f() -> i64", "let mut i: i64 = 0;\n        while i < 3 {\n            i = i + 1;\n        }\n        i"))),
    ("loops", "`while` without `invariant`", "verify",
     unit("    spec fn is3(x: i64) -> bool { x == 3 }\n"
          + fn("fn f() -> (r: i64)\n        ensures is3(r),",
               "let mut i: i64 = 0;\n        while i < 3\n            decreases 3 - i,\n"
               "        {\n            i = i + 1;\n        }\n        i", attrs="#[effects(pure)]\n    #[uses(is3)]"))),
    ("loops", "recursion without `decreases`", "check",
     unit(fn("fn f(n: u64) -> u64", "if n == 0 { 0 } else { f(n - 1) }"))),
    ("loops", "`forall` without trigger", "check",
     unit("    spec fn all_pos(a: &[i64]) -> bool { forall|i: usize| i < a.len() ==> a[i] > 0 }\n")),
    ("loops", "trigger misses a variable", "check",
     unit("    spec fn sorted(a: &[i64]) -> bool {\n        forall|i: usize, j: usize| i < j && j < a.len() ==> "
          "#[trigger] a[i] <= a[j]\n    }\n")),
    ("loops", "mixed integer types in code", "check", unit(fn("fn f(x: u64, y: usize) -> u64", "x + y"))),
]


def run(bmb, mode, src):
    with tempfile.TemporaryDirectory(prefix="bmb-rules-") as d:
        path = os.path.join(d, "probe.bmb")
        with open(path, "w") as f:
            f.write(src)
        r = subprocess.run([bmb, mode, "probe.bmb"], capture_output=True, text=True, cwd=d)
    return [json.loads(l) for l in r.stdout.splitlines() if l.startswith('{"code"')]


def cell(s):
    return s.replace("|", "\\|").replace("\n", " ")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    bmb = os.path.abspath(ap.parse_args().bmb)
    print("# 프롬프트 언어 규칙 절: 진단이 알려 주는 것\n")
    print("생성: `python3 dev-docs/experiments/context-size/rules.py`(모델 호출 없음). 해석은 `README.md` 2D-28 절.\n")
    print("| 규칙 줄 | 탐침 | 명령 | 진단 | 메시지 | 고칠 길(`fix`) |")
    print("| --- | --- | --- | --- | --- | --- |")
    for rule, name, mode, src in PROBES:
        diags = run(bmb, mode, src)
        if not diags:
            print(f"| {rule} | {name} | {mode} | 없음 | | |")
        seen = []
        for d in diags:
            fx = d.get("fix")
            fix = "" if fx is None else fx["message"] + (" (치환)" if "replacement" in fx else "")
            row = f"| {rule} | {name} | {mode} | {d['code']} | {cell(d['message'])} | {cell(fix)} |"
            if row not in seen and len(seen) < 2:
                seen.append(row)
                print(row)


if __name__ == "__main__":
    main()
