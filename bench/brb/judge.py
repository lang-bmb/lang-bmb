"""Hidden adversarial judge of BRB (ADR-0034 §2, backlog 2H-03).

The judge sees only the final implementation of a session and runs the task's
hidden cases through it. The agent never sees the cases, the model or the
mutants. Both arms are judged the same way, by behaviour:

- expected outputs come from the task's Python model (`model`), never from a
  reference implementation, so a wrong reference cannot hide a wrong answer;
- a case fails when the output differs, or the call panics, traps, crashes or
  hangs (Rust: debug build, so arithmetic overflow panics; BMB: a delivered
  program is fully proved, so it has no runtime checks left);
- an implementation that does not build (Rust: `cargo build` of the judge,
  BMB: the workspace does not verify, or the target has another signature) is
  not delivered: every case counts as failed.

BMB arm, in order:
1. `bmb emit-llvm` of the workspace: exit 0 means the whole workspace is
   proved (the `bmb build` condition).
2. Domain check: a hidden unit proves that a caller who knows only the stated
   input domain may call the target (the agent's `requires` follows from the
   domain). A function that refuses valid inputs is a defect even if its
   outputs are right.
3. A generated LLVM wrapper and C driver call the target on every case.

Rust arm: a generated judge crate depends on the agent's crate by path, so the
agent's tests and dev-dependencies play no part, and calls the target on every
case inside `catch_unwind`.

Case files hold one case per line: scalars as integers (bool 0/1), slices as
length then elements. Drivers print `K ok TOKENS..` (the return value, then
each `&mut` slice as length and elements) or `K panic`. A process that dies or
hangs is restarted after the case it stopped at (`run_cases`).

Standard library only.
"""
import copy
import errno
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import cases as casegen  # noqa: E402

PROCESS_TIMEOUT = 20  # seconds per driver process
MAX_RESTARTS = 25  # crashed or hung cases before the rest count as not run
SCALARS = ("i64", "u64", "usize", "bool")
SLICES = ("&[i64]", "&mut [i64]")


# ---------- tasks ----------

def load_task(path):
    name = "brb_task_" + re.sub(r"\W", "_", os.path.basename(path)[:-3])
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    for p, ty in mod.PARAMS:
        assert ty in SCALARS + SLICES, f"{mod.ID}: unsupported type {ty}"
    assert mod.RET in SCALARS + ("()",), f"{mod.ID}: unsupported return {mod.RET}"
    return mod


def load_tasks(tdir):
    return [load_task(os.path.join(tdir, f)) for f in sorted(os.listdir(tdir)) if f.endswith(".py")]


def params_text(task):
    return ", ".join(f"{n}: {t}" for n, t in task.PARAMS)


def sig_bmb(task):
    ret = "()" if task.RET == "()" else f"(r: {task.RET})"
    return f"fn {task.NAME}({params_text(task)}) -> {ret}"


def sig_rust(task):
    ret = "" if task.RET == "()" else f" -> {task.RET}"
    return f"pub fn {task.NAME}({params_text(task)}){ret}"


def task_prompt(task, arm):
    """The task message of one arm. The statement and domain are the same text
    in both arms; only the signature line and where it goes differ."""
    if arm == "bmb":
        where = "Required signature (create unit `tasks` and put it there):\n" + sig_bmb(task)
    else:
        where = "Required signature (in `src/lib.rs`):\n" + sig_rust(task)
    return f"Implement `{task.NAME}`. {task.STATEMENT}\n\nInput domain: {task.DOMAIN}\n\n{where}"


# ---------- cases ----------

def hidden_cases(task):
    return casegen.take(task)


def encode(task, args):
    out = []
    for (_, ty), v in zip(task.PARAMS, args):
        if ty in SLICES:
            out.append(str(len(v)))
            out += [str(x) for x in v]
        else:
            out.append(str(int(v)))
    return " ".join(out)


def expected(task, args):
    args = copy.deepcopy(list(args))
    r = task.model(*args)
    toks = [] if task.RET == "()" else [str(int(r))]
    for (_, ty), v in zip(task.PARAMS, args):
        if ty == "&mut [i64]":
            toks.append(str(len(v)))
            toks += [str(x) for x in v]
    return toks


def write_case_file(task, cs, path):
    with open(path, "w") as f:
        for args in cs:
            f.write(encode(task, args) + "\n")


