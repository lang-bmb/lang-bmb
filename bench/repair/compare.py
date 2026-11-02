#!/usr/bin/env python3
"""H-REPAIR verdict (ADR-0078, backlog T1-45): do counterexamples and `fix`
cut the repair iterations?

  compare.py FULL.jsonl NOREPAIR.jsonl     the comparison table and the H-REPAIR line

FULL is a run of `bench/m1b-write`, NOREPAIR one of `bench/m1b-write-norepair`
(the same set with the repair information filtered out of the tool results,
`gen.py`), by the same agent at the same compiler commit and condition. The
measure, registered before the first run (README.md):

  repair cost = sum over successful `write` sessions of (commits_rejected +
                proofs_failed) / number of successful `write` sessions

`commits_rejected` counts writes refused at commit (syntax, type), and
`proofs_failed` writes that committed but did not prove plus `verify` calls
that did not prove (m1 harness, `proof_failed`). Navigation tasks have no
writes and are left out. Confirmed when FULL's cost is at least 40% below
NOREPAIR's (FULL <= 0.6 x NOREPAIR); falsified otherwise. Undecided when a
side has no successful write session or NOREPAIR's cost is 0.

Standard library only.
"""
import json
import sys

REDUCTION = 0.40


def load(path):
    with open(path, encoding="utf-8") as f:
        recs = [json.loads(line) for line in f if line.strip()]
    head = next((r for r in recs if r.get("summary")), None)
    if head is None:
        sys.exit(f"{path}: no summary row (an unfinished run)")
    return head, [r for r in recs if not r.get("summary") and r.get("kind") == "write"]


def cost(rows):
    """(sessions, rejected commits, failed proofs) over the successful rows."""
    ok = [r for r in rows if r["success"]]
    if any("proofs_failed" not in r for r in ok):
        sys.exit("a row without proofs_failed: run with a harness that counts failed proofs (T1-45)")
    return len(ok), sum(r["commits_rejected"] for r in ok), sum(r["proofs_failed"] for r in ok)


def verdict(full, none):
    """The H-REPAIR line from two `cost` triples."""
    head = "H-REPAIR (성공 작성 세션당 거부 커밋 + 실패 증명, 확정: 전체 ≤ 절제 × 0.6)"
    (fn, fr, fp), (nn, nr, np_) = full, none
    if not fn or not nn:
        return f"{head}: 판정 불가(성공한 작성 세션이 없는 쪽)"
    a, b = (fr + fp) / fn, (nr + np_) / nn
    shares = f"전체 ({fr}+{fp})/{fn} = {a:.2f}, 절제 ({nr}+{np_})/{nn} = {b:.2f}"
    if b == 0:
        return f"{head}: {shares} → 판정 불가(절제 쪽 비용 0)"
    cut = 1 - a / b
    return f"{head}: {shares}, 감소 {cut:.1%} → {'확정' if cut >= REDUCTION else '반증'}"


def compare(full_path, none_path):
    (hf, rf), (hn, rn) = load(full_path), load(none_path)
    problems = []
    if (hf.get("task_set"), hn.get("task_set")) != ("m1b-write", "m1b-write-norepair"):
        problems.append(f"sets {hf.get('task_set')} / {hn.get('task_set')}, want m1b-write / m1b-write-norepair")
    for key in ("agent", "commit", "condition"):
        if hf.get(key) != hn.get(key):
            problems.append(f"{key} differs: {hf.get(key)} / {hn.get(key)}")
    if problems:
        sys.exit("not comparable: " + "; ".join(problems))
    cf, cn = cost(rf), cost(rn)
    out = ["| 지표 | m1b-write | m1b-write-norepair |", "| --- | --- | --- |"]
    out.append(f"| 작성 과제 성공 | {cf[0]}/{len(rf)} | {cn[0]}/{len(rn)} |")
    out.append(f"| 거부 커밋(성공 세션 합) | {cf[1]} | {cn[1]} |")
    out.append(f"| 실패 증명(성공 세션 합) | {cf[2]} | {cn[2]} |")
    out += ["", verdict(cf, cn)]
    return "\n".join(out) + "\n"


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    if len(argv) != 2:
        sys.exit(__doc__)
    sys.stdout.write(compare(*argv))


if __name__ == "__main__":
    main()
