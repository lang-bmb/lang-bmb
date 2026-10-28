#!/usr/bin/env python3
"""H-VIEW, the deterministic part (backlog T1-05): how big is the same code
in each view an agent could be shown?

  python3 dev-docs/experiments/view-size/run.py [--bmb PATH] [--model M] > results.md

Views of one workspace:
  stored    the source files as committed (comments included)
  normal    `bmb fmt` of every file (the normal form, ADR-0019; drops comments)
  minified  the normal form with every whitespace run collapsed to one space
  L1        `bmb map L1` (signatures, contracts, effects, status: no bodies)
  L0        `bmb map L0` (units and definition names)

Bytes are exact. Tokens are Claude's: the input tokens a CLI session reports
for the text, minus those of an empty prompt (`claude -p` reads the text on
stdin; no tools). The minified form is checked to be the same program: every
definition hash equals the normal form's.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
SETS = [("m1 seed", "bench/m1/seed/src"), ("m1b seed", "bench/m1b/seed/src"), ("bvb seed", "bench/bvb/seed/src"),
        ("lib", "lib")]


def files(d):
    return sorted(os.path.join(d, f) for f in os.listdir(d) if f.endswith(".bmb"))


def bmb(path, *args):
    r = subprocess.run([path, *args], capture_output=True, text=True)
    return r.returncode, r.stdout


def tokens(text, model, base=[None]):
    def measure(t):
        r = subprocess.run(["claude", "-p", "--model", model, "--system-prompt", "Reply with: ok", "--tools", "",
                            "--output-format", "json", "--no-session-persistence"],
                           input=t, capture_output=True, text=True)
        u = json.loads(r.stdout)["usage"]
        return u["input_tokens"] + u.get("cache_creation_input_tokens", 0) + u.get("cache_read_input_tokens", 0)
    if base[0] is None:
        base[0] = measure(".")
    return measure(text) - base[0] + 1


def hashes(path, text):
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "all.bmb")
        with open(p, "w") as f:
            f.write(text)
        _, out = bmb(path, "hash", p)
    return sorted((j["def"], j["body"]) for j in map(json.loads, out.splitlines()) if "def" in j)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    ap.add_argument("--model", default="claude-opus-5-5")
    args = ap.parse_args()
    print("# H-VIEW: 같은 코드의 투영별 크기\n")
    print(f"생성: `python3 dev-docs/experiments/view-size/run.py`. 토큰은 `{args.model}`의 입력 토큰(빈 입력 대비).\n")
    print("| 작업 공간 | 정의 | 투영 | 바이트 | 토큰 | 정규형 대비 토큰 | 같은 프로그램 |")
    print("| --- | ---: | --- | ---: | ---: | ---: | --- |")
    for label, rel in SETS:
        d = os.path.join(ROOT, rel)
        fs = files(d)
        stored = "".join(open(f).read() for f in fs)
        code, normal = bmb(args.bmb, "fmt", *fs)
        if code != 0:
            sys.exit(f"{label}: bmb fmt failed")
        minified = re.sub(r"\s+", " ", normal).strip() + "\n"
        _, l1 = bmb(args.bmb, "map", "L1", *fs)
        _, l0 = bmb(args.bmb, "map", "L0", *fs)
        defs = sum(1 for line in l1.splitlines() if '"level":"L1"' in line)
        same = hashes(args.bmb, normal) == hashes(args.bmb, minified)
        views = [("stored", stored, ""), ("normal", normal, ""), ("minified", minified, "예" if same else "아니오"),
                 ("L1", l1, ""), ("L0", l0, "")]
        n_tok = None
        for name, text, note in views:
            t = tokens(text, args.model)
            if name == "normal":
                n_tok = t
            ratio = f"{t / n_tok:.2f}" if n_tok else "—"
            print(f"| {label} | {defs} | {name} | {len(text.encode())} | {t} | {ratio} | {note} |", flush=True)


if __name__ == "__main__":
    main()