def spawn_retrying(argv, timeout):
    """subprocess.run, retried while the executable is "busy" (ETXTBSY). The
    judge executable was just written; with sessions judged in parallel
    threads, another thread's child can still hold the write descriptor for
    the instant between its fork and exec, and exec of a file open for writing
    fails. The window is tiny, so a short retry closes it."""
    for attempt in range(100):
        try:
            return subprocess.run(argv, capture_output=True, timeout=timeout)
        except OSError as e:
            if e.errno != errno.ETXTBSY or attempt == 99:
                raise
            time.sleep(0.02)


def run_cases(argv_for, n, timeout=PROCESS_TIMEOUT):
    """Runs a driver over cases 0..n, restarting after a case that kills or
    hangs the process. Returns {k: ("ok", tokens) | ("panic"|"crash"|"timeout"|"not run",)}."""
    got = {}
    start = 0
    restarts = 0
    while start < n:
        timed = False
        try:
            p = spawn_retrying(argv_for(start), timeout)
            out = p.stdout
        except subprocess.TimeoutExpired as e:
            out, timed = e.stdout or b"", True
        last = start - 1
        for line in out.decode(errors="replace").splitlines():
            parts = line.split()
            if len(parts) < 2 or not parts[0].isdigit():
                continue
            k = int(parts[0])
            if k < start or k >= n:
                continue
            got[k] = ("ok", parts[2:]) if parts[1] == "ok" else ("panic",)
            last = max(last, k)
        nxt = last + 1
        if nxt >= n:
            break
        got[nxt] = ("timeout",) if timed else ("crash",)
        start = nxt + 1
        restarts += 1
        if restarts > MAX_RESTARTS:
            for k in range(start, n):
                got[k] = ("not run",)
            break
    return got


def score(task, cs, got):
    """Per-case verdicts against the model."""
    kinds = {"pass": 0, "wrong": 0, "panic": 0, "crash": 0, "timeout": 0, "not run": 0}
    failing = []
    for k, args in enumerate(cs):
        g = got.get(k, ("not run",))
        if g[0] == "ok":
            kind = "pass" if g[1] == expected(task, args) else "wrong"
        else:
            kind = g[0]
        kinds[kind] += 1
        if kind != "pass" and len(failing) < 3:
            failing.append({"case": encode(task, args)[:200], "kind": kind,
                            "want": " ".join(expected(task, args))[:120],
                            "got": " ".join(g[1])[:120] if g[0] == "ok" else g[0]})
    return {"cases": len(cs), "passed": kinds["pass"], "failed": len(cs) - kinds["pass"],
            "fail_kinds": {k: v for k, v in kinds.items() if k != "pass" and v}, "failing": failing}


def not_delivered(cs, why):
    return {"delivered": False, "why": why, "cases": len(cs), "passed": 0, "failed": len(cs),
            "fail_kinds": {"not delivered": len(cs)}, "failing": []}


# ---------- BMB arm ----------

def bmb_rows(text):
    return [json.loads(l) for l in text.splitlines() if l.strip().startswith("{")]


def norm_sig(sig):
    """(name, param types, return type) of a BMB signature; names do not matter."""
    m = re.match(r"\s*fn (\w+)\((.*)\) -> (.*)$", sig.strip())
    if not m:
        return None
    params = [p.split(":", 1)[1].strip() for p in m.group(2).split(",") if ":" in p]
    ret = m.group(3).strip()
    m2 = re.match(r"\((\w+): (.*)\)$", ret)
    return m.group(1), params, (m2.group(2) if m2 else ret)


def ir_ty(ty):
    return {"bool": "i1", "()": "void"}.get(ty, "i64")


