#!/usr/bin/env python3
"""Compact agent view task sets (backlog 2D-23, 2D-25, ADR-0068): prepared, not frozen.

  python3 bench/view/gen.py OUT     writes OUT/m1b-write-compact and OUT/bvb-write-compact

Each is the frozen `write` set (`bench/m1b-write`, `bench/bvb-write`) with one
change: the server also runs with `--compact`, so the model sees map and query
results as Rust-shaped text without hashes or positions, and proof results
with verified definitions as a count (compiler `view.rs`). Seed, tasks, BSB
mutants and prompt are the source set's, byte for byte; the prompt names no
output format. The tool descriptions of `query`, `map`, `verify` and `write`
say what the compact view returns.

Freezing is a separate decision (ADR-0068 §4): it is `gen.py bench` and a
commit of the two directories with their MANIFESTs, before the first model
run on them (ADR-0021 §1). Until then the sets live only in a scratch
directory, and `test_gen.py` checks that the scripted reference and weak
agents still pass and fail through the compact view.
"""
import json
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
sys.path.insert(0, os.path.join(BENCH, "m1"))
import bsb  # noqa: E402
import harness  # noqa: E402

SOURCES = ("m1b-write", "bvb-write")


def build_set(root, source):
    src = os.path.join(BENCH, source)
    out = os.path.join(root, f"{source}-compact")
    shutil.rmtree(out, ignore_errors=True)
    shutil.copytree(os.path.join(src, "seed", "src"), os.path.join(out, "seed", "src"))
    shutil.copytree(os.path.join(src, "tasks"), os.path.join(out, "tasks"))
    if os.path.isdir(bsb.mutants_dir(src)):
        shutil.copytree(bsb.mutants_dir(src), bsb.mutants_dir(out))
    shutil.copyfile(os.path.join(src, "prompt.md"), os.path.join(out, "prompt.md"))
    with open(os.path.join(src, "set.json")) as f:
        cfg = json.load(f)
    cfg["mcp_flags"] = cfg["mcp_flags"] + ["--compact"]
    with open(os.path.join(out, "set.json"), "w") as f:
        json.dump(cfg, f, indent=1)
        f.write("\n")
    harness.SET = out
    with open(os.path.join(out, "MANIFEST"), "w") as f:
        f.write(harness.manifest_text())
    with open(os.path.join(out, "README.md"), "w") as f:
        f.write(f"# {source}-compact (압축 에이전트 투영, 백로그 2D-23·2D-25, ADR-0068)\n\n"
                f"`bench/view/gen.py`가 `bench/{source}`에서 만든 판이다. {source}와 MCP 서버 플래그 `--compact` 하나만 "
                "다르다(생성기 머리 주석).\n")
    return out


def build(root):
    return [build_set(root, s) for s in SOURCES]


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__.split("\n\n")[1])
    for out in build(os.path.abspath(sys.argv[1])):
        print(out)


if __name__ == "__main__":
    main()
