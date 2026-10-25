#!/usr/bin/env python3
"""BCB replay harness (backlog 2A-14c, ADR-0042 §2-§3).

Replays a history (history.py) and measures, per commit, what each arm runs:

  rust  one replay of the Rust twin serves R0, R1 and R2. Every commit runs
        `cargo build` and `cargo test --no-run` once (the arms build the same
        thing) and then the test executable three times: all tests (R0), the
        tests of the changed units and the units that depend on them (R1), the
        tests whose definition reaches a changed definition through calls and
        contract checks (R2).
  b1    the BMB workspace with a kept cache: `bmb build --cache` (proof and
        executable in one) and a run of the remaining tests the R2 rule
        selects.
  b0    the same with a new cache and every remaining test, at the sampled
        commits only (--b0-every; the cold proof does not depend on what came
        before).

The remaining tests are the io definitions' tests (ADR-0042 §2): one BMB test
function per `show` definition in tests/tU.bmb calls it on the inputs of its
Rust test (rtests.cases), a generated `main` (driver/bcb_main.bmb) calls the
selected test functions, and the printed output must equal the plan's values
(`evaluate`). `bmb build` is the proof gate and the build in one command.

Cost is the CPU time (user + sys) of each command and everything it waited for
(`wait4`), and its wall time. At the toolchain-event commits of the history
header (`events.four`) each arm is measured a second time after dropping its
cache (BMB cache directory; `cargo clean`); those rows carry "cold": true.

  bcb.py run HISTORY --arms rust,b1,b0 --work DIR --out R.jsonl [--upto K]
             [--b0-every S] [--from K]   (--from continues a stopped run in the same DIR
                                          and appends to the same R.jsonl)
  bcb.py defects HISTORY --work DIR --out D.jsonl [--upto K]
                                      (one planted mutant per commit; which arms catch it)
  bcb.py suite --work DIR --results DIR  (the whole plan, SUITE; continues where it stopped)
  bcb.py manifest [--write] [--histories DIR]
                                      (frozen harness files and histories, FROZEN)
  bcb.py table R.jsonl [D.jsonl ...]   (cumulative costs, detection, the ADR-0042 §5 checks)

Uses $BMB or target/release/bmb, and cargo.
"""

import argparse
import json
import os
import random
import re
import shutil
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BMB = os.environ.get("BMB", os.path.join(ROOT, "target", "release", "bmb"))
sys.path.insert(0, HERE)
import history  # noqa: E402
import rtests  # noqa: E402
import rustify  # noqa: E402

REF = re.compile(r"(?:\b(m\d+)::)?\b([a-z]+(?:_[a-z]+)*_\d+(?:_ok)?|inb|all_inb|inb_below|within)\(")


def measure(cmd, cwd=None, env=None, check=True):
    """Runs cmd; returns (cpu seconds, wall seconds, stdout). The CPU is the
    child's with everything it waited for (`wait4`)."""
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
        start = time.monotonic()
        p = subprocess.Popen(cmd, cwd=cwd, env=env, stdout=out, stderr=err, stdin=subprocess.DEVNULL)
        _, status, usage = os.wait4(p.pid, 0)
        p.returncode = os.waitstatus_to_exitcode(status)
        wall = time.monotonic() - start
        out.seek(0)
        err.seek(0)
        data, errs = out.read().decode(), err.read().decode()
    if check and p.returncode != 0:
        sys.exit(f"bcb: {' '.join(cmd[:3])} .. failed ({p.returncode})\n{data[-3000:]}\n{errs[-3000:]}")
    return usage.ru_utime + usage.ru_stime, wall, data


def cost(cpu, wall):
    return {"cpu": round(cpu, 4), "wall": round(wall, 4)}


# Definition graph from the rendered text (what runs: calls and, in Rust test
# builds, the contract checks; comments and docs excluded).
def block_map(text, lang):
    if lang == "bmb":
        return {rustify.def_name(b): b for b in rustify.blocks(text)}
    return rustify.rust_blocks(text.split("#[cfg(test)]")[0])


def refs(u, name, block, lang):
    out = set()
    for line in block.splitlines():
        s = line.strip()
        if s.startswith(("//", "#[", "requires", "ensures", "invariant", "decreases")):
            continue
        if s.startswith(("fn ", "pub fn ", "spec fn ")):
            continue
        for m in REF.finditer(s):
            v = int(m.group(1)[1:]) if m.group(1) else u
            out.add((v, m.group(2)))
    out.discard((u, name))
    return out


class Graph:
    """Per language: definition -> text and -> referenced definitions."""

    def __init__(self, tree, lang):
        self.lang = lang
        self.text, self.refs = {}, {}
        for u in range(tree.n_units):
            self.update(tree, u)

    def update(self, tree, u):
        for key in [k for k in self.text if k[0] == u]:
            del self.text[key]
            del self.refs[key]
        for name, b in block_map(tree.text[(self.lang, u)], self.lang).items():
            self.text[(u, name)] = b
            self.refs[(u, name)] = refs(u, name, b, self.lang)

    def closure(self, d):
        seen, todo = {d}, [d]
        while todo:
            for x in self.refs.get(todo.pop(), ()):
                if x not in seen:
                    seen.add(x)
                    todo.append(x)
        return seen


