#!/usr/bin/env python3
"""The checker's built-in declarations as a BMB unit, for the port.

stage0 builds its table of built-in functions and system types in Rust
(`Checker::new`, `system_builtins`, `builtin_adts` of `compiler/src/check.rs`).
The port's table is this generated file, `selfhost/builtins.bmb`: what
`bmb dump decls` prints for a program that declares nothing, read back.

    builtins.py write     regenerate the file (in normal form: `bmb fmt`)
    builtins.py check     fail if the file is stale (the gate)
"""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "selfhost" / "builtins.bmb"
EMPTY = '#[summary("none")]\nmod none {\n}\n'
INTS = ["i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize"]
PLAIN = {"!": "Never", "f64": "F64", "bool": "Bool", "()": "Unit", "char": "Char", "String": "String", "str": "Str"}


def lit(s: str) -> str:
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def split_top(s: str, sep: str) -> list:
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "<[(":
            depth += 1
        elif ch in ">])":
            depth -= 1
        if ch == sep and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    out.append(cur)
    return out


def kind(s: str) -> str:
    """The BMB expression that builds the type `ty_str` printed as `s`."""
    s = s.strip()
    k = "ast::TypeKind::"
    if s in INTS:
        return f"{k}Int(ast::IntTy::{s.capitalize()})"
    if s in PLAIN:
        return k + PLAIN[s]
    if s.startswith("&mut "):
        return f"{k}Ref(true, types::boxed({kind(s[5:])}))"
    if s.startswith("&"):
        return f"{k}Ref(false, types::boxed({kind(s[1:])}))"
    if s.startswith("[") and s.endswith("]"):
        parts = split_top(s[1:-1], ";")
        if len(parts) == 1:
            return f"{k}Slice(types::boxed({kind(parts[0])}))"
        return f"{k}Array(types::boxed({kind(parts[0])}), {int(parts[1])})"
    head, args = (s[: s.index("<")], split_top(s[s.index("<") + 1 : -1], ",")) if s.endswith(">") else (s, [])
    if head == "Vec":
        return f"{k}Vec(types::boxed({kind(args[0])}))"
    if head == "Box":
        return f"{k}Box(types::boxed({kind(args[0])}))"
    assert all(p.isidentifier() for p in head.split("::")), s
    return f"{k}Named(refs::split_path({lit(head)}), {types(args)})"


def types(args: list) -> str:
    assert len(args) <= 2, args
    return f"args{len(args)}({', '.join(kind(a) for a in args)})"


def kinds(ts: list) -> str:
    assert len(ts) <= 4, ts
    return f"tys{len(ts)}({', '.join(kind(t) for t in ts)})"


def effects(text: str) -> str:
    on = set() if text == "pure" else set(text.split(", "))
    assert on <= {"alloc", "io", "nondet", "panic"}, text
    return "types::Effects { " + ", ".join(f"{a}: {'true' if a in on else 'false'}" for a in ("alloc", "io", "nondet", "panic")) + " }"


