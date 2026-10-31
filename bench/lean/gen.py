#!/usr/bin/env python3
"""Lean fixed context task sets (backlog 2D-28): m1b-write and BVB-write with
the tool documentation in one place and a shorter language-rules section.

  python3 bench/lean/gen.py            writes bench/m1b-write-lean and bench/bvb-write-lean
  python3 bench/lean/gen.py --check    fails if a committed set differs from a fresh build

Each is the frozen `write` set (`bench/m1b-write`, `bench/bvb-write`) with the
part every request carries changed, and nothing else (seed, tasks and BSB
mutants are the source set's, byte for byte):

  - Tools are documented only by their descriptions. The server also runs
    with `--tool-docs` (compiler `mcp.rs`): `query` says what each query
    returns and `write` that it is the only way to write, which is what the
    prompt's "## Tools" section said; the prompt drops that section. `build`
    is hidden like the transaction tools: no task builds an executable (the
    judge reads `verify`, the signature and the spec strength).
  - "## Language rules" keeps the rules no diagnostic gives a way forward
    for, and those that hit nearly every definition and differ from Rust and
    Verus (learning them from a rejection would cost a rejected write in
    nearly every session): full paths, effects and explicit types on every
    function and `let`, contracts as named predicates, opaque predicates and
    `#[uses]`, `#[trigger]`. The rules the compiler reports with a way
    forward, the obligations Verus has too, and the bans on constructs the
    language has since gained go (ADR-0071,
    `dev-docs/experiments/context-size/rules.md`).
  - The example and "## How to work" stay.

Frozen like the write sets (ADR-0021 §1): regenerate with this script, never
edit by hand. The gate checks that a fresh build matches and that the scripted
reference and weak agents still pass and fail (`test_gen.py`).
"""
import importlib.util
import json
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
sys.path.insert(0, os.path.join(BENCH, "m1"))
import bsb  # noqa: E402
import harness  # noqa: E402

# The write sets' generator, for its prompt text and helpers (both are `gen.py`).
_spec = importlib.util.spec_from_file_location("edit_gen", os.path.join(BENCH, "edit", "gen.py"))
edit = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(edit)

SOURCES = ("m1b-write", "bvb-write")
FLAG = "--tool-docs"
HIDE = ["build"]

OLD_INTRO = """You can only reach the code through the tools below.
There is no file access and no shell.
"""
NEW_INTRO = """You can only reach the code through the tools; their
descriptions say what each does. There is no file access and no shell.
"""
OLD_TOOLS = """## Tools
- `query` — read the standard maps. Start with `units`, then narrow down:
  `surface DEF` (signature, contract, proof status), `edges DEF [out|in|both] [depth]`
  (resolved calls / contract uses / #[uses] / &mut writes), `slice DEF`
  (the definition plus the contracts it depends on), `find NAME`,
  `contracts PRED`, `status`.
- `map` — a whole map level (L0 units and definitions, L1 surfaces, L2 edges).
""" + edit.NEW_TOOL + """- `answer` — for questions: submit the list of full definition paths.

"""
OLD_RULES = """## Language rules (milestone 1)
- Code lives in units: `#[summary("one line")] mod name { .. }`. Refer to other
  units by full path (`unit::f(..)`). There is no `use`, `pub`, `struct`,
  `enum`, `impl`, generics, closures or macros (`vec!`, `println!`, `assert!`).
- Types: `i64`, `u64`, `usize`, `bool`, `()`, arrays `[T; N]`, and slices
  `&[T]` / `&mut [T]` only as parameters. References cannot be stored or
  returned. A `&mut` argument may not alias another argument of the same call.
- Every `let` has an explicit type. Every function has a return type
  (`-> ()` for unit) and an effect: `#[effects(pure)]` or `#[effects(io)]`
  (only `io` functions may call `print_i64(x: i64)`).
- No `return`, `for`, `loop`, `match`, `break`, `continue`, `as`, `?`,
  `.unwrap()`. Use `if`/`else` and `while`; a block's last expression is its value.
- Arithmetic must provably not overflow; division needs a nonzero divisor;
  indexing must be provably in bounds. `x.wrapping_add(y)` etc. wrap on purpose.
- Contracts: `requires` / `ensures` clauses are calls to named predicates
  (`spec fn name(..) -> bool { .. }`), in the order requires, ensures, decreases.
  Name the result to mention it: `fn f(x: i64) -> (r: i64)`. `old(a)` refers to
  a `&mut` parameter's value on entry.
- Predicates are opaque: a proof sees a `spec fn`'s body only if the function
  lists it in `#[uses(p, q, unit::r)]` (not transitive).
- Every `while` needs `invariant ..,` (as needed) and `decreases ..,`; recursive
  functions need `decreases`. `forall|i: usize| ..` needs `#[trigger]` on a term
  that mentions every bound variable. Specifications may mix integer types;
  executable code may not.
"""
NEW_RULES = """## Language rules
- Refer to other units by full path: `unit::f(..)`.
- Every function has an effect, `#[effects(pure)]` (or `io`), and a return
  type (`-> ()` for unit). Every `let` has an explicit type.
- `requires` / `ensures` clauses are calls to named predicates
  (`spec fn name(..) -> bool { .. }`).
- Predicates are opaque: a proof sees a `spec fn`'s body only if the function
  lists it in `#[uses(p, q, unit::r)]` (not transitive).
- `forall|i: usize| ..` needs `#[trigger]` on a term that mentions every bound
  variable.
- Other rules come back as diagnostics when a write breaks them, most with a fix.
"""