def changed_defs(old, new):
    keys = set(old) | set(new)
    return {k for k in keys if old.get(k) != new.get(k)}


def rust_tests(tree, u):
    """test name -> text of its test function."""
    block = tree.text[("rust", u)].split("#[cfg(test)]")[1]
    out = {}
    for part in block.split("    #[test]\n")[1:]:
        name = part.split("fn ", 1)[1].split("(")[0]
        out[name[2:]] = part
    return out


def unit_deps(graph, n_units):
    """unit -> units it depends on (Rust module dependencies through references)."""
    deps = {u: set() for u in range(n_units)}
    for (u, _), rs in graph.refs.items():
        deps[u] |= {v for v, _ in rs if v != u}
    return deps


def rdeps_closure(deps, units):
    rev = {}
    for u, vs in deps.items():
        for v in vs:
            rev.setdefault(v, set()).add(u)
    seen, todo = set(units), list(units)
    while todo:
        for w in rev.get(todo.pop(), ()):
            if w not in seen:
                seen.add(w)
                todo.append(w)
    return seen


# ---------------------------------------------------------------- rust arm

def test_exe(rs, env):
    cpu, wall, out = measure(["cargo", "test", "--offline", "--no-run", "--message-format=json"], cwd=rs, env=env)
    exe = [m["executable"] for m in map(json.loads, (x for x in out.splitlines() if x.startswith("{")))
           if m.get("reason") == "compiler-artifact" and m.get("executable")]
    assert len(exe) == 1, exe
    return cpu, wall, exe[0]


def run_tests(exe, names, all_names):
    if not names:
        return {"cpu": 0.0, "wall": 0.0, "tests": 0}
    args = [exe, "--test-threads", "1", "-q"]
    if set(names) != set(all_names):
        args += ["--exact"] + sorted(names)
    cpu, wall, out = measure(args)
    m = re.search(r"test result: ok\. (\d+) passed", out)
    assert m and int(m.group(1)) == len(names), out[-2000:]
    return {**cost(cpu, wall), "tests": len(names)}


class RustSelect:
    """The Rust arms' test selections, kept across commits."""

    def __init__(self, tree):
        self.graph = Graph(tree, "rust")
        self.tests = {u: rust_tests(tree, u) for u in range(tree.n_units)}
        self.first = True

    def step(self, tree, changed):
        """Selections after the commit that changed `changed` units (all at first)."""
        graph = self.graph
        first, self.first = self.first, False
        if first:
            changed_d, changed_t, changed_u = set(graph.text), set(), set(range(tree.n_units))
        else:
            old_text = dict(graph.text)
            for u in changed:
                graph.update(tree, u)
            changed_d = changed_defs(old_text, graph.text)
            changed_t = set()
            for u in changed:
                new, old = rust_tests(tree, u), self.tests[u]
                changed_t |= {(u, n) for n in new if new.get(n) != old.get(n)}
                self.tests[u] = new
            changed_u = set(changed)
        all_tests = [(u, n) for u in range(tree.n_units) for n in self.tests[u]]
        names = {t: f"m{t[0]}::tests::t_{t[1]}" for t in all_tests}
        r1_units = rdeps_closure(unit_deps(graph, tree.n_units), changed_u)
        r1 = [t for t in all_tests if t[0] in r1_units]
        r2 = list(all_tests) if first else [t for t in all_tests if t in changed_t or graph.closure(t) & changed_d]
        covered = set()
        for t in r2:
            covered |= graph.closure(t)
        return {"names": names, "all": [names[t] for t in all_tests], "R0": all_tests, "R1": r1, "R2": r2,
                "covered": covered, "changed_defs": changed_d, "changed_units": changed_u}


def rust_arm(header, commits, args, out):
    rs = os.path.join(args.work, "rust")
    bmb_unused = os.path.join(args.work, "rust-bmb-unused")
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(rs, "target"))
    four = set(header["events"]["four"])
    sel = None
    for k, tree, changed in history.replay(header, commits, args.upto):
        if k == 0:
            sel = RustSelect(tree)
        if k < args.start:
            sel.step(tree, changed)
            if k == args.start - 1:
                sync_tree(tree, bmb_unused, rs)
            continue
        if k == 0:
            history.write_tree(tree, bmb_unused, rs)
        else:
            history.write_tree(tree, bmb_unused, rs, changed)
        st = sel.step(tree, changed)
        names, all_names = st["names"], st["all"]
        for cold in ([False, True] if k in four else [False]):
            if not cold and k == args.start and getattr(args, "cold_only", False):
                continue  # measured warm before the run stopped
            if cold:
                measure(["cargo", "clean", "--offline"], cwd=rs, env=env)
            b = cost(*measure(["cargo", "build", "--offline"], cwd=rs, env=env)[:2])
            c_cpu, c_wall, exe = test_exe(rs, env)
            row = {"arm": "rust", "k": k, "cold": cold, "build": b, "test_build": cost(c_cpu, c_wall),
                   "changed_defs": len(st["changed_defs"]), "changed_units": len(st["changed_units"]),
                   "R0": run_tests(exe, all_names, all_names),
                   "R1": run_tests(exe, [names[t] for t in st["R1"]], all_names),
                   "R2": {**run_tests(exe, [names[t] for t in st["R2"]], all_names),
                          "covered_defs": len(st["covered"])}}
            out.write(json.dumps(row) + "\n")
            out.flush()


