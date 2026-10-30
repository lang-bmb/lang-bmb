#!/usr/bin/env python3
"""H-COMMUTE: concurrent edits merged by text and by definition (backlog T1-12).

  python3 dev-docs/experiments/commute/run.py --work DIR [--bmb PATH] [--profile P]
          [--span J] [--every S] [--defs N] [--variants none,own,reader,world]
          [--rows R.jsonl] > results.md

Since T1-27 each pair is also judged by judge.py (ADR-0065 and its variants:
read sets from `bmb reads`, the refinement exemption from `bmb weaken
--readers`), and the definition-level merge is proved whenever the two sides
wrote different definitions by their own text (the transitive `bmb hash`
sets below skip pairs whose edits touch different definitions). The second
report section compares the verdicts on that ground truth. Since T1-29 a
conflict without overlapping write sets is finished on the merge
(`judge.resolve`, ADR-0065 §4: only the conflict readers re-proved), and that
merge is also proved whole without caches for the cost; the third section.

Two agents start from the same tree: the BCB history (bench/bcb/history.py) at
commit c. Agent A makes the next J commits (c+1..c+J), agent B the J after
them (c+J+1..c+2J), each on that base: B's edits are re-applied to the base
plan, and a pair where B alone does not apply or does not prove is skipped (B
depended on A). J (`--span`) is how long the two worked apart. Each pair is merged two
ways and judged:

  text        `git merge-file` per unit file (BMB and the Rust twin); a clean
              BMB merge is proved, and a failure is a false merge.
  definition  both edit lists applied to the base plan; a write-write overlap
              (a definition both changed) is a conflict, otherwise the merged
              workspace is proved.

The commutation verdict (dev-docs/reviews/collaboration-2026-10-28.md §4) is
computed from the two sides alone, without proving the merge: A and B conflict
when their write sets overlap, or when one side changed the interface (iface
hash, or removed) of a definition that a definition the other side wrote reads
(an L2 edge out of it in that side's tree). Contract changes are not exempted
yet as refinements (2D-19), so the verdict is conservative. Soundness: no pair
judged independent may fail to prove when merged.
"""
import argparse
import contextlib
import copy
import json
import os
import random
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "bench", "bcb"))
import history  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import judge  # noqa: E402


def jsonl(text):
    return [json.loads(x) for x in text.splitlines() if x.strip().startswith("{")]


class Bmb:
    def __init__(self, path, cache):
        self.path, self.cache = path, cache

    def run(self, *args):
        r = subprocess.run([self.path, *args], capture_output=True, text=True)
        return r.returncode, r.stdout

    def proves(self, src):
        return self.run("verify", "--cache", self.cache, src)[0] == 0

    def hashes(self, src):
        _, out = self.run("hash", src)
        return {j["def"]: (j["body"], j["iface"]) for j in jsonl(out) if "def" in j}

    def reads(self, src):
        _, out = self.run("map", "--cache", self.cache, "L2", src)
        edges = {}
        for j in jsonl(out):
            if j.get("level") == "L2":
                edges.setdefault(j["from"], set()).add(j["to"])
        return edges


def write(tree, d):
    shutil.rmtree(d, ignore_errors=True)
    history.write_tree(tree, os.path.join(d, "bmb"), os.path.join(d, "rust"))
    return os.path.join(d, "bmb", "src")


def apply_all(tree, edits):
    t = copy.deepcopy(tree)
    for e in edits:
        t.apply(copy.deepcopy(e))
    return t


def text_merge(base, a, b, lang, work):
    """Merged unit texts and the number of conflicting units, by `git merge-file`."""
    merged, conflicts = {}, 0
    for u in range(base.n_units):
        o, x, y = base.text[(lang, u)], a.text[(lang, u)], b.text[(lang, u)]
        if x == y or y == o:
            merged[u] = x
            continue
        if x == o:
            merged[u] = y
            continue
        paths = []
        for name, text in (("a", x), ("o", o), ("b", y)):
            p = os.path.join(work, f"{lang}-{u}-{name}")
            with open(p, "w") as f:
                f.write(text)
            paths.append(p)
        r = subprocess.run(["git", "merge-file", "-p", *paths], capture_output=True, text=True)
        if r.returncode != 0:
            conflicts += 1
        merged[u] = r.stdout
    return merged, conflicts