def prompt(src):
    return edit.rewrite(os.path.join(src, "prompt.md"),
                        ((OLD_INTRO, NEW_INTRO), (OLD_TOOLS, ""), (OLD_RULES, NEW_RULES)))


def build_set(root, source):
    src = os.path.join(BENCH, source)
    out = os.path.join(root, f"{source}-lean")
    shutil.rmtree(out, ignore_errors=True)
    shutil.copytree(os.path.join(src, "seed"), os.path.join(out, "seed"))
    shutil.copytree(os.path.join(src, "tasks"), os.path.join(out, "tasks"))
    if os.path.isdir(bsb.mutants_dir(src)):
        shutil.copytree(bsb.mutants_dir(src), bsb.mutants_dir(out))
    with open(os.path.join(out, "prompt.md"), "w") as f:
        f.write(prompt(src))
    with open(os.path.join(src, "set.json")) as f:
        cfg = json.load(f)
    cfg["mcp_flags"] = cfg["mcp_flags"] + [FLAG]
    cfg["hide_tools"] = cfg["hide_tools"] + HIDE
    with open(os.path.join(out, "set.json"), "w") as f:
        json.dump(cfg, f, indent=1)
        f.write("\n")
    harness.SET = out
    with open(os.path.join(out, "MANIFEST"), "w") as f:
        f.write(harness.manifest_text())
    with open(os.path.join(out, "README.md"), "w") as f:
        f.write(f"# {source}-lean (고정 문맥, 백로그 2D-28)\n\n`bench/lean/gen.py`가 `bench/{source}`에서 만든 판이다(동결). "
                f"직접 고치지 않고 생성기로 다시 만든다. {source}와 매 요청에 들어가는 부분(프롬프트, 도구 설명 `--tool-docs`, "
                "보이는 도구)만 다르다(생성기 머리 주석).\n")
    return out


def build(root):
    return [build_set(root, s) for s in SOURCES]


def main():
    if "--check" in sys.argv:
        tmp = tempfile.mkdtemp(prefix="lean-")
        try:
            for out in build(tmp):
                name = os.path.basename(out)
                if edit.tree(out) != edit.tree(os.path.join(BENCH, name)):
                    sys.exit(f"bench/{name} differs from bench/lean/gen.py output: regenerate it")
                print(f"bench/{name} ok")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    else:
        for out in build(BENCH):
            print(out)


if __name__ == "__main__":
    main()