# ----------------------------------------------------------------- bmb arms

def bmb_int(x):
    """BMB has no unary minus: `-5` is written `0 - 5`."""
    return f"0 - {x[1:]}" if x.startswith("-") else x


def bmb_test_units(tree):
    """tests/tU.bmb for units with io definitions: one test function per `show`."""
    out = {}
    for u in range(tree.n_units):
        fns = []
        for name, kind, info in rtests.exec_defs(tree.units[u]):
            if kind != "show":
                continue
            calls = "".join(f"        m{u}::{name}({bmb_int(x)});\n" for x in rtests.cases(name, kind, info))
            fns.append(f"    #[effects(io)]\n    #[uses(m{u}::inb)]\n    fn t_{name}() -> () {{\n{calls}    }}\n")
        if fns:
            out[u] = f'#[summary("BCB 남은 테스트: 단위 {u}의 io 정의")]\nmod t{u} {{\n' + "\n".join(fns) + "}\n"
    return out


def div(a, b):
    """Integer division truncating toward zero (BMB and Rust `/`)."""
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b > 0) else -q


def evaluate(tree, u, name, x):
    """Value of the arithmetic definition `mu::name` at x, from the plan."""
    _, kind, info = tree.find(u, name)
    if kind == "leaf":
        c, t = info["c"], info["tmpl"]
        if t == "shift":
            return x - c if x > c else x + c
        if t == "halve":
            return div(x, 2)
        if t == "abs":
            return -x if x < 0 else x
        return c if x > c else (-c if x < -c else x)
    assert kind == "chain", kind
    ch = tree.units[u].choices[name]
    v = x
    for cu, cn in ch["callees"]:
        v = evaluate(tree, cu, cn, v)
    return div(x + v, 2) if ch["avg"] else v


def expected_output(tree, selected):
    """What the driver must print: each selected test's calls, in order."""
    out = []
    for u, n in sorted(selected):
        (cu, cn), = tree.units[u].choices[n]["callees"]
        for x in rtests.cases(n, "show", tree.find(u, n)[2]):
            out.append(str(evaluate(tree, cu, cn, int(x))))
    return "".join(x + "\n" for x in out)


def driver(selected):
    calls = "".join(f"        t{u}::t_{n}();\n" for u, n in sorted(selected))
    return ('#[summary("BCB 남은 테스트 실행기")]\nmod bcb_main {\n    #[effects(io)]\n'
            f"    fn main() -> () {{\n{calls}    }}\n}}\n")


def write_if_changed(path, text):
    try:
        if open(path, encoding="utf-8").read() == text:
            return
    except OSError:
        pass
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def bmb_measure(tree, ws, cache, selected, k, arm, cold, out, n_tests):
    write_if_changed(os.path.join(ws, "driver", "bcb_main.bmb"), driver(selected))
    stats = os.path.join(ws, "stats.json")
    exe = os.path.join(ws, "bcb_tests")
    cpu, wall, _ = measure([BMB, "build", "--cache", cache, "--stats", stats,
                            os.path.join(ws, "src"), os.path.join(ws, "tests"), os.path.join(ws, "driver"),
                            "-o", exe])
    st = json.load(open(stats))
    r_cpu, r_wall, printed = measure([exe])
    if printed != expected_output(tree, selected):
        sys.exit(f"bcb: commit {k}: remaining tests print an unexpected output")
    row = {"arm": arm, "k": k, "cold": cold, "build": cost(cpu, wall),
           "encoded": len(st["encoded"]), "lowered": len(st["lowered"]),
           "encoded_defs": [d for d in st["encoded"] if not d.startswith(("t", "bcb_main"))] if arm == "b1" else None,
           "tests": {**cost(r_cpu, r_wall), "tests": len(selected), "of": n_tests}}
    if row["encoded_defs"] is None:
        del row["encoded_defs"]
    out.write(json.dumps(row) + "\n")
    out.flush()


def sync_tree(tree, bmb, rust):
    """Brings a work directory left by an earlier run to `tree` touching only the
    files that differ (so the caches stay warm); a missing one is written whole."""
    if not os.path.exists(os.path.join(rust, "Cargo.toml")) or not os.path.isdir(os.path.join(bmb, "src")):
        history.write_tree(tree, bmb, rust)
    else:
        history.write_tree(tree, bmb, rust, range(tree.n_units))


