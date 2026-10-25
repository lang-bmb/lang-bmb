#!/usr/bin/env python3
"""Rust twin of a bench/m3 workspace (backlog 3A-05, BNB).

Writes OUT/Cargo.toml and OUT/src/{lib.rs, m<U>.rs}: an ordinary Rust library
crate with the same definitions, call graph, `&mut` write paths and io
functions as `gen.py OUT --defs N` with the same parameters. Both renderers
read one plan (gen.Unit + gen.choose), so the graphs are equal by
construction; analyze.py checks it independently from the text.

Mapping (BMB -> Rust):
  unit `mod mU { .. }`          -> module file src/mU.rs, `pub mod mU;` in lib.rs
  #[summary("..")]              -> `//! ..` module doc
  calls `mV::f(..)` / `f(..)`   -> same spelling; `use crate::{mV, ..};` for the
                                   units a file calls (module imports only)
  fn, #[effects(pure|io)]       -> `pub fn` (effects are not declared in Rust)
  spec fn P (predicate)         -> `pub fn P(..) -> bool`, doc line
                                   `/// Predicate (specification only).`
                                   (forall over a slice -> `.iter().all(..)`)
  requires P(..), ..            -> doc line `/// Requires: `P(..)`, ...` and
                                   `debug_assert!(P(..) && ..)` on entry
  ensures Q(..), ..             -> doc line `/// Ensures: `Q(..)`, ...` and
                                   `debug_assert!` before the result
  -> (r: T)                     -> `-> T`; the result is bound to `r`
                                   (`let r = ..;`) when an ensures names it
  invariant .. / decreases ..   -> `// invariant: ..` comment above the loop
  #[uses(..)]                   -> nothing (Rust has no opaque predicates)
  print_i64(e)                  -> `println!("{}", e)`
  `0 - c`                       -> `-c`; `-> ()` -> no return type

Usage: twin.py OUT --defs N [gen.py parameters]   (prints gen.py's JSON summary)
"""

import json
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(HERE), "m3"))
import gen  # noqa: E402

B = gen.B
PRED_DOC = "/// Predicate (specification only)."


def q(unit, cur, name):
    return gen.q(unit, cur, name)


def base_specs():
    rng = f"-{B} <= v && v <= {B}"
    return {
        "inb": ("pub fn inb(x: i64) -> bool {", f"    -{B} <= x && x <= {B}"),
        "all_inb": ("pub fn all_inb(a: &[i64]) -> bool {", f"    a.iter().all(|&v| {rng})"),
        "inb_below": ("pub fn inb_below(a: &[i64], n: usize) -> bool {", f"    a[..n].iter().all(|&v| {rng})"),
        "within": ("pub fn within(r: usize, a: &[i64]) -> bool {", "    r <= a.len()"),
    }


def contract_doc(requires, ensures):
    out = []
    if requires:
        out.append("/// Requires: " + ", ".join(f"`{c}`" for c in requires) + ".")
    if ensures:
        out.append("/// Ensures: " + ", ".join(f"`{c}`" for c in ensures) + ".")
    return out


def check(conds, pad="    "):
    return f"{pad}debug_assert!({' && '.join(conds)});" if conds else None


def fn_text(doc, header, body):
    return "\n".join(doc + [header] + [x for x in body if x is not None] + ["}"]) + "\n"