def changed(h0, h1):
    """(written defs, defs whose interface changed or that were removed)."""
    written = {d for d in set(h0) | set(h1) if h0.get(d) != h1.get(d)}
    iface = {d for d in h0 if d not in h1 or h0[d][1] != h1[d][1]}
    return written, iface


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    ap.add_argument("--profile", default="contract-heavy")
    ap.add_argument("--defs", type=int, default=1000)
    ap.add_argument("--commits", type=int, default=1000)
    ap.add_argument("--span", type=int, default=1, help="commits each agent makes apart")
    ap.add_argument("--every", type=int, default=10, help="sample a base commit every S commits")
    ap.add_argument("--rows", help="also write the per-pair rows (JSONL) here")
    ap.add_argument("--variants", default=",".join(judge.VARIANTS),
                    help="judge.py variants (`own` runs `bmb weaken` on the whole program per side)")
    args = ap.parse_args()
    os.makedirs(args.work, exist_ok=True)
    hist = os.path.join(args.work, f"{args.profile}-{args.defs}.jsonl")
    if not os.path.exists(hist):
        with contextlib.redirect_stdout(sys.stderr):
            history.main(["gen", hist, "--defs", str(args.defs), "--commits", str(args.commits),
                          "--profile", args.profile])
    header, commits = history.load(hist)
    bmb = Bmb(args.bmb, os.path.join(args.work, "cache"))
    jb = judge.Bmb(args.bmb, os.path.join(args.work, "cache"))
    bases = {}
    for k, tree, _ in history.replay(header, commits):
        if k % args.every == 0 and k + 2 * args.span <= len(commits):
            bases[k] = copy.deepcopy(tree)
    rows = []
    tmp = tempfile.mkdtemp(prefix="commute-")
    for k, base in sorted(bases.items()):
        ea = [e for c in commits[k:k + args.span] for e in c["edits"]]
        eb = [e for c in commits[k + args.span:k + 2 * args.span] for e in c["edits"]]
        row = {"k": k, "a": [e["op"] for e in ea], "b": [e["op"] for e in eb]}
        try:
            a, b = apply_all(base, ea), apply_all(base, eb)
        except (KeyError, ValueError, IndexError) as err:
            rows.append({**row, "skip": f"B does not apply alone: {type(err).__name__}"})
            continue
        srcs = [write(t, os.path.join(tmp, n)) for t, n in ((base, "o"), (a, "a"), (b, "b"))]
        if not bmb.proves(srcs[2]):
            rows.append({**row, "skip": "B alone does not prove"})
            continue
        h0, ha, hb = (bmb.hashes(s) for s in srcs)
        ra, rb = bmb.reads(os.path.join(tmp, "a", "bmb", "src")), bmb.reads(os.path.join(tmp, "b", "bmb", "src"))
        wa, ia = changed(h0, ha)
        wb, ib = changed(h0, hb)
        reads_a = set().union(*(ra.get(d, set()) for d in wa)) if wa else set()
        reads_b = set().union(*(rb.get(d, set()) for d in wb)) if wb else set()
        independent = not (wa & wb) and not (ib & reads_a) and not (ia & reads_b)
        # text merge (both languages)
        merged_bmb, conf_bmb = text_merge(base, a, b, "bmb", tmp)
        _, conf_rust = text_merge(base, a, b, "rust", tmp)
        text_proves = None
        if conf_bmb == 0:
            d = os.path.join(tmp, "t", "src")
            shutil.rmtree(os.path.join(tmp, "t"), ignore_errors=True)
            os.makedirs(d)
            for u, text in merged_bmb.items():
                with open(os.path.join(d, f"m{u}.bmb"), "w") as f:
                    f.write(text)
            text_proves = bmb.proves(d)
        # definition merge
        def_conflict = bool(wa & wb)
        def_proves = None
        src_m = None
        if not def_conflict:
            try:
                m = apply_all(a, eb)
                src_m = write(m, os.path.join(tmp, "m"))
                def_proves = bmb.proves(src_m)
            except (KeyError, ValueError, IndexError):
                def_proves = False
        # ADR-0065 (T1-27): judge.py from the three versions alone, and the
        # ground truth on the definitions' own write sets: the merge exists
        # unless both sides wrote one definition (the transitive sets above
        # also skip pairs whose edits touch different definitions).
        jb.take()
        verdict = judge.judge(jb, *(os.path.join(tmp, n, "bmb", "src") for n in "oab"),
                              variants=tuple(args.variants.split(",")))
        cost = jb.take()
        merge_proves = def_proves
        if verdict["ww"]:
            merge_proves, src_m = None, None
        elif def_conflict:
            try:
                m = apply_all(a, eb)
                src_m = write(m, os.path.join(tmp, "m"))
                merge_proves = bmb.proves(src_m)
            except (KeyError, ValueError, IndexError):
                merge_proves = False
        # ADR-0065 §4 (T1-29): finish the conflicts on the merge; the whole
        # merge proved without caches for the cost comparison.
        resolved = {v: judge.resolve(jb, verdict, src_m, v) for v in judge.RESOLVE if v in verdict}
        whole = None
        if any(r and r["merge"] for r in resolved.values()):
            whole = jb.verify(src_m, kind="verify_whole_cold")
        step_cost = jb.take()
        rows.append({**row, "wa": len(wa), "wb": len(wb), "independent": independent,
                     "text_conflict_bmb": conf_bmb, "text_conflict_rust": conf_rust, "text_proves": text_proves,
                     "def_conflict": def_conflict, "def_proves": def_proves,
                     "merge_proves": merge_proves, "judge": verdict, "cost": cost,
                     "resolve": resolved, "whole": whole, "step_cost": step_cost})
        print(json.dumps(rows[-1]), file=sys.stderr, flush=True)
    shutil.rmtree(tmp, ignore_errors=True)
    if args.rows:
        with open(args.rows, "w") as f:
            for r in rows:
                f.write(json.dumps(r, sort_keys=True) + "\n")
    report(args, rows)
    report_adr(args, rows)