def write_tests(ws, tree):
    """tests/tU.bmb as bmb_test_units says (stale files removed)."""
    tunits = bmb_test_units(tree)
    tdir = os.path.join(ws, "tests")
    if os.path.isdir(tdir):
        for f in os.listdir(tdir):
            if int(f[1:].split(".")[0]) not in tunits:
                os.unlink(os.path.join(tdir, f))
    for u, text in tunits.items():
        write_if_changed(os.path.join(tdir, f"t{u}.bmb"), text)


class BmbSelect:
    """The remaining tests (io definitions) and the B1 selection (R2 rule on
    the BMB text: calls only, contracts are not executed)."""

    def __init__(self):
        self.graph = self.cases = None

    def step(self, tree, changed, fresh):
        """(all tests, selected tests); `fresh` selects all (first commit, B0)."""
        tests = {(u, n) for u in range(tree.n_units) for n, kind, _ in rtests.exec_defs(tree.units[u]) if kind == "show"}
        cases = {t: rtests.cases(t[1], "show", tree.find(t[0], t[1])[2]) for t in tests}
        if fresh or self.graph is None:
            self.graph = Graph(tree, "bmb")
            selected = set(tests)
        else:
            old_text = dict(self.graph.text)
            for u in changed:
                self.graph.update(tree, u)
            changed_d = changed_defs(old_text, self.graph.text)
            selected = {t for t in tests if self.cases.get(t) != cases[t] or self.graph.closure(t) & changed_d}
        self.cases = cases
        return tests, selected


def bmb_arm(header, commits, args, out, arm):
    ws = os.path.join(args.work, arm)
    cache = os.path.join(args.work, arm + "-cache")
    rust_unused = os.path.join(args.work, arm + "-rust-unused")
    four = set(header["events"]["four"])
    last_k = commits[-1]["k"] if args.upto is None else min(args.upto, commits[-1]["k"])
    sel = BmbSelect()
    for k, tree, changed in history.replay(header, commits, args.upto):
        if arm == "b0" and k % args.b0_every and k != last_k:
            continue
        sample = arm == "b0"
        if k < args.start:
            if not sample:
                sel.step(tree, changed, k == 0)
                if k == args.start - 1:
                    sync_tree(tree, ws, rust_unused)
                    write_tests(ws, tree)
            continue
        if k == 0 or sample:
            history.write_tree(tree, ws, rust_unused)
        else:
            history.write_tree(tree, ws, rust_unused, changed)
        write_tests(ws, tree)
        tests, selected = sel.step(tree, changed, k == 0 or sample)
        if sample:
            shutil.rmtree(cache, ignore_errors=True)
        for cold in ([False, True] if (k in four and not sample) else [False]):
            if not cold and k == args.start and getattr(args, "cold_only", False):
                continue  # measured warm before the run stopped
            if cold:
                shutil.rmtree(cache, ignore_errors=True)
            bmb_measure(tree, ws, cache, selected, k, arm, cold or sample, out, len(tests))


# ------------------------------------------------------------------ defects

def defect_targets(tree, commit):
    """Executable definitions the commit edited that still exist, sorted."""
    out = set()
    for e in commit["edits"]:
        u = e["unit"]
        if e["op"] == "remove":
            continue
        name = e["def"]
        if e["op"] == "spec":
            try:
                name = tree.find(u, name)[2]["of"]
            except KeyError:
                continue
        try:
            _, kind, _ = tree.find(u, name)
        except KeyError:
            continue
        if kind in rtests.EXEC:
            out.add((u, name))
    return sorted(out)


def tests_fail(exe, names, all_names):
    """Whether any of the named tests fails (no names: nothing runs)."""
    if not names:
        return False
    args = [exe, "--test-threads", "1", "-q"]
    if set(names) != set(all_names):
        args += ["--exact"] + sorted(names)
    _, _, out = measure(args, check=False)
    if re.search(r"test result: ok\. (\d+) passed", out):
        return False
    assert "test result: FAILED" in out, out[-2000:]
    return True


