#!/usr/bin/env python3
"""Differential tests of the port (port map §3).

A case names two commands, the stage0 side `a` and the port side `b`, and
an input set. Each run starts both in an empty directory of its own with
the same environment and the same standard input, and compares standard
output, standard error, the exit code and the files they leave, byte for
byte. Nothing is normalised.

    difftest.py run CASES.json [--case NAME].. [--shard I/N] [--built DIR]
                               [--keep DIR] [--prebuilt NAME=PATH]..
    difftest.py build CASES.json DIR [--jobs N]
    difftest.py shards CASES.json N
    difftest.py costs CASES.json RESULTS..

`run` prints one JSON line per case (with the seconds it took) and exits 1
when a case differs or cannot run. `--shard I/N` runs the I-th of N disjoint
parts of the cases, balanced by the seconds each case took when last
measured (`costs.json` next to the cases file; a case it does not list counts
as the mean): the cases in decreasing cost go one by one to the part with
the least so far, the earlier case first on a tie, and each part runs its
cases in file order. Every case is in exactly one part. `shards` prints the
parts and their expected seconds; `costs` rewrites `costs.json` from the
lines of runs over every case.

`build` makes every build into DIR once, `DIR/NAME`, as many at once as
the machine has cores (`--jobs`); a build that fails leaves its error in
`DIR/NAME.error` and the exit code is 1. `run --built
DIR` takes the programs from there instead of building: a build that failed
or is missing fails the run that needs it, as a failed build does in `run`
alone (the gate builds once and runs the parts as concurrent jobs, so a
failed build is reported by the jobs, with the cases that needed it).
`--prebuilt NAME=PATH` uses the program at PATH as build NAME. The cases
file:

    {"builds": {NAME: {"bmb": FILE} | {"rustc": FILE} | {"example": NAME}},
     "cases": [{"name": .., "component": .., "a": [ARGV], "b": [ARGV],
                "inputs": {..}}]}

`builds` are made once: `{"bmb": FILE}` (or a list of globs, the files of one
program; `"except": GLOB..` leaves files out) with `bmb build` (so the
program is proved), `{"rustc": FILE}` with `rustc --edition 2021 -O`,
`{"example": NAME}` with `cargo build --example NAME` in `compiler/` (a
reference that calls the compiler's own functions). In an argv,
`{build:NAME}` is a built program, `{bmb}` the compiler under test and
`{inputs}` the run's files (it becomes several arguments); a word that
starts with `{root}/` is a path in the repository (a run's directory is a
temporary one).

Input sets (paths are relative to the repository):
    {"files": GLOB.., "chunk": N}   git-tracked files matching any glob, N
                                    per run, as `{inputs}`
                                    (`"except": GLOB..` leaves files out:
                                    the case says which and the reason is
                                    written where the case is described)
    {"stdin_lines_of": GLOB..}      one run: the lines of those files, in
                                    path order, as standard input
    {"stdin": FILE..}               one run per file, as standard input
    {"none": true}                  one run without input
"""
import argparse
import concurrent.futures
import fnmatch
import glob
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENV = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "LC_ALL": "C"}


def tracked(globs):
    out = subprocess.run(
        ["git", "ls-files", "-z"], cwd=ROOT, check=True, capture_output=True
    ).stdout
    names = [n for n in out.decode().split("\0") if n]
    hit = [
        n
        for n in names
        if any(fnmatch.fnmatch(n, g) for g in globs)
        and os.path.isfile(os.path.join(ROOT, n))
    ]
    return sorted(hit)