def report(args, rows):
    used = [r for r in rows if "skip" not in r]
    n = len(used)

    def count(f):
        return sum(1 for r in used if f(r))

    print(f"## 프로필 `{args.profile}`, 정의 {args.defs}개, 각자 커밋 {args.span}개\n")
    print(f"생성: `python3 dev-docs/experiments/commute/run.py --profile {args.profile} --span {args.span} "
          f"--every {args.every}`. 기준 커밋 c에서 A = 커밋 c+1..c+{args.span}, "
          f"B = 커밋 c+{args.span + 1}..c+{2 * args.span}를 각자 적용한 쌍.\n")
    print("| 항목 | 값 |\n| --- | ---: |")
    print(f"| 표본 쌍 | {len(rows)} |")
    print(f"| 건너뜀(B가 A에 기댐) | {len(rows) - n} |")
    print(f"| 쓴 쌍 | {n} |")
    print(f"| 텍스트 충돌(BMB 단위 파일, `git merge-file`) | {count(lambda r: r['text_conflict_bmb'] > 0)} |")
    print(f"| 텍스트 충돌(Rust 쌍둥이) | {count(lambda r: r['text_conflict_rust'] > 0)} |")
    print(f"| 깨끗한 텍스트 병합 중 증명 실패(거짓 병합) | "
          f"{count(lambda r: r['text_proves'] is False)} / {count(lambda r: r['text_proves'] is not None)} |")
    print(f"| 정의 단위 쓰기 충돌 | {count(lambda r: r['def_conflict'])} |")
    print(f"| 정의 단위 병합 중 증명 실패 | "
          f"{count(lambda r: r['def_proves'] is False)} / {count(lambda r: r['def_proves'] is not None)} |")
    print(f"| 교환 판정: 독립 | {count(lambda r: r['independent'])} |")
    print(f"| **독립 판정인데 병합이 증명 실패(건전성 위반)** | "
          f"{count(lambda r: r['independent'] and r['def_proves'] is False)} |")
    print(f"| 충돌 판정인데 정의 단위 병합이 증명됨(보수적 판정) | "
          f"{count(lambda r: not r['independent'] and r['def_proves'] is True)} |")
    print(f"| 충돌 판정인데 텍스트로는 깨끗이 병합 | "
          f"{count(lambda r: not r['independent'] and r['text_conflict_bmb'] == 0)} |")
    print("\n| k | A 편집 | B 편집 | 쓰기 A·B | 독립 | 텍스트 충돌 BMB·Rust | 텍스트 병합 증명 | 정의 충돌 | 정의 병합 증명 |")
    print("| ---: | --- | --- | --- | --- | --- | --- | --- | --- |")
    for r in rows:
        if "skip" in r:
            print(f"| {r['k']} | {len(r['a'])} | {len(r['b'])} | — | — | — | — | — | {r['skip']} |")
        else:
            print(f"| {r['k']} | {len(r['a'])} | {len(r['b'])} | {r['wa']}·{r['wb']} | {r['independent']} | "
                  f"{r['text_conflict_bmb']}·{r['text_conflict_rust']} | {r['text_proves']} | {r['def_conflict']} | "
                  f"{r['def_proves']} |")