def defects(args):
    """Plants one mutant per commit in an edited executable definition (ADR-0042
    §4) and records which arm's checks fail on it. The planted tree is a side
    branch: it is reverted before the next commit."""
    header, commits = history.load(args.history)
    ws, rs = os.path.join(args.work, "defect-bmb"), os.path.join(args.work, "defect-rust")
    cache = os.path.join(args.work, "defect-cache")
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(rs, "target"))
    rsel, bsel = None, BmbSelect()
    with open(args.out, "a", encoding="utf-8") as out:
        out.write(json.dumps({"header": True, "defects": True, "history": os.path.basename(args.history),
                              "profile": header["profile"], "defs": header["defs"], "upto": args.upto,
                              "from": args.start, "machine": {**machine(), "history_sha": sha(args.history)}}) + "\n")
        for k, tree, changed in history.replay(header, commits, args.upto):
            if k == 0:
                rsel = RustSelect(tree)
            if k < args.start:
                rsel.step(tree, changed)
                bsel.step(tree, changed, k == 0)
                if k == args.start - 1:
                    sync_tree(tree, ws, rs)
                    write_tests(ws, tree)
                continue
            if k == 0:
                history.write_tree(tree, ws, rs)
            else:
                history.write_tree(tree, ws, rs, changed)
            write_tests(ws, tree)
            st = rsel.step(tree, changed)
            tests, b1_sel = bsel.step(tree, changed, k == 0)
            # One generator per commit, so a resumed run plants the same defects.
            rng = random.Random(f"bcb-defect:{header['history_seed']}:{header['profile']}:{k}")
            base = subprocess.run([BMB, "verify", "--cache", cache, os.path.join(ws, "src")], capture_output=True, text=True)
            assert base.returncode == 0, f"commit {k} does not prove"
            if k == 0:
                continue
            row = {"k": k}
            planted = None
            targets = defect_targets(tree, commits[k - 1])
            for u, name in rng.sample(targets, len(targets)):
                r = subprocess.run([BMB, "mutate", "--def", f"m{u}::{name}", os.path.join(ws, "src")],
                                   capture_output=True, text=True, check=True)
                rows = [json.loads(x) for x in r.stdout.splitlines()]
                rows = [x for x in rows if not x.get("summary")]
                if rows:
                    i = rng.randrange(len(rows))
                    planted = (u, name, i, len(rows), rows[i])
                    break
            if planted is None:
                row["skipped"] = "no edited executable definition with a mutant"
                out.write(json.dumps(row) + "\n")
                continue
            u, name, i, n, m = planted
            row.update({"def": f"m{u}::{name}", "mutant": i, "mutants": n, "site": f"{m['before']} -> {m['after']}"})
            block = [b for b in rustify.blocks(m["text"]) if rustify.def_name(b) == name][0]
            # BMB: the gate (proof), then the remaining tests whose output could change.
            bpath = os.path.join(ws, "src", f"m{u}.bmb")
            open(bpath, "w", encoding="utf-8").write(m["text"])
            v = subprocess.run([BMB, "verify", "--cache", cache, os.path.join(ws, "src")], capture_output=True, text=True)
            proof = any(x.startswith('{"def"') and '"status":"failed"' in x for x in v.stdout.splitlines())
            row["proof"] = proof
            for arm, chosen in (("B1", b1_sel), ("B0", tests)):
                reach = {t for t in chosen if (u, name) in bsel.graph.closure(t)}
                caught = False
                if not proof and reach:
                    exe = os.path.join(ws, "defect_tests")
                    write_if_changed(os.path.join(ws, "driver", "bcb_main.bmb"), driver(reach))
                    measure([BMB, "build", "--cache", cache, os.path.join(ws, "src"), os.path.join(ws, "tests"),
                             os.path.join(ws, "driver"), "-o", exe])
                    caught = measure([exe], check=False)[2] != expected_output(tree, reach)
                row[arm] = proof or caught
                row[arm + "_tests"] = caught
            open(bpath, "w", encoding="utf-8").write(tree.text[("bmb", u)])
            # Rust: the arms' selected tests on the planted crate.
            rpath = os.path.join(rs, "src", f"m{u}.rs")
            open(rpath, "w", encoding="utf-8").write(
                rustify.replace_rust(tree.text[("rust", u)], name, rustify.translate(block)))
            _, _, exe = test_exe(rs, env)
            names = st["names"]
            for arm in ("R0", "R1", "R2"):
                row[arm] = tests_fail(exe, [names[t] for t in st[arm]], st["all"])
            open(rpath, "w", encoding="utf-8").write(tree.text[("rust", u)])
            out.write(json.dumps(row) + "\n")
            out.flush()


def machine():
    """Where the numbers come from (recorded in every result header)."""
    def out(cmd):
        try:
            return subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT).stdout.strip()
        except OSError:
            return None
    model = None
    try:
        model = next(x.split(":", 1)[1].strip() for x in open("/proc/cpuinfo") if x.startswith("model name"))
    except (OSError, StopIteration):
        pass
    return {"cpus": os.cpu_count(), "cpu": model, "rustc": out(["rustc", "-V"]), "cargo": out(["cargo", "-V"]),
            "z3": out(["z3", "--version"]), "commit": out(["git", "rev-parse", "HEAD"]),
            "dirty": bool(out(["git", "status", "--porcelain", "--untracked-files=no"])),
            "manifest": sha(os.path.join(HERE, "MANIFEST")) if os.path.exists(os.path.join(HERE, "MANIFEST")) else None,
            "bmb": BMB, "bmb_sha": sha(BMB) if os.path.exists(BMB) else None,
            "history_sha": None}


