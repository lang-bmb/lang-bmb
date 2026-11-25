#!/usr/bin/env python3
"""Differential tests of the port (port map §3).

A case names two commands, the stage0 side `a` and the port side `b`, and
an input set. Each run starts both in an empty directory of its own with
the same environment and the same standard input, and compares standard
output, standard error, the exit code and the files they leave, byte for
byte. Nothing is normalised. A run that does not end within the case's
`timeout` (600 seconds by default) is ended, with everything it started,
and the case fails, whether one side or both ran out.

    difftest.py run CASES.json [--case NAME].. [--shard I/N] [--built DIR]
                               [--keep DIR] [--prebuilt NAME=PATH]..
                               [--excerpts DIR]
    difftest.py build CASES.json DIR [--jobs N] [--proof DIR]
    difftest.py shards CASES.json N
    difftest.py costs CASES.json RESULTS..

`run` prints one JSON line per case (with the seconds it took, and per side
the seconds and the peak resident memory of its largest process) and exits
1 when a case differs or cannot run. A case with `"digest": true` compares
outputs too large to hold: each side's standard output and error are hashed
as they arrive and its files as soon as it ends, in blocks of 64 KiB, and
its directory is removed before the other side starts, so nothing is in
memory whole or on disk twice; a difference is located to its block and
line. `--excerpts DIR` leaves in `DIR/CASE` the difference and the bytes
around it (with digests: the first bytes of each side and side b's first
block that differs), not the outputs. `--shard I/N` runs the I-th of N disjoint
parts of the cases, balanced by the seconds each case took when last
measured (`costs.json` next to the cases file; a case it does not list counts
as the mean): the cases in decreasing cost go one by one to the part with
the least so far, the earlier case first on a tie, and each part runs its
cases in file order. Every case is in exactly one part. `shards` prints the
parts and their expected seconds; `costs` rewrites `costs.json` from the
lines of runs over every case.

`build` makes every build into DIR once, `DIR/NAME`, as many at once as
the machine has cores (`--jobs`), each BMB build from a copy of the proof
cache `--proof` (the gate's `target/port-proof` by default); a build that fails leaves its error in
`DIR/NAME.error` and the exit code is 1. `run --built
DIR` takes the programs from there instead of building: a build that failed
or is missing fails the run that needs it, as a failed build does in `run`
alone (the gate builds once and runs the parts as concurrent jobs, so a
failed build is reported by the jobs, with the cases that needed it).
`--prebuilt NAME=PATH` uses the program at PATH as build NAME. The cases
file:

    {"builds": {NAME: {"bmb": FILE} | {"proved": FILE} | {"rustc": FILE}
                      | {"example": NAME}},
     "cases": [{"name": .., "component": .., "a": [ARGV], "b": [ARGV],
                "inputs": {..}, "digest": BOOL, "timeout": SECONDS}]}

`builds` are made once: `{"bmb": FILE}` (or a list of globs, the files of one
program; `"except": GLOB..` leaves files out) with `bmb build` (so the
program is proved), `{"proved": ..}` the same files as a proof cache
directory (`bmb verify --cache` from a copy of the gate's port proof, when
there is one; it fails unless every definition is proved),
`{"rustc": FILE}` with `rustc --edition 2021 -O`,
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
import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
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


BLOCK = 1 << 16  # the unit a digest locates a difference to
HEAD = 1 << 16  # the bytes a digest of a stream keeps from its start
FILE_HEAD = 1 << 13  # .. of a file (a side may leave thousands)


class Digest:
    """What a channel held, in constant memory however long it is: its
    size, its SHA-256, the SHA-256 of each block of BLOCK bytes with the
    count of line feeds before it, and its first bytes. Given the digest
    of the other side's channel (`against`), it also keeps its own first
    block that differs from that one, the only place the bytes of both
    sides cannot be read from the digests."""

    def __init__(self, against=None, head=HEAD):
        self.size = 0
        self.whole = hashlib.sha256()
        self.blocks = []
        self.lines = []
        self.head = bytearray()
        self.excerpt = None
        self._against = against
        self._head = head
        self._buf = bytearray()
        self._newlines = 0

    def feed(self, data):
        self.size += len(data)
        self.whole.update(data)
        if len(self.head) < self._head:
            self.head += data[: self._head - len(self.head)]
        self._buf += data
        while len(self._buf) >= BLOCK:
            self._block(bytes(self._buf[:BLOCK]))
            del self._buf[:BLOCK]

    def close(self):
        if self._buf:
            self._block(bytes(self._buf))
            self._buf = bytearray()
        self.sha256 = self.whole.hexdigest()
        return self

    def _block(self, data):
        k = len(self.blocks)
        h = hashlib.sha256(data).digest()
        self.blocks.append(h)
        self.lines.append(self._newlines)
        self._newlines += data.count(b"\n")
        other = self._against
        if other is not None and self.excerpt is None and (k >= len(other.blocks) or other.blocks[k] != h):
            self.excerpt = (k * BLOCK, data)

    def __eq__(self, other):
        return isinstance(other, Digest) and (self.size, self.sha256) == (other.size, other.sha256)


def digest_file(path, against=None):
    d = Digest(against, FILE_HEAD)
    with open(path, "rb") as f:
        while chunk := f.read(1 << 20):
            d.feed(chunk)
    return d.close()


def tree_digest(d, against):
    """The files under `d`, each as its Digest (against the file of the same
    name in `against`)."""
    out = {}
    for base, dirs, names in os.walk(d):
        dirs.sort()
        for n in sorted(names):
            p = os.path.join(base, n)
            rel = os.path.relpath(p, d)
            out[rel] = digest_file(p, against.get(rel))
    return out


def spawn(argv, stdin, cwd, timeout, out, err):
    """Runs `argv` in `cwd` with `stdin` and hands what it writes to `out`
    and `err` (each called with bytes) as it comes. The exit code ("timeout"
    when it was ended at `timeout` seconds, with everything it started) and
    the peak resident memory, in MiB, of the largest process it ran (itself
    or one it waited for)."""
    try:
        p = subprocess.Popen(
            argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            cwd=cwd, env=ENV, start_new_session=True,
        )
    except OSError as e:
        return f"cannot run: {e.strerror}", 0

    def pump(pipe, sink):
        with pipe:
            while chunk := pipe.read1(1 << 20):
                sink(chunk)

    def feed():
        try:
            with p.stdin:
                p.stdin.write(stdin)
        except OSError:
            pass  # it ended or closed its input before reading all of it

    threads = [threading.Thread(target=pump, args=(p.stdout, out)),
               threading.Thread(target=pump, args=(p.stderr, err)),
               threading.Thread(target=feed)]
    for t in threads:
        t.start()
    timer = threading.Timer(timeout, os.killpg, (p.pid, signal.SIGKILL))
    timer.start()
    _, status, usage = os.wait4(p.pid, 0)
    timed_out = not timer.is_alive()
    timer.cancel()
    p.returncode = os.waitstatus_to_exitcode(status)
    for t in threads:
        t.join()
    return ("timeout" if timed_out else p.returncode), round(usage.ru_maxrss / 1024)


def observe(argv, stdin, timeout, digest=False, against=None):
    """What one side did: stdout, stderr, exit code and the files it left,
    with its seconds and peak memory (`rss_mb`). With `digest` every channel
    is a Digest (against the channels of `against`, the other side's
    observation), so that no output is held whole in memory and none
    outlives the run's directory."""
    d = tempfile.mkdtemp(prefix="difftest-")
    start = time.monotonic()
    try:
        if digest:
            other = against or {"stdout": None, "stderr": None, "files": {}}
            seen = {"stdout": Digest(other["stdout"]), "stderr": Digest(other["stderr"])}
            code, rss = spawn(argv, stdin, d, timeout, seen["stdout"].feed, seen["stderr"].feed)
            seen["stdout"].close()
            seen["stderr"].close()
            seen["files"] = tree_digest(d, other["files"])
        else:
            out, err = bytearray(), bytearray()
            code, rss = spawn(argv, stdin, d, timeout, out.extend, err.extend)
            seen = {"stdout": bytes(out), "stderr": bytes(err), "files": tree(d)}
        seen.update({"exit": code, "rss_mb": rss, "seconds": time.monotonic() - start})
        return seen
    finally:
        shutil.rmtree(d, ignore_errors=True)