def render_unit(units, u, args):
    unit = units[u]
    ch = gen.choose(units, u, args)
    used = set()

    def call(v, name):
        if v != u:
            used.add(v)
        return q(v, u, name)

    defs = {}
    for n, kind, info in unit.defs:
        if kind == "spec":
            header, line = base_specs()[n]
            defs[n] = fn_text([PRED_DOC], header, [line])
        elif kind == "post":
            c = info["c"]
            body = {
                "shift": f"r - x <= {c} && x - r <= {c}",
                "halve": "-1 <= 2 * r - x && 2 * r - x <= 1",
                "abs": "r >= 0 && (r == x || r == -x)",
                "clamp": f"-{c} <= r && r <= {c}",
            }[info["tmpl"]]
            defs[n] = fn_text([PRED_DOC], f"pub fn {n}(x: i64, r: i64) -> bool {{", ["    " + body])
        elif kind == "leaf":
            c = info["c"]
            expr = {
                "shift": [f"let r = if x > {c} {{", f"    x - {c}", "} else {", f"    x + {c}", "};"],
                "halve": ["let r = x / 2;"],
                "abs": ["let r = if x < 0 {", "    -x", "} else {", "    x", "};"],
                "clamp": [f"let r = if x > {c} {{", f"    {c}", f"}} else if x < -{c} {{", f"    -{c}", "} else {",
                          "    x", "};"],
            }[info["tmpl"]]
            ens = ["inb(r)"] + ([f"{n}_ok(x, r)"] if info["post"] else [])
            defs[n] = fn_text(contract_doc(["inb(x)"], ens), f"pub fn {n}(x: i64) -> i64 {{",
                              [check(["inb(x)"])] + ["    " + x for x in expr] + [check(ens), "    r"])
        elif kind == "chain":
            lines, prev = [], "x"
            for j, (v, cn) in enumerate(ch[n]["callees"]):
                lines.append(f"    let v{j} = {call(v, cn)}({prev});")
                prev = f"v{j}"
            last = f"v{args.fanout - 1}"
            res = f"(x + {last}) / 2" if ch[n]["avg"] else last
            defs[n] = fn_text(contract_doc(["inb(x)"], ["inb(r)"]), f"pub fn {n}(x: i64) -> i64 {{",
                              [check(["inb(x)"])] + lines + [f"    let r = {res};", check(["inb(r)"]), "    r"])
        elif kind == "scan_count":
            defs[n] = fn_text(contract_doc([], ["within(r, a)"]), f"pub fn {n}(a: &[i64], t: i64) -> usize {{", [
                "    let mut c = 0;", "    let mut k = 0;", "    // invariant: k <= a.len(), c <= k",
                "    while k < a.len() {", f"        if a[k] > t && a[k] != {info['c']} {{", "            c += 1;",
                "        }", "        k += 1;", "    }", "    let r = c;", check(["within(r, a)"]), "    r"])
        elif kind == "scan_peak":
            v, cn = ch[n]["callees"][0]
            defs[n] = fn_text(contract_doc(["all_inb(a)"], ["inb(r)"]), f"pub fn {n}(a: &[i64]) -> i64 {{", [
                check(["all_inb(a)"]), f"    let mut m = {info['c']};", "    let mut k = 0;",
                "    // invariant: k <= a.len(), inb(m)", "    while k < a.len() {",
                f"        let v = {call(v, cn)}(a[k]);", "        if v > m {", "            m = v;", "        }",
                "        k += 1;", "    }", "    let r = m;", check(["inb(r)"]), "    r"])
        elif kind == "fill":
            defs[n] = fn_text(contract_doc(["inb(v)"], ["all_inb(a)"]), f"pub fn {n}(a: &mut [i64], v: i64) {{", [
                check(["inb(v)"]), "    let mut k = 0;", "    // invariant: k <= a.len(), inb(v), inb_below(a, k)",
                "    while k < a.len() {", "        a[k] = v;", "        k += 1;", "    }", check(["all_inb(a)"])])
        elif kind == "remap":
            v, cn = ch[n]["callees"][0]
            defs[n] = fn_text(contract_doc(["all_inb(a)"], ["all_inb(a)"]), f"pub fn {n}(a: &mut [i64]) {{", [
                check(["all_inb(a)"]), "    let mut k = 0;", "    // invariant: k <= a.len(), all_inb(a)",
                "    while k < a.len() {", f"        a[k] = {call(v, cn)}(a[k]);", "        k += 1;", "    }",
                check(["all_inb(a)"])])
        elif kind == "reset":
            (fv, fn_), (mv, mn) = ch[n]["callees"]
            defs[n] = fn_text(contract_doc(["inb(v)"], ["all_inb(a)"]), f"pub fn {n}(a: &mut [i64], v: i64) {{", [
                check(["inb(v)"]), f"    {call(fv, fn_)}(a, v);", f"    {call(mv, mn)}(a);", check(["all_inb(a)"])])
        elif kind == "show":
            v, cn = ch[n]["callees"][0]
            defs[n] = fn_text(contract_doc(["inb(x)"], []), f"pub fn {n}(x: i64) {{",
                              [check(["inb(x)"]), f'    println!("{{}}", {call(v, cn)}(x));'])
    head = f"//! 생성 단위 {u}: 상한 산술, 배열 훑기, 변경 경로\n\n"
    if used:
        mods = ", ".join(f"m{v}" for v in sorted(used))
        head += (f"use crate::{{{mods}}};\n\n" if len(used) > 1 else f"use crate::{mods};\n\n")
    return head + "\n".join(defs[k] for k in sorted(defs))


def write(out, args):
    n_units = args.defs // args.dpu
    units = [gen.Unit(u, args) for u in range(n_units)]
    src = os.path.join(out, "src")
    if os.path.isdir(src):
        shutil.rmtree(src)
    os.makedirs(src)
    with open(os.path.join(out, "Cargo.toml"), "w", encoding="utf-8") as f:
        f.write('[package]\nname = "bnb_twin"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n\n'
                "[lib]\npath = \"src/lib.rs\"\n\n[workspace]\n")
    with open(os.path.join(src, "lib.rs"), "w", encoding="utf-8") as f:
        # Predicates keep the BMB parameter lists, some of which they do not read.
        f.write("//! Rust twin of a bench/m3 workspace (bench/bnb/twin.py).\n\n#![allow(unused_variables)]\n\n"
                + "".join(f"pub mod m{u};\n" for u in range(n_units)))
    for u in range(n_units):
        with open(os.path.join(src, f"m{u}.rs"), "w", encoding="utf-8") as f:
            f.write(render_unit(units, u, args))
    return units


def main(argv=None):
    args = gen.parse(argv)
    write(args.out, args)
    # Same summary as gen.py (the twin has the same definitions).
    n_units = args.defs // args.dpu
    kinds = {}
    for unit in (gen.Unit(u, args) for u in range(n_units)):
        for _, k, _ in unit.defs:
            kinds[k] = kinds.get(k, 0) + 1
    print(json.dumps({"defs": sum(kinds.values()), "units": n_units, "kinds": dict(sorted(kinds.items()))}))


if __name__ == "__main__":
    main()