def run(args):
    header, commits = history.load(args.history)
    os.makedirs(args.work, exist_ok=True)
    with open(args.out, "a", encoding="utf-8") as out:
        out.write(json.dumps({"header": True, "history": os.path.basename(args.history), "profile": header["profile"],
                              "defs": header["defs"], "commits": header["commits"], "upto": args.upto, "from": args.start,
                              "events": header["events"], "b0_every": args.b0_every,
                              "machine": {**machine(), "history_sha": sha(args.history)},
                              "digest": commits[-1]["digest"] if commits else header["digest0"]}) + "\n")
        for arm in args.arms.split(","):
            if arm == "rust":
                rust_arm(header, commits, args, out)
            elif arm in ("b0", "b1"):
                bmb_arm(header, commits, args, out, arm)
            else:
                sys.exit(f"bcb: unknown arm {arm}")


# ------------------------------------------------------------------ freezing

# The measured histories (ADR-0042 §1): K = 1,000, history seed 1, gen.py defaults
# unless noted. Regenerating one must give the bytes MANIFEST records.
FROZEN = [
    ("body-heavy-1k", 1000, "body-heavy", []),
    ("contract-heavy-1k", 1000, "contract-heavy", []),
    ("body-only-1k", 1000, "body-only", []),
    ("body-heavy-1k-depth2", 1000, "body-heavy", ["--depth", "2"]),
    ("body-heavy-1k-depth6", 1000, "body-heavy", ["--depth", "6"]),
    ("body-heavy-10k", 10000, "body-heavy", []),
    ("contract-heavy-10k", 10000, "contract-heavy", []),
    ("body-only-10k", 10000, "body-only", []),
]
HARNESS = ("bcb.py", "history.py", "rtests.py", "rustify.py", "kills.py")


def sha(path):
    import hashlib
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def gen_frozen(outdir):
    os.makedirs(outdir, exist_ok=True)
    out = {}
    for name, n, profile, extra in FROZEN:
        path = os.path.join(outdir, name + ".jsonl")
        if not os.path.exists(path):
            history.main(["gen", path, "--defs", str(n), "--commits", "1000", "--profile", profile] + extra)
        out[name] = sha(path)
    return out


def manifest_text(hist):
    lines = [f"{sha(os.path.join(HERE, f))}  {f}" for f in HARNESS]
    lines += [f"{h}  histories/{name}.jsonl" for name, h in sorted(hist.items())]
    return "\n".join(lines) + "\n"


def manifest(args):
    path = os.path.join(HERE, "MANIFEST")
    if args.write:
        open(path, "w").write(manifest_text(gen_frozen(args.histories)))
        print("MANIFEST written")
        return 0
    recorded = dict(reversed(x.split("  ", 1)) for x in open(path).read().splitlines()) if os.path.exists(path) else {}
    bad = [f for f in HARNESS if recorded.get(f) != sha(os.path.join(HERE, f))]
    if args.histories:
        got = gen_frozen(args.histories)
        bad += [n for n, h in got.items() if recorded.get(f"histories/{n}.jsonl") != h]
    if bad:
        print("bcb: frozen inputs differ from MANIFEST: " + ", ".join(bad), file=sys.stderr)
        return 1
    print("MANIFEST ok")
    return 0


# --------------------------------------------------------------------- suite

# The measurement plan of ADR-0042 in order: (history, arm, B0 sampling).
SUITE = [
    ("body-only-1k", "rust", 0), ("body-only-1k", "b1", 0),
    ("body-heavy-1k", "rust", 0), ("body-heavy-1k", "b1", 0), ("body-heavy-1k", "b0", 50),
    ("body-heavy-1k", "defects", 0),
    ("contract-heavy-1k", "rust", 0), ("contract-heavy-1k", "b1", 0), ("contract-heavy-1k", "b0", 50),
    ("contract-heavy-1k", "defects", 0), ("body-only-1k", "defects", 0),
    ("body-heavy-1k-depth2", "rust", 0), ("body-heavy-1k-depth2", "b1", 0),
    ("body-heavy-1k-depth6", "rust", 0), ("body-heavy-1k-depth6", "b1", 0),
    ("body-heavy-10k", "rust", 0), ("body-heavy-10k", "b1", 0), ("body-heavy-10k", "b0", 250),
    ("contract-heavy-10k", "rust", 0), ("contract-heavy-10k", "b1", 0), ("contract-heavy-10k", "b0", 250),
    ("body-only-10k", "rust", 0), ("body-only-10k", "b1", 0),
]


def resume_point(path, arm, four, last):
    """(first commit still to measure, whether only its cold measurement is
    missing) for `path`, or None when the file is complete. A line cut off by a
    kill is dropped. An event commit with its warm row but not its cold one
    resumes with the cold measurement only: measuring it warm again would find
    its work already in the caches."""
    if not os.path.exists(path):
        return 0, False
    lines = []
    for x in open(path, encoding="utf-8").read().splitlines():
        try:
            lines.append(json.loads(x))
        except ValueError:
            break  # cut off mid-write: drop it and what follows
    with open(path, "w", encoding="utf-8") as f:
        f.write("".join(json.dumps(r) + "\n" for r in lines))
    rows = [r for r in lines if not r.get("header")]
    done = max((r["k"] for r in rows), default=-1)
    if arm not in ("defects", "b0") and done in four and not any(r["k"] == done and r["cold"] for r in rows):
        return done, True
    if done >= last:
        return None
    return done + 1, False