def differ(x, y):
    """Where two channels part (bytes or Digests); None when they do not."""
    if x == y:
        return None
    if isinstance(x, Digest):
        # The first block that differs, or where the shorter one ends.
        k = next((i for i, (p, q) in enumerate(zip(x.blocks, y.blocks)) if p != q), min(len(x.blocks), len(y.blocks)))
        lines = x.lines[k] if k < len(x.lines) else x._newlines
        return {
            "byte_from": k * BLOCK,
            "byte_to": min((k + 1) * BLOCK, max(x.size, y.size)),
            "line_from": lines + 1,
            "size": [x.size, y.size],
            "sha256": [x.sha256, y.sha256],
        }
    at = next((i for i in range(min(len(x), len(y))) if x[i] != y[i]), min(len(x), len(y)))
    return {
        "byte": at,
        "line": x[:at].count(b"\n") + 1,
        "a": x[max(0, at - 40) : at + 40].decode("utf-8", "replace"),
        "b": y[max(0, at - 40) : at + 40].decode("utf-8", "replace"),
    }


def first_difference(a, b):
    """None when the two observations are the same, else where they part."""
    if a["exit"] != b["exit"]:
        return {"what": "exit", "a": a["exit"], "b": b["exit"]}
    if a["exit"] == "timeout":
        return {"what": "timeout"}
    for stream in ("stdout", "stderr"):
        where = differ(a[stream], b[stream])
        if where:
            return {"what": stream, **where}
    if a["files"] != b["files"]:
        names = sorted(set(a["files"]) | set(b["files"]))
        name = next(n for n in names if a["files"].get(n) != b["files"].get(n))
        x, y = a["files"].get(name), b["files"].get(name)
        return {"what": "file", "name": name, **(differ(x, y) if x is not None and y is not None else {"only": "a" if y is None else "b"})}
    return None