def bmb_wrapper(task):
    """External LLVM entry `brb_entry` with C-friendly arguments (all i64 or ptr)."""
    params, args, body = [], [], []
    for i, (_, ty) in enumerate(task.PARAMS):
        if ty in SLICES:
            params += [f"ptr %p{i}", f"i64 %n{i}"]
            args += [f"ptr %p{i}", f"i64 %n{i}"]
        elif ty == "bool":
            params.append(f"i64 %p{i}")
            body.append(f"  %b{i} = icmp ne i64 %p{i}, 0")
            args.append(f"i1 %b{i}")
        else:
            params.append(f"i64 %p{i}")
            args.append(f"i64 %p{i}")
    rt = ir_ty(task.RET)
    call = f'call {rt} @"tasks::{task.NAME}"({", ".join(args)})'
    if rt == "void":
        body += [f"  {call}", "  ret i64 0"]
    elif rt == "i1":
        body += [f"  %r = {call}", "  %z = zext i1 %r to i64", "  ret i64 %z"]
    else:
        body += [f"  %r = {call}", "  ret i64 %r"]
    return f"\ndefine i64 @brb_entry({', '.join(params)}) {{\nentry:\n" + "\n".join(body) + "\n}\n"


def c_driver(task):
    decl, parse, call, free, post = [], [], [], [], []
    for i, (_, ty) in enumerate(task.PARAMS):
        if ty in SLICES:
            decl += ["int64_t *", "int64_t"]
            parse.append(f"    int64_t n{i} = strtoll(p, &e, 10); p = e;\n"
                         f"    int64_t *a{i} = malloc(8 * (size_t)(n{i} + 1));\n"
                         f"    for (int64_t j = 0; j < n{i}; j++) {{ a{i}[j] = strtoll(p, &e, 10); p = e; }}")
            call += [f"a{i}", f"n{i}"]
            free.append(f"    free(a{i});")
            if ty == "&mut [i64]":
                post.append(f'    printf(" %" PRId64, n{i});\n'
                            f'    for (int64_t j = 0; j < n{i}; j++) printf(" %" PRId64, a{i}[j]);')
        elif ty in ("u64", "usize"):
            decl.append("int64_t")
            parse.append(f"    int64_t s{i} = (int64_t)strtoull(p, &e, 10); p = e;")
            call.append(f"s{i}")
        else:
            decl.append("int64_t")
            parse.append(f"    int64_t s{i} = strtoll(p, &e, 10); p = e;")
            call.append(f"s{i}")
    if task.RET == "()":
        show = ""
    elif task.RET in ("u64", "usize"):
        show = '    printf(" %" PRIu64, (uint64_t)r);'
    else:
        show = '    printf(" %" PRId64, r);'
    return (
        "#include <inttypes.h>\n#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n"
        f"extern int64_t brb_entry({', '.join(decl) or 'void'});\n"
        "int main(int argc, char **argv) {\n"
        "  FILE *f = fopen(argv[1], \"r\");\n  long start = atol(argv[2]);\n"
        "  char *line = NULL; size_t cap = 0; long k = 0;\n"
        "  while (getline(&line, &cap, f) > 0) {\n"
        "    if (k < start) { k++; continue; }\n"
        "    char *p = line, *e;\n"
        + "\n".join(parse) + "\n"
        f"    int64_t r = brb_entry({', '.join(call)});\n"
        "    (void)r;\n"
        "    printf(\"%ld ok\", k);\n"
        + show + "\n" + "\n".join(post) + "\n"
        "    printf(\"\\n\");\n    fflush(stdout);\n"
        + "\n".join(free) + "\n"
        "    k++;\n  }\n  return 0;\n}\n"
    )


def domain_unit(task, hidden, specs):
    pnames = ", ".join(n for n, _ in task.PARAMS)
    ret = "()" if task.RET == "()" else task.RET
    uses = ", ".join((["dom"] if task.BMB_DOMAIN else []) + specs)
    uses_attr = f"#[uses({uses})]\n" if uses else ""
    req = f"    requires dom({pnames}),\n" if task.BMB_DOMAIN else ""
    return (f'#[summary("hidden domain check")]\nmod {hidden} {{\n'
            + (task.BMB_DOMAIN + "\n\n" if task.BMB_DOMAIN else "")
            + f"#[effects(pure)]\n{uses_attr}fn check({params_text(task)}) -> {ret}\n{req}"
            + f"{{\n    tasks::{task.NAME}({pnames})\n}}\n}}\n")