def raw(rows: list) -> str:
    adts, fns = [], []
    for r in rows:
        if "type" in r:
            assert not r["generics"] and r["inv"] is None, r
            unit, name = r["type"].rsplit("::", 1)
            head = f"path: String::from({lit(r['type'])}), unit: String::from({lit(unit)}), name: String::from({lit(name)}), generics: Vec::new()"
            tail = "span: source::Span { file: 0, lo: 0, hi: 0 }, inv: None"
            if r["kind"] == "struct":
                body = "".join(f"        fs.push(types::FieldSig {{ name: String::from({lit(n)}), ty: {kind(t)} }});\n" for n, t in r["body"])
                adts.append(f"        let mut fs: Vec<types::FieldSig> = Vec::new();\n{body}        out.push(types::Adt {{ {head}, kind: types::AdtKind::Struct(fs), {tail} }});\n")
            else:
                body = "".join(f"        vs.push(types::VariantSig {{ name: String::from({lit(n)}), tys: {kinds(ts)} }});\n" for n, ts in r["body"])
                adts.append(f"        let mut vs: Vec<types::VariantSig> = Vec::new();\n{body}        out.push(types::Adt {{ {head}, kind: types::AdtKind::Enum(vs), {tail} }});\n")
        elif r.get("builtin"):
            assert r["kind"] == "exec" and not r["generics"] and r["impl_generics"] == 0, r
            assert r["trait_of"] is None and not r["default"] and not r["vacuous"], r
            fns.append(
                f"        out.push(sig({lit(r['fn'])}, {kinds(r['params'])}, {kind(r['ret'])}, {effects(r['effects'])}, {'true' if r['in_impl'] else 'false'}));\n"
            )
        else:
            raise AssertionError(r)
    blocks = "".join("        {\n" + a.replace("\n        ", "\n            ").rstrip(" ") + "        }\n" for a in adts)
    helpers = ""
    for n in range(3):
        ps = ", ".join(f"a{i}: ast::TypeKind" for i in range(n))
        body = "".join(f"        v.push(types::ty(a{i}));\n" for i in range(n))
        mut = "mut " if n else ""
        helpers += f"    #[effects(alloc)]\n    fn args{n}({ps}) -> Vec<ast::Type> {{\n        let {mut}v: Vec<ast::Type> = Vec::new();\n{body}        v\n    }}\n\n"
    for n in range(5):
        ps = ", ".join(f"a{i}: ast::TypeKind" for i in range(n))
        body = "".join(f"        v.push(a{i});\n" for i in range(n))
        mut = "mut " if n else ""
        helpers += f"    #[effects(alloc)]\n    fn tys{n}({ps}) -> Vec<ast::TypeKind> {{\n        let {mut}v: Vec<ast::TypeKind> = Vec::new();\n{body}        v\n    }}\n\n"
    return (
        "#[summary(\"The checker's built-in declarations: scripts/builtins.py generates them from `bmb dump decls` of a program that declares nothing (not edited by hand). "
        "Takes the place of stage0's Checker::new·system_builtins·builtin_adts (ADR-0091)\")]\n"
        "mod builtins {\n"
        '    #[summary("System types (in path order)")]\n'
        "    #[effects(alloc)]\n"
        "    fn adts() -> Vec<types::Adt> {\n"
        "        let mut out: Vec<types::Adt> = Vec::new();\n"
        f"{blocks}"
        "        out\n"
        "    }\n\n"
        '    #[summary("Built-in functions (in path order)")]\n'
        "    #[effects(alloc)]\n"
        "    fn fns() -> Vec<types::FnSig> {\n"
        "        let mut out: Vec<types::FnSig> = Vec::new();\n"
        f"{''.join(fns)}"
        "        out\n"
        "    }\n\n"
        f"{helpers}"
        "    #[effects(alloc)]\n"
        "    fn sig(path: &str, params: Vec<ast::TypeKind>, ret: ast::TypeKind, effect: types::Effects, in_impl: bool) -> types::FnSig {\n"
        "        types::FnSig { path: String::from(path), kind: ast::FnKind::Exec, generics: Vec::new(), impl_generics: 0, params: params, ret: ret, effect: effect, "
        "vacuous: false, builtin: true, in_impl: in_impl, trait_of: None, default: false, span: source::Span { file: 0, lo: 0, hi: 0 } }\n"
        "    }\n"
        "}\n"
    )


def generate(bmb: str) -> str:
    with tempfile.TemporaryDirectory() as d:
        src = Path(d) / "none.bmb"
        src.write_text(EMPTY, encoding="utf-8")
        r = subprocess.run([bmb, "dump", "decls", str(src)], capture_output=True, text=True)
        if r.returncode != 0 or not r.stdout.startswith("---\n"):
            sys.exit(f"builtins: bmb dump decls failed\n{r.stdout}{r.stderr}")
        rows = [json.loads(line) for line in r.stdout.splitlines()[1:]]
        f = Path(d) / "builtins.bmb"
        f.write_text(raw(rows), encoding="utf-8")
        # The normal form needs the units it names: the unit is cut out of what `bmb fmt` prints for the whole port.
        rest = sorted(str(p) for p in OUT.parent.glob("*.bmb") if p != OUT)
        r = subprocess.run([bmb, "fmt", str(f), *rest], capture_output=True, text=True)
        if r.returncode != 0:
            sys.exit(f"builtins: bmb fmt failed\n{r.stdout[:2000]}{r.stderr}")
        at = r.stdout.index("\nmod builtins {\n")
        lo = r.stdout.rindex("\n", 0, at) + 1
        return r.stdout[lo : r.stdout.index("\n}\n", at) + 3]


def main() -> int:
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    bmb = str(ROOT / "target" / "debug" / "bmb")
    if cmd == "write":
        OUT.write_text(generate(bmb), encoding="utf-8")
        return 0
    if cmd == "check":
        want = generate(bmb)
        have = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
        if want != have:
            print("builtins: selfhost/builtins.bmb is stale; run scripts/builtins.py write")
            return 1
        print("builtins: fresh")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