def suite(args):
    """Runs (or continues) SUITE: results in RESULTS/<history>.<arm>.jsonl,
    work directories in WORK/<history>-<arm>. The compiler is copied to
    WORK/bin/bmb on the first run and every later run uses that copy, so
    rebuilding the repository's compiler cannot change a measurement halfway."""
    global BMB
    if manifest(argparse.Namespace(write=False, histories=os.path.join(args.work, "histories"))):
        sys.exit("bcb suite: refusing to run")
    if machine()["dirty"] and not os.environ.get("BCB_ALLOW_DIRTY"):
        sys.exit("bcb suite: the repository has uncommitted changes; measure a committed tree")
    pin = os.path.join(args.work, "bin", "bmb")
    if not os.path.exists(pin):
        os.makedirs(os.path.dirname(pin), exist_ok=True)
        shutil.copy2(BMB, pin)
    BMB = pin
    os.makedirs(args.results, exist_ok=True)
    for name, arm, every in SUITE:
        hist = os.path.join(args.work, "histories", name + ".jsonl")
        header, _ = history.load(hist)
        out = os.path.join(args.results, f"{name}.{arm}.jsonl")
        point = resume_point(out, arm, set(header["events"]["four"]), header["commits"])
        if point is None:
            continue
        start, cold_only = point
        print(f"bcb suite: {name} {arm} from commit {start}{' (cold only)' if cold_only else ''}", flush=True)
        ns = argparse.Namespace(history=hist, work=os.path.join(args.work, f"{name}-{arm}"), out=out,
                                upto=None, start=start, arms=arm, b0_every=every or 1, cold_only=cold_only)
        if arm == "defects":
            defects(ns)
        else:
            run(ns)
    print("bcb suite: complete")


# ------------------------------------------------------------------- table

def load_rows(path):
    head, rows = None, []
    for line in open(path, encoding="utf-8"):
        r = json.loads(line)
        if r.get("header"):
            head = r
        else:
            rows.append(r)
    return head, rows


def cumulative(head, rows):
    """Arm -> {events e: cumulative CPU}, plus per-commit invalidation means."""
    warm = {}
    cold = {}
    seen = set()
    for r in rows:
        key = (r["arm"], r["k"], r["cold"])
        if key in seen:
            sys.exit(f"bcb: commit {r['k']} of arm {r['arm']} measured twice (overlapping --from runs?)")
        seen.add(key)
        if r["arm"] == "rust":
            base = r["build"]["cpu"] + r["test_build"]["cpu"]
            vals = {a: base + r[a]["cpu"] for a in ("R0", "R1", "R2")}
        elif r["arm"] == "b1":
            vals = {"B1": r["build"]["cpu"] + r["tests"]["cpu"]}
        else:
            vals = {"B0": r["build"]["cpu"] + r["tests"]["cpu"]}
        target = cold if r["cold"] and r["arm"] != "b0" else warm
        for a, v in vals.items():
            target.setdefault(a, {})[r["k"]] = v
    out = {}
    one = set(head["events"]["one"])
    for a, per in warm.items():
        if a == "B0":
            ks = sorted(per)
            mean = sum(per.values()) / len(per)
            n = max((r["k"] for r in rows if r["arm"] != "b0"), default=head["commits"]) + 1
            # Always cold: toolchain events change nothing.
            out[a] = {"0": mean * n, "1": mean * n, "4": mean * n, "estimated": True, "samples": len(ks)}
            continue
        c0 = sum(per.values())
        extra = {k: cold.get(a, {}).get(k, per[k]) - per[k] for k in per if k in cold.get(a, {})}
        out[a] = {"0": c0, "1": c0 + sum(v for k, v in extra.items() if k in one), "4": c0 + sum(extra.values())}
    return out


def detection(rows):
    """Arm -> (caught, planted) over the planted commits."""
    planted = [r for r in rows if "def" in r]
    return {a: (sum(r[a] for r in planted), len(planted)) for a in ("R0", "R1", "R2", "B0", "B1")}