def judge_bmb(task, ws, bmb_bin, tmp, cache=None):
    """Judges the BMB workspace `ws` (see the module doc). `cache` is the
    workspace's proof cache directory, if any."""
    cs = hidden_cases(task)
    cargs = ["--cache", cache] if cache else []

    def bmb(*a):
        r = subprocess.run([bmb_bin, *a], capture_output=True, text=True)
        return r.returncode, r.stdout

    _, surf = bmb("query", *cargs, "surface", f"tasks::{task.NAME}", "--", ws)
    found = [r for r in bmb_rows(surf) if r.get("level") == "L1"]
    if not found or norm_sig(found[0]["signature"]) != norm_sig(sig_bmb(task)):
        return {**not_delivered(cs, "target missing or wrong signature"), "verified": None, "domain_ok": None}
    ll = os.path.join(tmp, "judge.ll")
    code, _ = bmb("emit-llvm", *cargs, ws, "-o", ll)
    if code != 0:
        return {**not_delivered(cs, "not verified"), "verified": False, "domain_ok": None}
    _, l0 = bmb("map", *cargs, "L0", ws)
    rows = bmb_rows(l0)
    specs = sorted(r["path"] for r in rows if r.get("def_kind") == "spec fn")
    units = {r["path"] for r in rows if r.get("kind") == "unit"}
    hidden = "brb_domain"
    while hidden in units:
        hidden += "_"
    dpath = os.path.join(tmp, "domain.bmb")
    with open(dpath, "w") as f:
        f.write(domain_unit(task, hidden, specs))
    dcode, _ = bmb("verify", *cargs, ws, dpath)
    with open(ll) as f:
        ir = f.read()
    ir = re.sub(r"define i32 @main\(\)", "define i32 @brb_agent_main()", ir)
    with open(ll, "w") as f:
        f.write(ir + bmb_wrapper(task))
    cpath = os.path.join(tmp, "driver.c")
    with open(cpath, "w") as f:
        f.write(c_driver(task))
    exe = os.path.join(tmp, "judge-bmb")
    b = subprocess.run(["clang", "-O1", "-w", "-Wno-override-module", "-o", exe, cpath, ll],
                       capture_output=True, text=True)
    if b.returncode != 0:
        return {**not_delivered(cs, "driver build: " + b.stderr[-400:]), "verified": True, "domain_ok": dcode == 0}
    cfile = os.path.join(tmp, "cases.txt")
    write_case_file(task, cs, cfile)
    got = run_cases(lambda s: [exe, cfile, str(s)], len(cs))
    res = {"delivered": True, "verified": True, "domain_ok": dcode == 0, **score(task, cs, got)}
    return res


# ---------- Rust arm ----------

def rust_main(task, callees):
    """Judge `main.rs` calling `callees[which]` (a path such as `solution::f`)."""
    parse, args, post = [], [], []
    for i, (_, ty) in enumerate(task.PARAMS):
        if ty in SLICES:
            parse.append(f"        let n{i}: usize = it.next().unwrap().parse().unwrap();\n"
                         f"        let mut v{i}: Vec<i64> = (0..n{i}).map(|_| it.next().unwrap().parse().unwrap()).collect();")
            args.append(f"&mut v{i}[..]" if ty == "&mut [i64]" else f"&v{i}[..]")
            if ty == "&mut [i64]":
                post.append(f'                write!(o, " {{}}", v{i}.len()).unwrap();\n'
                            f'                for x in v{i}.iter() {{ write!(o, " {{}}", x).unwrap(); }}')
        elif ty == "bool":
            parse.append(f"        let s{i}: bool = it.next().unwrap().parse::<i64>().unwrap() != 0;")
            args.append(f"s{i}")
        else:
            parse.append(f"        let s{i}: {ty} = it.next().unwrap().parse().unwrap();")
            args.append(f"s{i}")
    arms = "\n".join(f"                {j} => {c}({', '.join(args)})," for j, c in enumerate(callees))
    if task.RET == "()":
        show = "                let _: () = r;"
    elif task.RET == "bool":
        show = '                write!(o, " {}", r as i64).unwrap();'
    else:
        show = '                write!(o, " {}", r).unwrap();'
    return (
        "#![allow(unused_mut, unused_variables, clippy::all)]\n"
        "use std::io::Write;\n\n"
        "fn main() {\n"
        "    std::panic::set_hook(Box::new(|_| {}));\n"
        "    let args: Vec<String> = std::env::args().collect();\n"
        "    let text = std::fs::read_to_string(&args[1]).unwrap();\n"
        "    let start: usize = args[2].parse().unwrap();\n"
        "    let which: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(0);\n"
        "    let out = std::io::stdout();\n"
        "    for (k, line) in text.lines().enumerate().skip(start) {\n"
        "        let mut it = line.split_whitespace();\n"
        + "\n".join(parse) + "\n"
        "        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match which {\n"
        + arms + "\n"
        "            _ => unreachable!(),\n"
        "        }));\n"
        "        let mut o = out.lock();\n"
        "        match res {\n"
        "            Ok(r) => {\n"
        "                write!(o, \"{} ok\", k).unwrap();\n"
        + show + "\n" + "\n".join(post) + "\n"
        "                writeln!(o).unwrap();\n"
        "            }\n"
        "            Err(_) => writeln!(o, \"{} panic\", k).unwrap(),\n"
        "        }\n"
        "        o.flush().unwrap();\n"
        "    }\n"
        "}\n"
    )


