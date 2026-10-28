#!/usr/bin/env python3
"""Rename ripple (direction discussion 2026-10-28, hypothesis H-ID).

For each definition whose last path segment is unique in its file, rename it
(every whole-word occurrence) and count the *other* definitions whose body or
interface hash changes. Today a reference is hashed by its full path
(ADR-0019 §2), so a rename changes every referrer; with hash references and
names as metadata (Unison) the count would be zero.

Renames that no longer check (the name was also a method, field or local) are
skipped and counted. A changed definition that is not an L2 referrer of the
renamed one is reported apart as `noise` (the word also occurred elsewhere).

  python3 dev-docs/experiments/rename-ripple/run.py [--bmb PATH] > results.md
"""
import argparse
import glob
import json
import os
import re
import subprocess
import tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))


def jsonl(out):
    return [json.loads(line) for line in out.splitlines() if line.strip()]


def probe(bmb, path, text):
    with open(path, "w") as f:
        f.write(text)
    ok = subprocess.run([bmb, "check", path], capture_output=True).returncode == 0
    hashes = {}
    if ok:
        out = subprocess.run([bmb, "hash", path], capture_output=True, text=True).stdout
        hashes = {j["def"]: (j["body"], j["iface"]) for j in jsonl(out)}
    return ok, hashes


def referrers(bmb, path):
    out = subprocess.run([bmb, "map", "L2", path], capture_output=True, text=True).stdout
    refs = {}
    for j in jsonl(out):
        if j.get("level") == "L2":
            refs.setdefault(j["to"], set()).add(j["from"])
    return refs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bmb", default=os.path.join(ROOT, "compiler", "target", "release", "bmb"))
    args = ap.parse_args()
    files = [os.path.join(ROOT, "lib", "bytes.bmb")]
    files += sorted(glob.glob(os.path.join(ROOT, "tests", "run", "*.bmb")))
    tot = dict(files=0, defs=0, renamed=0, skipped=0, rippling=0, body=0, iface=0, noise=0)
    rows = []
    with tempfile.TemporaryDirectory() as tmp:
        work = os.path.join(tmp, "w.bmb")
        for src in files:
            text = open(src).read()
            ok, base = probe(args.bmb, work, text)
            if not ok:
                continue
            refs = referrers(args.bmb, work)
            names = [d.split("::")[-1] for d in base]
            tot["files"] += 1
            tot["defs"] += len(base)
            f = dict(renamed=0, rippling=0, iface=0)
            for d in base:
                name = d.split("::")[-1]
                if names.count(name) != 1:
                    tot["skipped"] += 1
                    continue
                ok, h = probe(args.bmb, work, re.sub(r"\b%s\b" % re.escape(name), name + "_rn", text))
                if not ok or any(k not in h for k in base if k != d):
                    # the word also named a unit or another path segment
                    tot["skipped"] += 1
                    continue
                f["renamed"] += 1
                others = [k for k in base if k != d and h.get(k) != base[k]]
                clean = [k for k in others if k in refs.get(d, set())]
                tot["noise"] += len(others) - len(clean)
                tot["body"] += sum(1 for k in clean if h[k][0] != base[k][0])
                ifc = sum(1 for k in clean if h[k][1] != base[k][1])
                tot["iface"] += ifc
                f["iface"] += ifc
                if clean:
                    f["rippling"] += 1
            tot["renamed"] += f["renamed"]
            tot["rippling"] += f["rippling"]
            rows.append((os.path.relpath(src, ROOT), len(base), f["renamed"], f["rippling"], f["iface"]))
    print("# 이름 바꾸기 파급 (H-ID)\n")
    print("생성: `python3 dev-docs/experiments/rename-ripple/run.py`. 정의 하나의 이름만 바꿨을 때 해시가 바뀌는 *다른* 정의(그 정의를 부르는 L2 참조자) 수.\n")
    print("| 항목 | 값 |\n| --- | ---: |")
    print("| 파일 | %d |" % tot["files"])
    print("| 정의 | %d |" % tot["defs"])
    print("| 이름을 바꾼 정의(검사 통과) | %d |" % tot["renamed"])
    print("| 건너뜀(이름 중복·검사 실패) | %d |" % tot["skipped"])
    print("| 다른 정의를 바꾼 이름 바꾸기 | %d (%.0f%%) |" % (tot["rippling"], 100.0 * tot["rippling"] / max(tot["renamed"], 1)))
    print("| 본문 해시가 바뀐 참조자(합) | %d |" % tot["body"])
    print("| 계약 해시가 바뀐 참조자(합, 그 호출자의 증명 캐시까지 무효) | %d |" % tot["iface"])
    print("| 참조자가 아닌데 바뀐 정의(같은 낱말, 집계에서 뺌) | %d |" % tot["noise"])
    print("\n해시 참조(이름은 메타데이터)라면 위의 참조자 수는 모두 0이다.\n")
    print("| 파일 | 정의 | 바꾼 이름 | 파급 | 계약 해시 파급 |\n| --- | ---: | ---: | ---: | ---: |")
    for r in rows:
        print("| `%s` | %d | %d | %d | %d |" % r)


if __name__ == "__main__":
    main()
