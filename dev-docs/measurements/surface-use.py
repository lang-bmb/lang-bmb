"""Use of each language feature across the repository's BMB (backlog 4D-11).

Usage: python3 -I dev-docs/measurements/surface-use.py [--md]

A feature is a set of node kinds or keys of the JSON syntax tree (`bmb ast`),
so the count is over parsed programs, not text. A file counts once per
feature. Files that do not parse (the error corpus has some on purpose) are
counted as skipped. The last column is the non-test size of the stage0 files
that exist only for the feature; code spread over the checker, the prover
and the code generator is not attributed.
"""
import glob
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BMB = os.path.join(ROOT, "target", "debug", "bmb")

# name -> (kinds, keys, dedicated stage0 files)
FEATURES = {
    "클로저": ({"closure"}, set(), ["closures.rs"]),
    "함수 값 경계 `F: Fn(..)`": (set(), {"fn_bound"}, []),
    "사용자 제네릭": (set(), {"generics"}, []),
    "trait·`impl Tr for`": (set(), {"traits", "trait_impls"}, []),
    "상수 항목 `const`": (set(), {"consts"}, ["consts.rs"]),
    "`f64`": ({"f64", "float"}, set(), []),
    "좁은 정수(`i8`~`i32`, `u16`, `u32`, `isize`)": ({"i8", "i16", "i32", "u16", "u32", "isize"}, set(), []),
    "`char`": ({"char"}, set(), []),
    "문자열·바이트 리터럴": ({"str", "bytes", "byte"}, set(), []),
    "`loop`": ({"loop"}, set(), []),
    "`break`·`continue`": ({"break", "continue"}, set(), []),
    "루프 레이블": (set(), {"label"}, []),
    "`for`": ({"for"}, set(), []),
    "`?`": ({"try"}, set(), []),
    "`return`": ({"return"}, set(), []),
    "배열 반복 `[x; n]`": ({"repeat"}, set(), []),
    "`proof fn`·`proof` 블록": ({"proof fn", "proof"}, set(), []),
    "`exists`": ({"exists"}, set(), []),
    "`assume`": ({"assume"}, set(), []),
    "`#[external_body]`": (set(), {"external_body"}, []),
    "`enum`·`match`": ({"enum", "match"}, set(), []),
    "`struct`": ({"struct"}, set(), []),
}

GROUPS = [
    ("실행 코퍼스", ["tests/run/*.bmb", "tests/lib_run/*.bmb"]),
    ("증명 코퍼스", ["tests/verify/**/*.bmb"]),
    ("검사 코퍼스", ["tests/check/**/*.bmb"]),
    ("도구 코퍼스", ["tests/tools/**/*.bmb", "tests/weaken/**/*.bmb", "tests/duplicates/**/*.bmb"]),
    ("`lib/std`", ["lib/**/*.bmb"]),
    ("벤치 원본", ["bench/baseline/*.bmb", "bench/game/*.bmb", "bench/twin/**/*.bmb", "bench/lib/**/*.bmb",
                "bench/m1b/**/*.bmb", "bench/bvb/**/*.bmb", "bench/brb-encap/**/*.bmb", "bench/impossible/**/*.bmb"]),
]


def seen(x, kinds, keys):
    if isinstance(x, dict):
        for k, v in x.items():
            keys.add(k)
            if k == "kind" and isinstance(v, str):
                kinds.add(v)
            seen(v, kinds, keys)
    elif isinstance(x, list):
        for v in x:
            seen(v, kinds, keys)


def file_lines(name):
    text = open(os.path.join(ROOT, "compiler", "src", name), encoding="utf-8").read().split("#[cfg(test)]")[0]
    return len(text.splitlines())


def main():
    counts = {f: [0] * len(GROUPS) for f in FEATURES}
    sizes, skipped = [], []
    for gi, (_, pats) in enumerate(GROUPS):
        files = sorted({p for pat in pats for p in glob.glob(os.path.join(ROOT, pat), recursive=True)})
        n = 0
        for p in files:
            r = subprocess.run([BMB, "ast", p], capture_output=True, text=True)
            if r.returncode != 0 or not r.stdout.startswith("{"):
                skipped.append(os.path.relpath(p, ROOT))
                continue
            n += 1
            kinds, keys = set(), set()
            seen(json.loads(r.stdout), kinds, keys)
            for f, (fk, fy, _) in FEATURES.items():
                if kinds & fk or keys & fy:
                    counts[f][gi] += 1
        sizes.append(n)
    head = ["기능"] + [f"{g}({n})" for (g, _), n in zip(GROUPS, sizes)] + ["합", "전용 stage0 파일"]
    print("| " + " | ".join(head) + " |")
    print("|" + " --- |" * len(head))
    for f, row in sorted(counts.items(), key=lambda kv: sum(kv[1])):
        own = FEATURES[f][2]
        files = ", ".join(f"`{o}` {file_lines(o):,}" for o in own) or "—"
        print(f"| {f} | " + " | ".join(str(c) for c in row) + f" | {sum(row)} | {files} |")
    print(f"\n파싱되지 않아 뺀 파일: {len(skipped)}개.")


if __name__ == "__main__":
    sys.exit(main())
