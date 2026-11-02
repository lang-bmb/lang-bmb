#!/usr/bin/env python3
"""H-REPAIR's ablation set (ADR-0078, backlog T1-45): m1b-write with the
repair information taken out of every tool result the model sees.

  python3 bench/repair/gen.py            writes bench/m1b-write-norepair
  python3 bench/repair/gen.py --check    fails if the committed set differs from a fresh build

The set is the frozen `bench/m1b-write` byte for byte (prompt, seed, tasks,
BSB mutants) except `set.json`, which adds `"tool_filter": "no-repair"`: the
m1 harness (`bench/m1/harness.py`, `strip_repair`) then removes from each
tool result the counterexample that ends a diagnostic's message, every `fix`
(suggested change and hints) and every `counterexample` field. Only what the
model sees in tool results differs; the prompt is the same text (it still
says failures come with counterexamples: changing it would be a second
difference). The judge, tasks and limits are m1b-write's.

The measurement and its verdict: `compare.py`.

Frozen like the other generated sets (ADR-0021 §1): regenerate with this
script, never edit by hand. The gate checks that a fresh build matches and
that the scripted agents still pass and fail (`test_gen.py`).
"""
import json
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
sys.path.insert(0, os.path.join(BENCH, "m1"))
import bsb  # noqa: E402
import harness  # noqa: E402

SOURCE = "m1b-write"
NAME = "m1b-write-norepair"
FILTER = "no-repair"

README = f"""# {NAME} (H-REPAIR 절제 판, 백로그 T1-45)

`bench/repair/gen.py`가 `bench/{SOURCE}`에서 만든 판이다(동결). 직접 고치지 않고 생성기로 다시 만든다. {SOURCE}와 다른 것은 `set.json`의 `"tool_filter": "{FILTER}"` 하나다: m1 하네스가 모델에게 가는 모든 도구 결과에서 진단 메시지 끝의 반례(`; counterexample: ..`, `; unconfirmed counterexample: ..`), 모든 `fix`(수정 제안과 힌트), 모든 `counterexample` 필드를 지운다. 프롬프트·씨앗·과제·변이체·판정·한도는 {SOURCE}와 바이트 단위로 같다. 판정 규칙은 `bench/repair/README.md`.
"""


def build(root):
    src = os.path.join(BENCH, SOURCE)
    out = os.path.join(root, NAME)
    shutil.rmtree(out, ignore_errors=True)
    shutil.copytree(os.path.join(src, "seed"), os.path.join(out, "seed"))
    shutil.copytree(os.path.join(src, "tasks"), os.path.join(out, "tasks"))
    if os.path.isdir(bsb.mutants_dir(src)):
        shutil.copytree(bsb.mutants_dir(src), bsb.mutants_dir(out))
    shutil.copy(os.path.join(src, "prompt.md"), os.path.join(out, "prompt.md"))
    with open(os.path.join(src, "set.json")) as f:
        cfg = json.load(f)
    cfg["tool_filter"] = FILTER
    with open(os.path.join(out, "set.json"), "w") as f:
        json.dump(cfg, f, indent=1)
        f.write("\n")
    saved = harness.SET
    harness.SET = out
    try:
        with open(os.path.join(out, "MANIFEST"), "w") as f:
            f.write(harness.manifest_text())
    finally:
        harness.SET = saved
    with open(os.path.join(out, "README.md"), "w") as f:
        f.write(README)
    return out


def tree(d):
    """{relative path: bytes} of a set, without its measurement results."""
    out = {}
    for base, dirs, files in os.walk(d):
        if base == d:
            dirs[:] = [x for x in dirs if x != "results"]
        for f in files:
            if "__pycache__" in base:
                continue
            p = os.path.join(base, f)
            with open(p, "rb") as fh:
                out[os.path.relpath(p, d)] = fh.read()
    return out


def main():
    if "--check" in sys.argv:
        tmp = tempfile.mkdtemp(prefix="repair-")
        try:
            out = build(tmp)
            if tree(out) != tree(os.path.join(BENCH, NAME)):
                sys.exit(f"bench/{NAME} differs from bench/repair/gen.py output: regenerate it")
            print(f"bench/{NAME} ok")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    else:
        print(build(BENCH))


if __name__ == "__main__":
    main()