def table(paths):
    dets = []
    lines = ["| 이력 | 정의 | 커밋 | 팔 | 누적 CPU(사건 0) | 사건 1 | 사건 4 | 커밋당 재검증 평균 |",
             "| --- | --- | --- | --- | --- | --- | --- | --- |"]
    verdicts = []
    groups = {}
    for p in paths:
        head, rows = load_rows(p)
        if head.get("defects"):
            g = groups.setdefault(("defects", head["history"]), [head, []])
        else:
            g = groups.setdefault(("run", head["history"]), [head, []])
        g[1] += rows
    for (kind, _), (head, rows) in sorted(groups.items()):
        if kind == "defects":
            dets.append((head, detection(rows), sum(r.get("proof", False) for r in rows)))
            continue
        cum = cumulative(head, rows)
        warm = [r for r in rows if not r["cold"]]
        inv = {}
        for r in warm:
            if r["k"] == 0:
                continue
            if r["arm"] == "rust":
                inv.setdefault("R1", []).append(r["R1"]["tests"])
                inv.setdefault("R2", []).append(r["R2"]["tests"])
                inv.setdefault("R2 covered", []).append(r["R2"]["covered_defs"])
            elif r["arm"] == "b1":
                inv.setdefault("B1", []).append(len(r["encoded_defs"]))
        mean = {a: sum(v) / len(v) for a, v in inv.items() if v}
        n = max((r["k"] for r in rows if r["arm"] != "b0"), default=head["commits"])
        for a in ("R0", "R1", "R2", "B0", "B1"):
            if a not in cum:
                continue
            c = cum[a]
            est = " (추정)" if c.get("estimated") else ""
            inv_s = {"R1": f"테스트 {mean.get('R1', 0):.1f}", "R2": f"테스트 {mean.get('R2', 0):.1f}·정의 {mean.get('R2 covered', 0):.1f}",
                     "B1": f"재증명 {mean.get('B1', 0):.1f}"}.get(a, "")
            ev = " | ".join(f"{c[e]:.1f}s" if e in c else "—" for e in ("1", "4"))
            lines.append(f"| {head['profile']} | {head['defs']} | {n} | {a} | {c['0']:.1f}s{est} | {ev} | {inv_s} |")
        if "B1" in cum and "R1" in cum:
            verdicts.append(f"- {head['profile']} N={head['defs']} K={n}: C_B1/C_R1 = {cum['B1']['0'] / cum['R1']['0']:.3f}"
                            f" (사건 4: {cum['B1']['4'] / cum['R1']['4']:.3f})")
        if "B1" in mean and "R2 covered" in mean:
            verdicts.append(f"- {head['profile']} N={head['defs']}: 커밋당 B1 재증명 {mean['B1']:.2f} 대 R2가 덮는 정의 {mean['R2 covered']:.2f}")
    if len(lines) == 2:
        lines = []  # defect files only
    if dets:
        lines += ["", "| 이력 | 정의 | 심은 결함 | R0 | R1 | R2 | B0 | B1 |", "| --- | --- | --- | --- | --- | --- | --- | --- |"]
        total = {a: [0, 0] for a in ("R0", "R1", "R2", "B0", "B1")}
        for head, d, _ in dets:
            n = d["R0"][1]
            lines.append(f"| {head['profile']} | {head['defs']} | {n} | " + " | ".join(
                f"{d[a][0]} ({100 * d[a][0] / max(n, 1):.0f}%)" for a in ("R0", "R1", "R2", "B0", "B1")) + " |")
            for a in total:
                total[a][0] += d[a][0]
                total[a][1] += d[a][1]
        verdicts.append(f"- 결함 검출(전체): B1 {total['B1'][0]}/{total['B1'][1]}, R2 {total['R2'][0]}/{total['R2'][1]}"
                        f" — 조건 4(B1 < R2) {'성립: 반증' if total['B1'][0] < total['R2'][0] else '불성립'}")
    return "\n".join(lines + [""] + verdicts) + "\n"


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("history")
    r.add_argument("--arms", default="rust,b1,b0")
    r.add_argument("--work", required=True)
    r.add_argument("--out", required=True)
    r.add_argument("--upto", type=int)
    r.add_argument("--b0-every", type=int, default=1)
    r.add_argument("--from", dest="start", type=int, default=0,
                   help="measure from commit K on; the work directory holds the run that stopped before K")
    d = sub.add_parser("defects")
    d.add_argument("history")
    d.add_argument("--work", required=True)
    d.add_argument("--out", required=True)
    d.add_argument("--upto", type=int)
    d.add_argument("--from", dest="start", type=int, default=0)
    m = sub.add_parser("manifest", help="check (or --write) the frozen harness and histories")
    m.add_argument("--write", action="store_true")
    m.add_argument("--histories", help="directory to (re)generate the frozen histories in and check them")
    su = sub.add_parser("suite", help="run or continue the whole measurement plan (SUITE)")
    su.add_argument("--work", required=True)
    su.add_argument("--results", required=True)
    t = sub.add_parser("table")
    t.add_argument("results", nargs="+")
    args = p.parse_args(argv)
    if args.cmd == "run":
        run(args)
    elif args.cmd == "defects":
        defects(args)
    elif args.cmd == "suite":
        suite(args)
    elif args.cmd == "manifest":
        sys.exit(manifest(args))
    else:
        sys.stdout.write(table(args.results))


if __name__ == "__main__":
    main()
