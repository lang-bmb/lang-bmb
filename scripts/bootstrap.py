#!/usr/bin/env python3
"""The bootstrap chain of the port, in all-checks mode (backlog 4E-02, E1).

stage1 is the compiler written in BMB (`selfhost/*.bmb`) built by stage0
(`bmb emit-llvm --all-checks` and clang), stage2 the same source built by
stage1, stage3 by stage2 (ADR-0037). The early fixed point holds when the IR
stage2 writes for the source is the IR stage1 wrote (stage2 IR = stage3 IR):
the compiler reproduces itself.

    bootstrap.py run [--keep DIR] [--difftest] [--json]
        builds the chain and compares the IR of the three stages. Exit 1
        when stage2's IR differs from stage1's (no fixed point). With
        `--difftest`, stage2 then stands in for stage1 in the port's
        differential cases (`tests/port/cases.json`): exit 1 when a case
        differs. `--json` prints the measurements as one JSON object.

Standard library only.
"""
import argparse
import glob
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CASES = os.path.join(ROOT, "tests", "port", "cases.json")


def sources():
    return sorted(glob.glob(os.path.join(ROOT, "selfhost", "*.bmb")))


def timed(cmd):
    """Runs `cmd`: (seconds, error text or None)."""
    t = time.monotonic()
    r = subprocess.run(cmd, capture_output=True)
    dt = time.monotonic() - t
    if r.returncode != 0:
        text = (r.stdout + r.stderr).decode("utf-8", "replace")
        return dt, f"{' '.join(cmd[:3])} ..: exit {r.returncode}\n{text[-2000:]}"
    return dt, None


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def chain(bmb, out):
    """Builds stage1, stage2 and stage3's IR under `out`: the measurements,
    or an error text."""
    files = sources()
    m = {"sources": len(files), "source_bytes": sum(os.path.getsize(f) for f in files)}
    compiler = bmb
    for n in (1, 2, 3):
        ll = os.path.join(out, f"stage{n}.ll")
        dt, err = timed([compiler, "emit-llvm", "--all-checks", *files, "-o", ll])
        if err:
            return None, err
        m[f"stage{n - 1}_emit_s"] = round(dt, 2)
        m[f"stage{n}_ir_bytes"] = os.path.getsize(ll)
        m[f"stage{n}_ir_sha256"] = sha(ll)
        if n == 3:
            break
        exe = os.path.join(out, f"stage{n}")
        # The flags of `bmb build` (compiler/src/toolchain.rs).
        dt, err = timed(["clang", "-O2", "-Wno-override-module", "-o", exe, ll])
        if err:
            return None, err
        m[f"stage{n}_clang_s"] = round(dt, 2)
        m[f"stage{n}_bytes"] = os.path.getsize(exe)
        compiler = exe
    m["stage1_ir_is_stage2_ir"] = m["stage1_ir_sha256"] == m["stage2_ir_sha256"]
    m["fixed_point"] = m["stage2_ir_sha256"] == m["stage3_ir_sha256"]
    return m, None


def run(args):
    bmb = os.path.abspath(args.bmb)
    out = args.keep or tempfile.mkdtemp(prefix="bootstrap-")
    os.makedirs(out, exist_ok=True)
    try:
        m, err = chain(bmb, out)
        if err:
            print(f"bootstrap: {err}", file=sys.stderr)
            return 1
        ok = m["fixed_point"]
        if ok and args.difftest:
            cmd = [sys.executable, os.path.join(ROOT, "scripts", "difftest.py"), "run", CASES]
            cmd += ["--bmb", bmb, "--prebuilt", f"stage1={os.path.join(out, 'stage2')}"]
            t = time.monotonic()
            r = subprocess.run(cmd, capture_output=True, text=True)
            m["difftest_s"] = round(time.monotonic() - t, 2)
            rows = [json.loads(l) for l in r.stdout.splitlines() if l.startswith("{")]
            m["difftest_cases"] = len(rows)
            m["difftest_differing"] = [x.get("case", x.get("build")) for x in rows if not x.get("same")]
            if r.returncode != 0:
                ok = False
                sys.stderr.write(r.stdout[-4000:] + r.stderr[-2000:])
        if args.json:
            print(json.dumps(m, ensure_ascii=False, indent=2))
        else:
            print(
                f"bootstrap: stage1 IR {'=' if m['stage1_ir_is_stage2_ir'] else '!='} stage2 IR "
                f"{'=' if m['fixed_point'] else '!='} stage3 IR ({m['stage2_ir_bytes']} bytes)"
            )
        if not m["fixed_point"]:
            print("bootstrap: no fixed point: stage2 and stage3 write different IR", file=sys.stderr)
        return 0 if ok else 1
    finally:
        if not args.keep:
            shutil.rmtree(out, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    r.add_argument("--keep", help="keep the stages in DIR")
    r.add_argument("--difftest", action="store_true")
    r.add_argument("--json", action="store_true")
    return run(ap.parse_args())


if __name__ == "__main__":
    sys.exit(main())