def report_adr(args, rows):
    """The ADR-0065 judgement and its variants (judge.py) on the same pairs."""
    used = [r for r in rows if "skip" not in r]
    verdicts = ("old",) + tuple(v for v in judge.VARIANTS if v in args.variants.split(","))

    def commute(r, v):
        return r["independent"] if v == "old" else r["judge"][v]["commute"]
    truth = [r for r in used if r["merge_proves"] is not None]
    print(f"\n### ADR-0065 판정(T1-27): 프로필 `{args.profile}`, 각자 커밋 {args.span}개\n")
    print(f"정답은 정의 단위 병합의 증명이다. 두 편집이 같은 정의를 썼으면(자기 텍스트 기준) 병합이 없으므로 충돌이 "
          f"정답이다. 쓴 쌍 {len(used)}개 중 같은 정의를 쓴 쌍 {len(used) - len(truth)}개, 병합이 증명된 쌍 "
          f"{sum(1 for r in truth if r['merge_proves'])}개, 실패 {sum(1 for r in truth if not r['merge_proves'])}개. "
          f"첫 재생은 전이적 쓰기 집합(`bmb hash`)이 겹친 쌍을 병합하지 않았다. 그중 자기 텍스트로는 겹치지 않는 "
          f"{sum(1 for r in used if r['def_conflict'] and not r['judge']['ww'])}쌍을 이번에 병합했다.\n")
    print("| 판정 | 교환 | 충돌 | 건전성 위반(교환인데 병합 실패) | 거짓 충돌(충돌인데 병합 증명) |")
    print("| --- | ---: | ---: | ---: | ---: |")
    for v in verdicts:
        c = [r for r in used if commute(r, v)]
        print(f"| `{v}` | {len(c)} | {len(used) - len(c)} | "
              f"{sum(1 for r in c if r['merge_proves'] is not True)} | "
              f"{sum(1 for r in used if not commute(r, v) and r['merge_proves'] is True)} |")
    asks = sum(r["judge"]["queries"]["a"][0] + r["judge"]["queries"]["b"][0] for r in used)
    obls = sum(r["judge"]["queries"]["a"][1] + r["judge"]["queries"]["b"][1] for r in used)
    cost = {}
    for r in used:
        for k, (n, s) in r["cost"].items():
            c = cost.setdefault(k, [0, 0.0])
            c[0] += n
            c[1] += s
    print(f"\n면제 질의(읽는 쪽 판): 행 {asks}개, 해법기 의무 {obls}개. "
          f"`id` 쓰기 집합(ADR-0065 §1 초안)이면 생기는 쓰기 충돌(자기 텍스트로는 없음): "
          f"{sum(1 for r in used if r['judge']['ww_id'] and not r['judge']['ww'])}쌍. CPU(호출 수 · 초): "
          + ", ".join(f"`{k}` {n} · {s:.1f}" for k, (n, s) in sorted(cost.items())) + ".\n")
    print("| k | 쓰기 겹침(자기·id) | 병합 증명 | " + " | ".join(verdicts) + " | 면제 질의 |")
    print("| ---: | --- | --- | " + " | ".join("---" for _ in verdicts) + " | ---: |")
    for r in used:
        j = r["judge"]
        print(f"| {r['k']} | {len(j['ww'])}·{j['ww_id']} | {r['merge_proves']} | "
              + " | ".join("교환" if commute(r, v) else "충돌" for v in verdicts)
              + f" | {j['queries']['a'][0] + j['queries']['b'][0]} |")
    if not used or not all("resolve" in r for r in used):
        return
    print(f"\n### 비교환 쌍의 해소(T1-29, ADR-0065 결정 4): 프로필 `{args.profile}`, 각자 커밋 {args.span}개\n")
    judge.report_resolve([{"verdict": r["judge"], "merge_proves": r["merge_proves"], "resolve": r["resolve"],
                           "whole": r["whole"], "cost": {**r["cost"], **r["step_cost"]}} for r in used])
    step = [r for r in used if any(x and x["merge"] for x in r["resolve"].values())]
    if step:
        print("\n| k | 병합 증명 | " + " | ".join(f"`{v}` 충돌 원소 · 재증명 정의 · 의무 · 해소" for v in r["resolve"])
              + " | 병합 전체 정의 · 의무 | CPU초: 재증명 " + " · ".join(r["resolve"]) + " · 병합 전체 |")
        print("| ---: | --- | " + " | ".join("---" for _ in r["resolve"]) + " | --- | --- |")
        for r in step:
            cells = []
            for v, x in r["resolve"].items():
                if x is None:
                    cells.append("교환")
                elif not x["merge"]:
                    cells.append("쓰기 겹침")
                else:
                    cells.append(f"{r['judge'][v]['n_conflicts']} · {x['defs']} · {x['obligations']} · "
                                 f"{'해소' if x['resolved'] else '충돌'}")
            c = r["step_cost"]
            print(f"| {r['k']} | {r['merge_proves']} | " + " | ".join(cells)
                  + f" | {r['whole'][1]} · {r['whole'][2]} | "
                  + " · ".join(f"{c.get(f'reprove_{v}', (0, 0.0))[1]:.2f}" for v in r["resolve"])
                  + f" · {c['verify_whole_cold'][1]:.1f} |")


if __name__ == "__main__":
    random.seed(0)
    main()