PORT_PROOF = os.path.join(ROOT, "target", "port-proof")


def program_files(spec, key):
    """The files of one program: a file or a list of globs (in path order),
    without those `except` names."""
    globs = spec[key] if isinstance(spec[key], list) else [spec[key]]
    left = [os.path.join(ROOT, g) for g in spec.get("except", [])]
    files = sorted(p for g in globs for p in glob.glob(os.path.join(ROOT, g)))
    return globs, [p for p in files if not any(fnmatch.fnmatch(p, g) for g in left)]


def build(spec, out, bmb, cache=PORT_PROOF):
    """Builds one program; the error text when it cannot be built."""
    if "bmb" in spec or "proved" in spec:
        globs, files = program_files(spec, "bmb" if "bmb" in spec else "proved")
        if not files:
            return f"no file matches {globs}"
        if "bmb" in spec:
            # The gate proves the port once (check.sh "port proof"): a
            # build finds the proofs of the files it shares with it in this cache.
            cmd = [bmb, "build", "--cache", cache, *files, "-o", out]
        else:
            # A cache directory of the program proved: the given cache
            # (the gate's port proof, when there is one) and what is left.
            if os.path.isdir(cache):
                shutil.copytree(cache, out)
            cmd = [bmb, "verify", "--cache", out, *files]
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


def run_case(case, built, bmb, timeout=600, excerpts=None):
    runs, size = runs_of(case["inputs"])
    result = {"case": case["name"], "component": case.get("component"), "inputs": size, "runs": len(runs)}
    timeout = case.get("timeout", timeout)
    digest = case.get("digest", False)
    sides = {"a": {"seconds": 0.0, "rss_mb": 0}, "b": {"seconds": 0.0, "rss_mb": 0}}
    start = time.monotonic()
    for label, files, stdin in runs:
        a = observe(argv_of(case["a"], files, built, bmb), stdin, timeout, digest)
        b = observe(argv_of(case["b"], files, built, bmb), stdin, timeout, digest, a)
        for name, seen in (("a", a), ("b", b)):
            sides[name]["seconds"] += seen["seconds"]
            sides[name]["rss_mb"] = max(sides[name]["rss_mb"], seen["rss_mb"])
        diff = first_difference(a, b)
        if diff:
            diff["run"] = label
            result["same"] = False
            result["difference"] = diff
            if excerpts:
                write_excerpts(os.path.join(excerpts, case["name"]), diff, a, b)
            break
    else:
        result["same"] = True
    result["seconds"] = round(time.monotonic() - start, 1)
    for side in sides.values():
        side["seconds"] = round(side["seconds"], 1)
    result["sides"] = sides
    return result


EXCERPT = 1 << 16  # the bytes of each side an excerpt keeps around a difference


