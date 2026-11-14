#!/usr/bin/env python3
"""The ablation sets of backlog T0-37: `bvb-write` and `brb-write`, each with
one thing changed and nothing else.

  python3 bench/ablate/gen.py            writes the six sets
  python3 bench/ablate/gen.py --check    fails if a committed set differs from a fresh build

  <source>-undecided   the server runs with `--undecided-info` (ADR-0108, H-UNDECIDE:
                       an undecided obligation carries the verdict of a bounded search)
  <source>-probe       the server runs with `--probe-tool` (ADR-0109, H-PROBE: a
                       read-only tool that tries candidate invariants)
  <source>-stateless   `"history": "stateless"` (ADR-0110, H-HISTORY: each repair turn
                       is sent the fixed context, the current state and one line per
                       earlier attempt; BRB's Rust arm takes `--history stateless`)

Each ADR registered its comparison before any run as "this one flag against
the original set, at the same commit". So the prompt, the seed, the tasks and
the hidden judge are the source set's byte for byte; only `set.json` differs,
in the one key the variant is about. The prompt does not mention the probe
tool: the server's tool list and description are what the model is shown, as
for every tool behind a flag (ADR-0109 decision 4).

Frozen like the write sets (ADR-0021 §1): regenerate with this script, never
edit by hand. The gate checks that a fresh build matches, that only the one
key differs, that the server of each set shows the tools it should, and that
the scripted reference and weak agents pass and fail (`test_gen.py`).
"""
import importlib.util
import json
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
sys.path.insert(0, os.path.join(BENCH, "m1"))
import harness  # noqa: E402
import resultlog  # noqa: E402

# The write sets' generator, for the BRB variant's MANIFEST and `tree`.
_spec = importlib.util.spec_from_file_location("edit_gen", os.path.join(BENCH, "edit", "gen.py"))
edit = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(edit)

SOURCES = ("bvb-write", "brb-write")
# name -> (what it changes in `set.json`, the ADR, the hypothesis, one line)
VARIANTS = {
    "undecided": ({"mcp_flags": "--undecided-info"}, "ADR-0108", "H-UNDECIDE",
                  "서버 플래그 `--undecided-info`(미결정 의무에 범위 탐색의 판정)"),
    "probe": ({"mcp_flags": "--probe-tool"}, "ADR-0109", "H-PROBE",
              "서버 플래그 `--probe-tool`(후보 불변식을 커밋 없이 시험하는 읽기 도구)"),
    "stateless": ({"history": "stateless"}, "ADR-0110", "H-HISTORY",
                  "`\"history\": \"stateless\"`(수리 차례마다 고정 문맥·현재 상태·시도마다 한 줄)"),
}


def config(source, variant):
    with open(os.path.join(BENCH, source, "set.json")) as f:
        cfg = json.load(f)
    for key, value in VARIANTS[variant][0].items():
        if key == "mcp_flags":
            cfg[key] = cfg[key] + [value]
        else:
            cfg[key] = value
    return cfg


def build_set(root, source, variant):
    src = os.path.join(BENCH, source)
    name = f"{source}-{variant}"
    out = os.path.join(root, name)
    resultlog.clear_set(out)
    os.makedirs(out, exist_ok=True)
    brb = source.startswith("brb")
    if brb:
        shutil.copy(os.path.join(src, "prompt-bmb.md"), os.path.join(out, "prompt-bmb.md"))
    else:
        shutil.copytree(os.path.join(src, "seed"), os.path.join(out, "seed"))
        shutil.copytree(os.path.join(src, "tasks"), os.path.join(out, "tasks"))
        shutil.copy(os.path.join(src, "prompt.md"), os.path.join(out, "prompt.md"))
    with open(os.path.join(out, "set.json"), "w") as f:
        json.dump(config(source, variant), f, indent=1)
        f.write("\n")
    if brb:
        text = edit.brb_manifest(out)
    else:
        harness.SET = out
        text = harness.manifest_text()
    with open(os.path.join(out, "MANIFEST"), "w") as f:
        f.write(text)
    _, adr, hyp, what = VARIANTS[variant]
    with open(os.path.join(out, "README.md"), "w") as f:
        f.write(f"# {name} ({hyp}, 백로그 T0-37)\n\n`bench/ablate/gen.py`가 `bench/{source}`에서 만든 판이다(동결). "
                f"직접 고치지 않고 생성기로 다시 만든다. {source}와 `set.json`의 한 가지만 다르다: {what}. "
                f"프롬프트·씨앗·과제·판정은 그대로다. 비교와 반증 조건은 {adr}에 실행 전에 등록돼 있다"
                + ("(Rust 팔은 `harness.py --history stateless`)" if brb and variant == "stateless" else "")
                + ".\n")
    return out


def build(root):
    return [build_set(root, s, v) for s in SOURCES for v in VARIANTS]


def main():
    if "--check" in sys.argv:
        tmp = tempfile.mkdtemp(prefix="ablate-")
        try:
            for out in build(tmp):
                name = os.path.basename(out)
                if edit.tree(out) != edit.tree(os.path.join(BENCH, name)):
                    sys.exit(f"bench/{name} differs from bench/ablate/gen.py output: regenerate it")
                print(f"bench/{name} ok")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    else:
        for out in build(BENCH):
            print(out)


if __name__ == "__main__":
    main()
