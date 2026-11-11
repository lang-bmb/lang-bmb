#!/usr/bin/env python3
"""Measures the two arms of the port-shape experiment (results.md).

  measure.py [--bmb PATH] [--seeds N] [--lines N]

Per arm (tree.bmb, arena.bmb): lines and the lines only the proof reads
(scripts/components.py `spec_lines`, ADR-0122), definitions, proved
definitions, obligations, rlimit (sum, the largest obligation), stability
over solver seeds, the output against ref.py on generated input, and the
run time of the built program. Standard library only.
"""
import argparse
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import components  # noqa: E402


def rows(out):
    return [json.loads(l) for l in out.splitlines() if l.startswith("{")]


def arm(bmb, name, seeds, text, want, tmp):
    src = HERE / f"{name}.bmb"
    code = src.read_text(encoding="utf-8")
    r = {"arm": name, "lines": len(code.splitlines()), "spec": components.spec_lines(code)}
    assert subprocess.run([bmb, "fmt", "--check", str(src)], capture_output=True).returncode == 0, f"{name}: not in normal form"
    defs = [x for x in rows(subprocess.run([bmb, "verify", str(src)], capture_output=True, text=True).stdout) if "def" in x]
    r["defs"] = len(defs)
    r["proved"] = sum(x["status"] == "verified" and "trust" not in x for x in defs)
    r["obligations"] = sum(x["obligations"] for x in defs)
    r["rlimit"] = sum(x["rlimit"] for x in defs)
    r["rlimit_max"] = max(x["rlimit_max"] for x in defs)
    r["escapes"] = code.count("#[external_body]") + code.count("assume(")
    st = rows(subprocess.run([bmb, "stability", "--seeds", str(seeds), str(src)], capture_output=True, text=True).stdout)
    summary = next(x for x in st if "unstable" in x)
    r["unstable"], r["spread"] = summary["unstable"], summary.get("rlimit_spread_max")
    exe = Path(tmp) / name
    b = subprocess.run([bmb, "build", str(src), "-o", str(exe)], capture_output=True, text=True)
    assert b.returncode == 0, b.stdout
    best = None
    for _ in range(5):
        t0 = time.perf_counter()
        got = subprocess.run([str(exe)], input=text, capture_output=True).stdout
        dt = time.perf_counter() - t0
        best = dt if best is None else min(best, dt)
    r["same"] = got == want
    r["seconds"] = round(best, 3)
    return r


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bmb", default=str(ROOT / "target" / "debug" / "bmb"))
    ap.add_argument("--seeds", type=int, default=8)
    ap.add_argument("--lines", type=int, default=200000)
    a = ap.parse_args()
    text = subprocess.run([sys.executable, str(HERE / "ref.py"), "gen", "9", str(a.lines)], capture_output=True).stdout
    want = subprocess.run([sys.executable, str(HERE / "ref.py")], input=text, capture_output=True).stdout
    with tempfile.TemporaryDirectory() as tmp:
        for name in ("tree", "arena"):
            print(json.dumps(arm(a.bmb, name, a.seeds, text, want, tmp)))


if __name__ == "__main__":
    main()