JUDGE_TOML = """[package]
name = "brb_judge"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
solution = {{ path = "{path}" }}

[workspace]

[profile.dev]
opt-level = 1
overflow-checks = true
debug-assertions = true
debug = false
"""


def build_rust_judge(task, crate, tmp, callees=None):
    """Builds the judge crate against the crate at `crate` (own target
    directory: judges may run concurrently). Returns (exe, error)."""
    jdir = os.path.join(tmp, "judge")
    target_dir = os.path.join(jdir, "target")
    os.makedirs(os.path.join(jdir, "src"), exist_ok=True)
    with open(os.path.join(jdir, "Cargo.toml"), "w") as f:
        f.write(JUDGE_TOML.format(path=os.path.abspath(crate)))
    with open(os.path.join(jdir, "src", "main.rs"), "w") as f:
        f.write(rust_main(task, callees or [f"solution::{task.NAME}"]))
    env = {**os.environ, "CARGO_TARGET_DIR": target_dir, "CARGO_TERM_COLOR": "never"}
    b = subprocess.run(["cargo", "build", "--offline", "--quiet", "--manifest-path",
                        os.path.join(jdir, "Cargo.toml")], capture_output=True, text=True, env=env)
    if b.returncode != 0:
        return None, b.stderr[-600:]
    exe = os.path.join(tmp, "judge-rust")
    shutil.copy(os.path.join(target_dir, "debug", "brb_judge"), exe)
    return exe, None


def judge_rust(task, crate, tmp):
    cs = hidden_cases(task)
    exe, err = build_rust_judge(task, crate, tmp)
    if exe is None:
        return not_delivered(cs, "judge build: " + err)
    cfile = os.path.join(tmp, "cases.txt")
    write_case_file(task, cs, cfile)
    got = run_cases(lambda s: [exe, cfile, str(s)], len(cs))
    return {"delivered": True, **score(task, cs, got)}


def mutant_crate(task, path):
    """A `solution` crate with one module per mutant (`m0`, `m1`, ..)."""
    os.makedirs(os.path.join(path, "src"), exist_ok=True)
    with open(os.path.join(path, "Cargo.toml"), "w") as f:
        f.write('[package]\nname = "solution"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n')
    with open(os.path.join(path, "src", "lib.rs"), "w") as f:
        f.write("#![allow(warnings, clippy::all)]\n")
        for i, m in enumerate(task.RUST_MUTANTS):
            f.write(f"pub mod m{i} {{\n{m}\n}}\n")


def judge_mutants(task, tmp):
    """Each mutant's hidden-case result: the judge must kill every mutant
    (judge adequacy, run in CI)."""
    cs = hidden_cases(task)
    crate = os.path.join(tmp, "mutants")
    mutant_crate(task, crate)
    callees = [f"solution::m{i}::{task.NAME}" for i in range(len(task.RUST_MUTANTS))]
    exe, err = build_rust_judge(task, crate, tmp, callees)
    if exe is None:
        return [{"mutant": i, "killed": None, "error": err} for i in range(len(callees))]
    cfile = os.path.join(tmp, "cases.txt")
    write_case_file(task, cs, cfile)
    out = []
    for i in range(len(callees)):
        got = run_cases(lambda s, i=i: [exe, cfile, str(s), str(i)], len(cs))
        s = score(task, cs, got)
        out.append({"mutant": i, "killed": s["failed"] > 0, "failed": s["failed"], "fail_kinds": s["fail_kinds"]})
    return out