def write_excerpts(out, diff, a, b):
    """Leaves in `out` what a reader needs of a difference without the
    outputs themselves: `difference.json`, and of the channel that differs
    the bytes of each side around the first difference (whole outputs) or
    the first bytes of each side and side b's first block that differs
    (digests)."""
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "difference.json"), "w", encoding="utf-8") as f:
        json.dump(diff, f, indent=1, ensure_ascii=False)
        f.write("\n")
    what = diff["what"]
    if what in ("stdout", "stderr"):
        pair = (a[what], b[what])
    elif what == "file":
        pair = (a["files"].get(diff["name"]), b["files"].get(diff["name"]))
    else:
        return
    for side, x in zip("ab", pair):
        if x is None:
            continue
        if isinstance(x, Digest):
            with open(os.path.join(out, f"{side}.head"), "wb") as f:
                f.write(x.head)
            if x.excerpt:
                at, data = x.excerpt
                with open(os.path.join(out, f"{side}.from-{at}"), "wb") as f:
                    f.write(data)
        else:
            at = max(0, diff.get("byte", 0) - EXCERPT // 2)
            with open(os.path.join(out, f"{side}.from-{at}"), "wb") as f:
                f.write(x[at : at + EXCERPT])


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


def build_all(spec, names, out_dir, bmb, jobs=1, proof=PORT_PROOF):
    """Builds `names` into `out_dir`; prints a line for each that fails and
    leaves its error in `NAME.error`. Whether all were built.

    With more than one job the builds run concurrently, and each `bmb` build
    reads and writes a copy of its own of the proof cache, since a build
    rewrites the cache it is given; one at a time they share the cache."""
    out_dir = os.path.abspath(out_dir)

    def one(name):
        out = os.path.join(out_dir, name)
        for stale in (out, out + ".error"):
            if os.path.isdir(stale):
                shutil.rmtree(stale)
            elif os.path.exists(stale):
                os.remove(stale)
        b = spec["builds"][name]
        cache = proof
        if jobs > 1 and ("bmb" in b or "proved" in b):
            cache = os.path.join(out_dir, f".cache-{name}")
            shutil.rmtree(cache, ignore_errors=True)
            if os.path.isdir(proof):
                shutil.copytree(proof, cache)
        try:
            err = build(b, out, bmb, cache)
        finally:
            if cache != proof:
                shutil.rmtree(cache, ignore_errors=True)
        if err:
            with open(out + ".error", "w", encoding="utf-8") as f:
                f.write(err)
        return err

    # The programs of many files first: they take longest.
    def size(name):
        b = spec["builds"][name].get("bmb", spec["builds"][name].get("proved", []))
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
        elif not os.path.exists(out):
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


def run(path, only, bmb, keep, prebuilt=None, shard=None, built_dir=None, excerpts=None):
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
            result = run_case(case, built, bmb, excerpts=excerpts)
            print(json.dumps(result, ensure_ascii=False), flush=True)
            failed = failed or not result["same"]
    finally:
        if out_dir and not keep:
            shutil.rmtree(out_dir, ignore_errors=True)
    return 1 if failed else 0


def build_cmd(path, out_dir, bmb, jobs, proof=PORT_PROOF):
    spec = load(path)
    wanted = wanted_builds(spec["cases"])
    if unknown_builds(spec, wanted):
        return 2
    os.makedirs(out_dir, exist_ok=True)
    return 0 if build_all(spec, wanted, out_dir, bmb, jobs, os.path.abspath(proof)) else 1


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
        "about": "Seconds of each case in one run of scripts/difftest.py (the gate's cases with the debug compiler, "
        "the nightly ones with the optimized one), written by `difftest.py costs`. Only how `--shard` balances "
        "its parts depends on them.",
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
    r.add_argument("--excerpts", metavar="DIR", help="leave in DIR/CASE the bytes around each difference")
    b = sub.add_parser("build")
    b.add_argument("cases")
    b.add_argument("dir")
    b.add_argument("--bmb", default=default_bmb)
    b.add_argument("--jobs", type=int, default=os.cpu_count() or 1, help="builds at once (default: the cores)")
    b.add_argument("--proof", default=PORT_PROOF, metavar="DIR", help="the proof cache builds start from (default: the gate's)")
    s = sub.add_parser("shards")
    s.add_argument("cases")
    s.add_argument("n", type=int)
    c = sub.add_parser("costs")
    c.add_argument("cases")
    c.add_argument("results", nargs="+")
    args = ap.parse_args()
    if args.cmd == "build":
        return build_cmd(args.cases, args.dir, os.path.abspath(args.bmb), args.jobs, args.proof)
    if args.cmd == "shards":
        return shards_cmd(args.cases, args.n)
    if args.cmd == "costs":
        return costs_cmd(args.cases, args.results)
    if args.keep and args.built:
        ap.error("--keep and --built do not go together")
    prebuilt = dict(p.split("=", 1) for p in args.prebuilt)
    return run(args.cases, args.case, os.path.abspath(args.bmb), args.keep, prebuilt, args.shard, args.built,
               args.excerpts)


if __name__ == "__main__":
    sys.exit(main())