def runs_of(inputs):
    """The runs of an input set: (label, files, stdin bytes)."""
    if "files" in inputs:
        names = tracked(inputs["files"])
        left_out = inputs.get("except", [])
        names = [p for p in names if not any(fnmatch.fnmatch(p, g) for g in left_out)]
        n = inputs.get("chunk", 100)
        return [
            (f"{names[i]}..", [os.path.join(ROOT, p) for p in names[i : i + n]], b"")
            for i in range(0, len(names), n)
        ], len(names)
    if "stdin_lines_of" in inputs:
        names = tracked(inputs["stdin_lines_of"])
        data = b""
        for p in names:
            with open(os.path.join(ROOT, p), "rb") as f:
                text = f.read()
            data += text if text.endswith(b"\n") or not text else text + b"\n"
        return [("lines", [], data)], data.count(b"\n")
    if "stdin" in inputs:
        runs = []
        for p in inputs["stdin"]:
            with open(os.path.join(ROOT, p), "rb") as f:
                runs.append((p, [], f.read()))
        return runs, len(runs)
    if inputs.get("none"):
        return [("-", [], b"")], 0
    raise ValueError(f"unknown input set: {sorted(inputs)}")


def argv_of(template, files, built, bmb):
    out = []
    for word in template:
        if word == "{inputs}":
            out.extend(files)
        elif word == "{bmb}":
            out.append(bmb)
        elif word.startswith("{build:") and word.endswith("}"):
            out.append(built[word[7:-1]])
        elif word.startswith("{root}/"):
            out.append(os.path.join(ROOT, word[7:]))
        else:
            out.append(word)
    return out


def tree(d):
    """The files under `d`: relative path -> bytes."""
    out = {}
    for base, _, names in os.walk(d):
        for n in names:
            p = os.path.join(base, n)
            with open(p, "rb") as f:
                out[os.path.relpath(p, d)] = f.read()
    return out


def observe(argv, stdin, timeout):
    """What one side did: stdout, stderr, exit code and the files it left."""
    d = tempfile.mkdtemp(prefix="difftest-")
    try:
        try:
            r = subprocess.run(
                argv, input=stdin, cwd=d, env=ENV, capture_output=True, timeout=timeout
            )
            seen = {"stdout": r.stdout, "stderr": r.stderr, "exit": r.returncode}
        except subprocess.TimeoutExpired:
            seen = {"stdout": b"", "stderr": b"", "exit": "timeout"}
        except OSError as e:
            seen = {"stdout": b"", "stderr": b"", "exit": f"cannot run: {e.strerror}"}
        seen["files"] = tree(d)
        return seen
    finally:
        shutil.rmtree(d, ignore_errors=True)


def first_difference(a, b):
    """None when the two observations are the same, else where they part."""
    if a["exit"] != b["exit"]:
        return {"what": "exit", "a": a["exit"], "b": b["exit"]}
    for stream in ("stdout", "stderr"):
        x, y = a[stream], b[stream]
        if x != y:
            at = next((i for i in range(min(len(x), len(y))) if x[i] != y[i]), min(len(x), len(y)))
            line = x[:at].count(b"\n") + 1
            return {
                "what": stream,
                "byte": at,
                "line": line,
                "a": x[max(0, at - 40) : at + 40].decode("utf-8", "replace"),
                "b": y[max(0, at - 40) : at + 40].decode("utf-8", "replace"),
            }
    if a["files"] != b["files"]:
        names = sorted(set(a["files"]) | set(b["files"]))
        name = next(n for n in names if a["files"].get(n) != b["files"].get(n))
        return {"what": "file", "name": name}
    return None


PORT_PROOF = os.path.join(ROOT, "target", "port-proof")


