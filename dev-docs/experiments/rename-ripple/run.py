#!/usr/bin/env python3
"""Rename ripple (direction discussion 2026-10-28, hypothesis H-ID, ADR-0059).

For each definition whose last path segment is unique in its file, rename it
(every whole-word occurrence) and count the *other* definitions whose `id`,
body or interface hash changes. Before ADR-0059 a reference was hashed by its
full path (ADR-0019 §2), so a rename changed every referrer; with hash
references and names as metadata (Unison) the count is zero.

Renames that no longer check (the name was also a method, field or local) are
skipped and counted. A changed definition that is not an L2 referrer of the
renamed one is reported apart as `noise` (the word also occurred elsewhere).

With `--stats`, each rename is also verified through a copy of a warm cache
(`bmb verify --cache --stats`): the definitions the incremental build encoded
again and the proof cache misses (ADR-0059 H-ID closing measurement).

  python3 dev-docs/experiments/rename-ripple/run.py [--bmb PATH] [--stats] > results.md
"""
import argparse
import glob
import json
import os
import re
import shutil
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor

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
        hashes = {j["def"]: (j["body"], j["iface"], j.get("id")) for j in jsonl(out)}
    return ok, hashes


def referrers(bmb, path):
    out = subprocess.run([bmb, "map", "L2", path], capture_output=True, text=True).stdout
    refs = {}
    for j in jsonl(out):
        if j.get("level") == "L2":
            refs.setdefault(j["to"], set()).add(j["from"])
    return refs


def stats_of(bmb, warm, text, tmp, tag):
    """Encoded definitions and proof cache misses of verifying `text` through
    a copy of the warm cache `warm`."""
    d = os.path.join(tmp, f"s{tag}")
    os.makedirs(d)
    src, cache, st = os.path.join(d, "w.bmb"), os.path.join(d, "cache"), os.path.join(d, "stats.json")
    shutil.copytree(warm, cache)
    with open(src, "w") as f:
        f.write(text)
    subprocess.run([bmb, "verify", "--cache", cache, "--stats", st, src], capture_output=True)
    s = json.load(open(st))
    shutil.rmtree(d)
    return len(s["encoded"]), s["proof_cache_misses"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bmb", default=os.path.join(ROOT, "target", "release", "bmb"))
    ap.add_argument("--stats", action="store_true")
    ap.add_argument("--jobs", type=int, default=4)
    args = ap.parse_args()
    files = [os.path.join(ROOT, "lib", "bytes.bmb")]
    files += sorted(glob.glob(os.path.join(ROOT, "tests", "run", "*.bmb")))
    tot = dict(files=0, defs=0, renamed=0, skipped=0, rippling=0, body=0, iface=0, id=0, noise=0,
               encoded=0, encoding_renames=0, misses=0, missing_renames=0)
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
            f = dict(renamed=0, rippling=0, iface=0, encoded=0, misses=0)
            renamed_texts = []
            for d in base:
                name = d.split("::")[-1]
                if names.count(name) != 1:
                    tot["skipped"] += 1
                    continue
                new = re.sub(r"\b%s\b" % re.escape(name), name + "_rn", text)
                ok, h = probe(args.bmb, work, new)
                if not ok or any(k not in h for k in base if k != d):
                    # the word also named a unit or another path segment
                    tot["skipped"] += 1
                    continue
                f["renamed"] += 1
                renamed_texts.append(new)
                others = [k for k in base if k != d and h.get(k) != base[k]]
                clean = [k for k in others if k in refs.get(d, set())]
                tot["noise"] += len(others) - len(clean)
                tot["body"] += sum(1 for k in clean if h[k][0] != base[k][0])
                tot["id"] += sum(1 for k in clean if h[k][2] != base[k][2])
                ifc = sum(1 for k in clean if h[k][1] != base[k][1])
                tot["iface"] += ifc
                f["iface"] += ifc
                if clean:
                    f["rippling"] += 1
            if args.stats and renamed_texts:
                warm = os.path.join(tmp, "warm")
                with open(work, "w") as fh:
                    fh.write(text)
                subprocess.run([args.bmb, "verify", "--cache", warm, work], capture_output=True)
                with ThreadPoolExecutor(args.jobs) as ex:
                    res = list(ex.map(lambda kt: stats_of(args.bmb, warm, kt[1], tmp, kt[0]),
                                      enumerate(renamed_texts)))
                shutil.rmtree(warm)
                for enc, miss in res:
                    f["encoded"] += enc
                    f["misses"] += miss
                    tot["encoded"] += enc
                    tot["misses"] += miss
                    tot["encoding_renames"] += enc > 0
                    tot["missing_renames"] += miss > 0
            tot["renamed"] += f["renamed"]
            tot["rippling"] += f["rippling"]
            rows.append((os.path.relpath(src, ROOT), len(base), f["renamed"], f["rippling"], f["iface"],
                         f["encoded"], f["misses"]))
    print("# 이름 바꾸기 파급 (H-ID)\n")
    print("생성: `python3 dev-docs/experiments/rename-ripple/run.py%s`. 정의 하나의 이름만 바꿨을 때 정체성·해시가 바뀌는 *다른* 정의(그 정의를 부르는 L2 참조자) 수."
          % (" --stats" if args.stats else ""))
    print("`bmb hash`의 `id`·`body`·`iface`(ADR-0059 §1·§4).\n")
    print("| 항목 | 값 |\n| --- | ---: |")
    print("| 파일 | %d |" % tot["files"])
    print("| 정의 | %d |" % tot["defs"])
    print("| 이름을 바꾼 정의(검사 통과) | %d |" % tot["renamed"])
    print("| 건너뜀(이름 중복·검사 실패) | %d |" % tot["skipped"])
    print("| 다른 정의를 바꾼 이름 바꾸기 | %d (%.0f%%) |" % (tot["rippling"], 100.0 * tot["rippling"] / max(tot["renamed"], 1)))
    print("| 본문 해시가 바뀐 참조자(합) | %d |" % tot["body"])
    print("| 계약 해시가 바뀐 참조자(합) | %d |" % tot["iface"])
    print("| `id`가 바뀐 참조자(합) | %d |" % tot["id"])
    print("| 참조자가 아닌데 바뀐 정의(같은 낱말, 집계에서 뺌) | %d |" % tot["noise"])
    if args.stats:
        print("| 증분 빌드가 다시 인코딩한 정의(합, 자기 포함) | %d |" % tot["encoded"])
        print("| 다시 인코딩이 있었던 이름 바꾸기 | %d |" % tot["encoding_renames"])
        print("| 증명 캐시 놓침(합) | %d |" % tot["misses"])
        print("| 증명 캐시 놓침이 있었던 이름 바꾸기 | %d |" % tot["missing_renames"])
    print()
    if args.stats:
        print("| 파일 | 정의 | 바꾼 이름 | 파급 | 계약 해시 파급 | 재인코딩 | 캐시 놓침 |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
        for r in rows:
            print("| `%s` | %d | %d | %d | %d | %d | %d |" % r)
    else:
        print("| 파일 | 정의 | 바꾼 이름 | 파급 | 계약 해시 파급 |\n| --- | ---: | ---: | ---: | ---: |")
        for r in rows:
            print("| `%s` | %d | %d | %d | %d |" % r[:5])


if __name__ == "__main__":
    main()
