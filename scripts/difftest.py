#!/usr/bin/env python3
"""Differential tests of the port (port map §3, backlog 4D-15).

A case names two commands, the stage0 side `a` and the port side `b`, and
an input set. Each run starts both in an empty directory of its own with
the same environment and the same standard input, and compares standard
output, standard error, the exit code and the files they leave, byte for
byte. Nothing is normalised.

    difftest.py run CASES.json [--case NAME]... [--keep DIR]

prints one JSON line per case and exits 1 when a case differs or cannot
run. The cases file:

    {"builds": {NAME: {"bmb": FILE} | {"rustc": FILE}},
     "cases": [{"name": .., "component": .., "a": [ARGV], "b": [ARGV],
                "inputs": {..}}]}

`builds` are made once: `{"bmb": FILE}` with `bmb build` (so the program is
proved), `{"rustc": FILE}` with `rustc --edition 2021 -O`. In an argv,
`{build:NAME}` is a built program, `{bmb}` the compiler under test and
`{inputs}` the run's files (it becomes several arguments).

Input sets (paths are relative to the repository):
    {"files": GLOB.., "chunk": N}   git-tracked files matching any glob, N
                                    per run, as `{inputs}`
    {"stdin_lines_of": GLOB..}      one run: the lines of those files, in
                                    path order, as standard input
    {"stdin": FILE..}               one run per file, as standard input
    {"none": true}                  one run without input
"""
import argparse
import fnmatch
import json
import os
import shutil
import subprocess
import sys
import tempfile

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


def build(spec, out, bmb):
    """Builds one program; the error text when it cannot be built."""
    if "bmb" in spec:
        cmd = [bmb, "build", os.path.join(ROOT, spec["bmb"]), "-o", out]
    elif "rustc" in spec:
        cmd = ["rustc", "--edition", "2021", "-O", "--cap-lints", "allow"]
        cmd += [os.path.join(ROOT, spec["rustc"]), "-o", out]
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
    for label, files, stdin in runs:
        a = observe(argv_of(case["a"], files, built, bmb), stdin, timeout)
        b = observe(argv_of(case["b"], files, built, bmb), stdin, timeout)
        diff = first_difference(a, b)
        if diff:
            diff["run"] = label
            result["same"] = False
            result["difference"] = diff
            return result
    result["same"] = True
    return result


def run(path, only, bmb, keep):
    with open(path, encoding="utf-8") as f:
        spec = json.load(f)
    cases = [c for c in spec["cases"] if not only or c["name"] in only]
    missing = set(only) - {c["name"] for c in cases}
    if missing:
        print(f"difftest: no such case: {', '.join(sorted(missing))}", file=sys.stderr)
        return 2
    words = [w for c in cases for w in c["a"] + c["b"]]
    wanted = {w[7:-1] for w in words if w.startswith("{build:")}
    out_dir = keep or tempfile.mkdtemp(prefix="difftest-build-")
    os.makedirs(out_dir, exist_ok=True)
    failed = False
    try:
        built = {}
        for name in sorted(wanted):
            if name not in spec.get("builds", {}):
                print(f"difftest: no build named {name}", file=sys.stderr)
                return 2
            built[name] = os.path.join(os.path.abspath(out_dir), name)
            err = build(spec["builds"][name], built[name], bmb)
            if err:
                print(json.dumps({"build": name, "error": err}, ensure_ascii=False))
                failed = True
        if failed:
            return 1
        for case in cases:
            result = run_case(case, built, bmb)
            print(json.dumps(result, ensure_ascii=False))
            failed = failed or not result["same"]
    finally:
        if not keep:
            shutil.rmtree(out_dir, ignore_errors=True)
    return 1 if failed else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("cases")
    r.add_argument("--case", action="append", default=[])
    r.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    r.add_argument("--keep", help="keep the built programs in DIR")
    args = ap.parse_args()
    return run(args.cases, args.case, os.path.abspath(args.bmb), args.keep)


if __name__ == "__main__":
    sys.exit(main())