def build(spec, out, bmb, cache=PORT_PROOF):
    """Builds one program; the error text when it cannot be built."""
    if "bmb" in spec:
        # One file, or globs of the files of one program (in path order).
        globs = spec["bmb"] if isinstance(spec["bmb"], list) else [spec["bmb"]]
        left = [os.path.join(ROOT, g) for g in spec.get("except", [])]
        files = sorted(p for g in globs for p in glob.glob(os.path.join(ROOT, g)))
        files = [p for p in files if not any(fnmatch.fnmatch(p, g) for g in left)]
        if not files:
            return f"no file matches {globs}"
        # The gate proves the port once (check.sh "port proof"): a
        # build finds the proofs of the files it shares with it in this cache.
        cmd = [bmb, "build", "--cache", cache, *files, "-o", out]
    elif "rustc" in spec:
        cmd = ["rustc", "--edition", "2021", "-O", "--cap-lints", "allow"]
        cmd += [os.path.join(ROOT, spec["rustc"]), "-o", out]
    elif "example" in spec:
        # An example of the compiler's crate (`compiler/Cargo.toml`): a
        # program that calls stage0's own functions. cargo builds it in the
        # profile of the compiler under test, next to it.
        where = os.path.dirname(os.path.abspath(bmb))
        cmd = ["cargo", "build", "--quiet", "--example", spec["example"]]
        if os.path.basename(where) == "release":
            cmd.append("--release")
        r = subprocess.run(cmd, capture_output=True, cwd=os.path.join(ROOT, "compiler"))
        made = os.path.join(where, "examples", spec["example"])
        if r.returncode != 0 or not os.path.exists(made):
            return (r.stdout + r.stderr).decode("utf-8", "replace")[-2000:] or f"no {made}"
        shutil.copy2(made, out)
        return None
    else:
        return f"unknown build: {sorted(spec)}"
    r = subprocess.run(cmd, capture_output=True, cwd=os.path.dirname(out))
    if r.returncode != 0 or not os.path.exists(out):
        text = (r.stdout + r.stderr).decode("utf-8", "replace")
        errors = [l for l in text.splitlines() if '"verified"' not in l]
        return "\n".join(errors[-20:]) or f"exit {r.returncode}"
    return None


def run_case(case, built, bmb, timeout=600):
    runs, size = runs_of(case["inputs"])
    result = {"case": case["name"], "component": case.get("component"), "inputs": size, "runs": len(runs)}
    start = time.monotonic()
    for label, files, stdin in runs:
        a = observe(argv_of(case["a"], files, built, bmb), stdin, timeout)
        b = observe(argv_of(case["b"], files, built, bmb), stdin, timeout)
        diff = first_difference(a, b)
        if diff:
            diff["run"] = label
            result["same"] = False
            result["difference"] = diff
            break
    else:
        result["same"] = True
    result["seconds"] = round(time.monotonic() - start, 1)
    return result


def costs_path(path):
    return os.path.join(os.path.dirname(os.path.abspath(path)), "costs.json")


def load_costs(path):
    """The recorded seconds of each case (an empty table without the file)."""
    try:
        with open(costs_path(path), encoding="utf-8") as f:
            return json.load(f)["seconds"]
    except FileNotFoundError:
        return {}


def shards(cases, costs, n):
    """The cases (indices into `cases`) of each of `n` parts, in file order.

    Longest processing time first: the cases in decreasing cost, the earlier
    case first on a tie, each to the part with the least cost so far, the
    lower part first on a tie. A case without a recorded cost counts as the
    mean of those recorded (1 without any)."""
    if n < 1:
        raise ValueError("the number of parts is at least 1")
    known = [costs[c["name"]] for c in cases if c["name"] in costs]
    default = sum(known) / len(known) if known else 1.0
    cost = [costs.get(c["name"], default) for c in cases]
    parts = [[] for _ in range(n)]
    loads = [0.0] * n
    for i in sorted(range(len(cases)), key=lambda i: (-cost[i], i)):
        k = min(range(n), key=lambda k: (loads[k], k))
        parts[k].append(i)
        loads[k] += cost[i]
    return [sorted(p) for p in parts], loads


def parse_shard(text):
    i, _, n = text.partition("/")
    if not (i.isdigit() and n.isdigit() and 1 <= int(i) <= int(n)):
        raise ValueError(f"--shard takes I/N with 1 <= I <= N, not {text!r}")
    return int(i), int(n)


def selected(spec, path, only, shard):
    """The cases to run, and the names asked for that no case has."""
    cases = spec["cases"]
    if shard:
        i, n = shard
        parts, _ = shards(cases, load_costs(path), n)
        cases = [cases[k] for k in parts[i - 1]]
    picked = [c for c in cases if not only or c["name"] in only]
    return picked, set(only) - {c["name"] for c in picked}


def wanted_builds(cases):
    words = [w for c in cases for w in c["a"] + c["b"]]
    return {w[7:-1] for w in words if w.startswith("{build:")}


def build_all(spec, names, out_dir, bmb, jobs=1):
    """Builds `names` into `out_dir`; prints a line for each that fails and
    leaves its error in `NAME.error`. Whether all were built.

    With more than one job the builds run concurrently, and each `bmb` build
    reads and writes a copy of its own of the proof cache, since a build
    rewrites the cache it is given; one at a time they share the cache."""
    out_dir = os.path.abspath(out_dir)

    def one(name):
        out = os.path.join(out_dir, name)
        for stale in (out, out + ".error"):
            if os.path.exists(stale):
                os.remove(stale)
        b = spec["builds"][name]
        cache = PORT_PROOF
        if jobs > 1 and "bmb" in b:
            cache = os.path.join(out_dir, f".cache-{name}")
            shutil.rmtree(cache, ignore_errors=True)
            if os.path.isdir(PORT_PROOF):
                shutil.copytree(PORT_PROOF, cache)
        try:
            err = build(b, out, bmb, cache)
        finally:
            if cache != PORT_PROOF:
                shutil.rmtree(cache, ignore_errors=True)
        if err:
            with open(out + ".error", "w", encoding="utf-8") as f:
                f.write(err)
        return err

    # The programs of many files first: they take longest.
    def size(name):
        b = spec["builds"][name].get("bmb", [])
        return -len(b) if isinstance(b, list) else 0

    order = sorted(names, key=lambda n: (size(n), n))
    with concurrent.futures.ThreadPoolExecutor(max(1, jobs)) as pool:
        errors = dict(zip(order, pool.map(one, order)))
    for name in sorted(names):
        if errors[name]:
            print(json.dumps({"build": name, "error": errors[name]}, ensure_ascii=False), flush=True)
    return not any(errors.values())


def from_built(names, built_dir):
    """The programs `names` in `built_dir` (made by `build`), and a line for
    each that failed or is missing there."""
    built, errors = {}, []
    for name in sorted(names):
        out = os.path.join(os.path.abspath(built_dir), name)
        if os.path.exists(out + ".error"):
            with open(out + ".error", encoding="utf-8") as f:
                errors.append({"build": name, "error": f.read()})
        elif not os.path.isfile(out):
            errors.append({"build": name, "error": f"not built in {built_dir}"})
        else:
            built[name] = out
    return built, errors


def load(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def unknown_builds(spec, names):
    missing = sorted(n for n in names if n not in spec.get("builds", {}))
    for name in missing:
        print(f"difftest: no build named {name}", file=sys.stderr)
    return missing


def run(path, only, bmb, keep, prebuilt=None, shard=None, built_dir=None):
    spec = load(path)
    cases, missing = selected(spec, path, only, shard)
    if missing:
        print(f"difftest: no such case: {', '.join(sorted(missing))}", file=sys.stderr)
        return 2
    wanted = wanted_builds(cases)
    if unknown_builds(spec, wanted):
        return 2
    # A program built elsewhere stands in (the bootstrap chain's stage2 for
    # stage1).
    built = {n: os.path.abspath(p) for n, p in (prebuilt or {}).items() if n in wanted}
    rest = wanted - set(built)
    out_dir = None
    try:
        if built_dir:
            found, errors = from_built(rest, built_dir)
            built.update(found)
            for e in errors:
                print(json.dumps(e, ensure_ascii=False))
            if errors:
                return 1
        else:
            out_dir = keep or tempfile.mkdtemp(prefix="difftest-build-")
            os.makedirs(out_dir, exist_ok=True)
            if not build_all(spec, rest, out_dir, bmb):
                return 1
            built.update({n: os.path.join(os.path.abspath(out_dir), n) for n in rest})
        failed = False
        for case in cases:
            result = run_case(case, built, bmb)
            print(json.dumps(result, ensure_ascii=False), flush=True)
            failed = failed or not result["same"]
    finally:
        if out_dir and not keep:
            shutil.rmtree(out_dir, ignore_errors=True)
    return 1 if failed else 0


def build_cmd(path, out_dir, bmb, jobs):
    spec = load(path)
    wanted = wanted_builds(spec["cases"])
    if unknown_builds(spec, wanted):
        return 2
    os.makedirs(out_dir, exist_ok=True)
    return 0 if build_all(spec, wanted, out_dir, bmb, jobs) else 1


def shards_cmd(path, n):
    spec = load(path)
    parts, loads = shards(spec["cases"], load_costs(path), n)
    for k, (part, secs) in enumerate(zip(parts, loads), 1):
        names = " ".join(spec["cases"][i]["name"] for i in part)
        print(f"{k}/{n} {secs:7.1f}s {len(part):3} cases: {names}")
    return 0


def costs_cmd(path, results):
    """Rewrites costs.json from result lines that cover every case."""
    names = [c["name"] for c in load(path)["cases"]]
    seconds = {}
    for r in results:
        with open(r, encoding="utf-8") as f:
            for line in f:
                row = json.loads(line) if line.startswith("{") else {}
                if "case" in row and "seconds" in row:
                    seconds[row["case"]] = row["seconds"]
    missing = [n for n in names if n not in seconds]
    if missing:
        print(f"difftest: no seconds for: {', '.join(missing)}", file=sys.stderr)
        return 1
    table = {
        "about": "Seconds of each case in one run of scripts/difftest.py with the debug compiler, "
        "written by `difftest.py costs`. Only how `--shard` balances its parts depends on them.",
        "seconds": {n: seconds[n] for n in names},
    }
    with open(costs_path(path), "w", encoding="utf-8") as f:
        json.dump(table, f, indent=2, ensure_ascii=False)
        f.write("\n")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    default_bmb = os.path.join(ROOT, "target", "debug", "bmb")
    r = sub.add_parser("run")
    r.add_argument("cases")
    r.add_argument("--case", action="append", default=[])
    r.add_argument("--shard", type=parse_shard, metavar="I/N", help="run the I-th of N parts of the cases")
    r.add_argument("--bmb", default=default_bmb)
    r.add_argument("--keep", help="keep the built programs in DIR")
    r.add_argument("--built", metavar="DIR", help="take the programs from DIR (made by `build`)")
    r.add_argument("--prebuilt", action="append", default=[], metavar="NAME=PATH", help="use the program at PATH as build NAME")
    b = sub.add_parser("build")
    b.add_argument("cases")
    b.add_argument("dir")
    b.add_argument("--bmb", default=default_bmb)
    b.add_argument("--jobs", type=int, default=os.cpu_count() or 1, help="builds at once (default: the cores)")
    s = sub.add_parser("shards")
    s.add_argument("cases")
    s.add_argument("n", type=int)
    c = sub.add_parser("costs")
    c.add_argument("cases")
    c.add_argument("results", nargs="+")
    args = ap.parse_args()
    if args.cmd == "build":
        return build_cmd(args.cases, args.dir, os.path.abspath(args.bmb), args.jobs)
    if args.cmd == "shards":
        return shards_cmd(args.cases, args.n)
    if args.cmd == "costs":
        return costs_cmd(args.cases, args.results)
    if args.keep and args.built:
        ap.error("--keep and --built do not go together")
    prebuilt = dict(p.split("=", 1) for p in args.prebuilt)
    return run(args.cases, args.case, os.path.abspath(args.bmb), args.keep, prebuilt, args.shard, args.built)


if __name__ == "__main__":
    sys.exit(main())
