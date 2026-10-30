#!/usr/bin/env python3
"""H-COMMUTE: edits with large overlapping reads (backlog T1-31).

  hot.py gen OUT.jsonl                     (the frozen set; MANIFEST pins it)
  hot.py run PAIRS.jsonl --work DIR [--bmb PATH] [--jobs J] [--rows R.jsonl] [--only K,..] > results.md
  hot.py report PAIRS.jsonl R.jsonl > results.md

T1-29 measured the re-proof of ADR-0065 decision 4 on pairs whose overlap
was one or two definitions. This set makes the overlap large on purpose: a
heavily called contract f (N callers) is changed by edit A while edit B
touches every caller, so both sides re-prove all N callers and the
overlapping read set is N definitions. The families:

  hot_sweep   A refines f (drops a requires clause); B rewrites the argument
              of every caller. The merge proves.
  hot_two     A refines f; B refines g (adds an ensures clause); every caller
              calls both. The merge proves.
  hot_break   A strengthens f (adds a requires clause every base caller
              meets); B rewrites the arguments and one caller's new argument
              breaks the added clause. The merge fails (negative control).

Each family is drawn for N in NS and two shapes of the callers' `#[uses]`:
`shared` (every caller reveals the same spec fns, so the exemption query
asks one (callee, uses) question per side) and `distinct` (each caller also
reveals a spec fn of its own, so it asks N). The background is the BCB tree
of semantic.py (D definitions).

Per pair (`run`), with one fresh proof cache:
  valid      O, A and B each prove (the cache then holds what they proved).
  gate       the definition-level merge M proved through that warm cache
             (the build gate after the merge, ADR-0009): the ground truth,
             the definitions it encoded again and its CPU.
  judge      judge.py's `none` and `world` verdicts (reads, exemption query).
  resolve    ADR-0065 §4 on M for both verdicts: the conflict readers proved
             without caches (`verify --def`), as T1-29 measured.
  shared     on a copy of the warm cache: the `none` readers proved through
             it, then the gate; how much of the re-proof the gate reuses.
  whole      M proved without caches.
CPU is the children's user+system time (judge.Bmb), so the shared machine's
load does not enter it.
"""
import argparse
import json
import os
import random
import shutil
import sys
import tempfile
from concurrent.futures import ProcessPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, HERE)
import judge  # noqa: E402
import semantic  # noqa: E402
from semantic import Scenario, fn, spec  # noqa: E402

FORMAT = 1
FROZEN = {"seed": 1, "defs": 1000}
NS = (1, 4, 16, 64)
SHAPES = ("shared", "distinct")


def callers(n):
    return [f"c{i}" for i in range(n)]


def common(s, rng, n, distinct, req):
    """The spec fns, f (requires `req`, ensures out) and the N callers' own
    spec fns; returns (H, K, f_text)."""
    S = s
    h = rng.randint(100, 900)
    k = rng.randint(0, h)
    s.put(s.base, "in", spec(S.name("in"), "x: i64", f"0 <= x && x <= {h}"))
    s.put(s.base, "nz", spec(S.name("nz"), "x: i64", f"x != {k}"))
    s.put(s.base, "out", spec(S.name("out"), "r: i64", "0 <= r && r <= 1000"))
    if distinct:
        for i in range(n):
            s.put(s.base, f"t{i}", spec(S.name(f"t{i}"), "x: i64", f"x != {-1 - i}"))

    def f_text(req):
        return fn(S.name("f"), "x: i64", "(r: i64)", ["x + 1"], uses=S.refs(["in", "out"], "f"),
                  requires=[f"{S.ref(q, 'f')}(x)" for q in req], ensures=[f"{S.ref('out', 'f')}(r)"])
    s.put(s.base, "f", f_text(req))
    return h, k, f_text


def arg(rng, h, k):
    v = rng.randint(0, h - 1)
    return v + (v >= k)


def caller(S, i, body, reveal, distinct):
    c = f"c{i}"
    uses = list(reveal) + ([f"t{i}"] if distinct else [])
    return fn(S.name(c), "", "i64", [body], uses=S.refs(uses, c))


def f_hot_sweep(rng, mk, n, distinct):
    roles = ["in", "nz", "out", "f"] + callers(n) + ([f"t{i}" for i in range(n)] if distinct else [])
    s = mk(roles)
    S = s
    h, k, f_text = common(s, rng, n, distinct, ["in", "nz"])
    for i in range(n):
        v0, v1 = arg(rng, h, k), arg(rng, h, k)
        s.put(s.base, f"c{i}", caller(S, i, f"{S.ref('f', f'c{i}')}({v0})", ["in", "nz", "out"], distinct))
        s.set(s.b, f"c{i}", caller(S, i, f"{S.ref('f', f'c{i}')}({v1})", ["in", "nz", "out"], distinct))
    s.set(s.a, "f", f_text(["in"]))
    return s, {"h": h, "k": k}


def f_hot_two(rng, mk, n, distinct):
    roles = ["in", "nz", "out", "lo", "tight", "f", "g"] + callers(n) \
        + ([f"t{i}" for i in range(n)] if distinct else [])
    s = mk(roles)
    S = s
    h, k, f_text = common(s, rng, n, distinct, ["in", "nz"])
    c = rng.randint(100, 900)
    b = rng.randint(1, c)
    t = rng.randint(b, c)
    s.put(s.base, "lo", spec(S.name("lo"), "r: i64", f"0 <= r && r <= {c}"))
    s.put(s.base, "tight", spec(S.name("tight"), "r: i64", f"r <= {t}"))

    def g_text(post):
        return fn(S.name("g"), "x: i64", "(r: i64)", [f"if x > {b} {{ {b} }} else {{ x }}"],
                  uses=S.refs(["in"] + post, "g"), requires=[f"{S.ref('in', 'g')}(x)"],
                  ensures=[f"{S.ref(q, 'g')}(r)" for q in post])
    s.put(s.base, "g", g_text(["lo"]))
    for i in range(n):
        v, w = arg(rng, h, k), rng.randint(0, h)
        cur = f"c{i}"
        s.put(s.base, cur, caller(S, i, f"{S.ref('f', cur)}({v}) + {S.ref('g', cur)}({w})",
                                  ["in", "nz", "out", "lo"], distinct))
    s.set(s.a, "f", f_text(["in"]))
    s.set(s.b, "g", g_text(["lo", "tight"]))
    return s, {"h": h, "k": k, "c": c, "t": t}


def f_hot_break(rng, mk, n, distinct):
    roles = ["in", "nz", "out", "f"] + callers(n) + ([f"t{i}" for i in range(n)] if distinct else [])
    s = mk(roles)
    S = s
    h, k, f_text = common(s, rng, n, distinct, ["in"])
    j = rng.randrange(n)
    for i in range(n):
        v0 = arg(rng, h, k)
        v1 = k if i == j else arg(rng, h, k)
        s.put(s.base, f"c{i}", caller(S, i, f"{S.ref('f', f'c{i}')}({v0})", ["in", "nz", "out"], distinct))
        s.set(s.b, f"c{i}", caller(S, i, f"{S.ref('f', f'c{i}')}({v1})", ["in", "nz", "out"], distinct))
    s.set(s.a, "f", f_text(["in", "nz"]))
    return s, {"h": h, "k": k, "breaks": j}


FAMILIES = {"hot_sweep": f_hot_sweep, "hot_two": f_hot_two, "hot_break": f_hot_break}


# --- gen ----------------------------------------------------------------------

def generate(seed, defs):
    bg = semantic.background(defs)
    n_units = len(bg)
    rng = random.Random(f"commute-hot:{seed}")
    header = {"format": FORMAT, "set": "commute-hot", "seed": seed, "defs": defs,
              "families": sorted(FAMILIES), "ns": list(NS), "shapes": list(SHAPES)}
    lines = [header]
    k = 0
    for fam in sorted(FAMILIES):
        for shape in SHAPES:
            for n in NS:
                prefix = f"h{k}"
                s, params = FAMILIES[fam](rng, lambda roles: Scenario(rng, prefix, n_units, roles, False),
                                          n, shape == "distinct")
                sizes = [len(u["blocks"]) for u in bg]
                for op in s.base:
                    sizes[op["unit"]] += 1
                    op["at"] = rng.randint(0, sizes[op["unit"]] - 1)
                lines.append({"k": k, "family": fam, "shape": shape, "n": n, "params": params,
                              "base": s.base, "a": s.a, "b": s.b})
                k += 1
    return lines


def frozen_name():
    return f"pairs/hot-s{FROZEN['seed']}-d{FROZEN['defs']}.jsonl"


# --- run ----------------------------------------------------------------------

def measure(job):
    """One pair (see the module comment); runs in a worker process."""
    bmb_path, work, defs, pair = job
    pdir = tempfile.mkdtemp(prefix=f"hot-{pair['k']}-", dir=work)
    try:
        bg = semantic.background(defs)
        base, a, b, m = semantic.trees(bg, pair)
        src = {n: t.write(os.path.join(pdir, n)) for n, t in (("o", base), ("a", a), ("b", b), ("m", m))}
        warm = os.path.join(pdir, "cache")
        bmb = judge.Bmb(bmb_path, warm)
        row = {"k": pair["k"], "family": pair["family"], "shape": pair["shape"], "n": pair["n"],
               "params": pair["params"]}
        valid = {n: bmb.proves(src[n], "verify_sides") for n in "oab"}
        row["valid"] = all(valid.values())
        if not row["valid"]:
            row["invalid"] = [n for n, ok in valid.items() if not ok]
            return row
        bmb.take()
        shared = os.path.join(pdir, "cache2")
        shutil.copytree(warm, shared)
        # The build gate after the merge, through the warm cache.
        row["merge_proves"], row["gate_encoded"] = bmb.proves_stats(src["m"], "gate")
        verdict = judge.judge(bmb, src["o"], src["a"], src["b"], variants=judge.RESOLVE)
        row["verdict"] = verdict
        row["resolve"] = {v: judge.resolve(bmb, verdict, src["m"], v) for v in judge.RESOLVE}
        # The `none` readers proved through a copy of the warm cache, then
        # the gate through it.
        readers = verdict["none"]["readers"]
        shared_bmb = judge.Bmb(bmb_path, shared)
        if readers:
            args = [x for d in readers for x in ("--def", d)]
            shared_bmb.run("reprove_shared", "verify", "--cache", shared, *args, src["m"])
        _, row["gate_after_encoded"] = shared_bmb.proves_stats(src["m"], "gate_after")
        row["whole"] = bmb.verify(src["m"], kind="verify_whole_cold")
        row["cost"] = {**bmb.take(), **shared_bmb.take()}
        return row
    finally:
        shutil.rmtree(pdir, ignore_errors=True)


def run(args):
    lines = [json.loads(x) for x in open(args.pairs)]
    header, pairs = lines[0], lines[1:]
    if args.only:
        want = {int(x) for x in args.only.split(",")}
        pairs = [p for p in pairs if p["k"] in want]
    os.makedirs(args.work, exist_ok=True)
    work = os.path.abspath(args.work)
    jobs = [(args.bmb, work, header["defs"], p) for p in pairs]
    rows = []
    with ProcessPoolExecutor(max_workers=args.jobs) as ex:
        for row in ex.map(measure, jobs):
            rows.append(row)
            print(json.dumps(row, sort_keys=True), file=sys.stderr, flush=True)
            if args.rows:
                with open(args.rows, "a") as f:
                    f.write(json.dumps(row, sort_keys=True) + "\n")
    report(header, rows)


# --- report -------------------------------------------------------------------

def report(header, rows):
    def cpu(r, kind):
        return r["cost"].get(kind, (0, 0.0))[1]

    used = [r for r in rows if r["valid"]]
    print(f"## 겹친 읽기가 큰 편집: 시드 {header['seed']}, 배경 정의 {header['defs']}개\n")
    print("생성: `python3 dev-docs/experiments/commute/hot.py gen P.jsonl`(동결: `MANIFEST`), "
          "측정: `hot.py run P.jsonl --work W --jobs 2`.\n")
    print(f"쌍 {len(rows)}개 중 기준·A·B가 각자 증명된 쌍 {len(used)}개"
          f"(표본 제외: {', '.join(str(r['k']) for r in rows if not r['valid']) or '없음'}).\n")
    print("| k | 가족 | 호출자 `#[uses]` | N | 병합 증명 | `none`: 판정 · 재증명 정의 · 의무 · 해소 | "
          "`world`: 판정 · 면제 질의 행 · 재증명 정의 · 해소 | CPU초: 읽기 집합(`reads` 3회) | "
          "면제 질의 | 재증명 `none` | 재증명 `world` | 병합 전체(캐시 없음) | 게이트(따뜻한 캐시): 정의 · 초 | "
          "재증명 뒤 게이트: 정의 · 초(재증명 · 게이트) |")
    print("| ---: | --- | --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |")
    for r in sorted(used, key=lambda r: r["k"]):
        v = r["verdict"]

        def cell(name):
            x = r["resolve"][name]
            if x is None:
                return "교환"
            return (f"충돌 · {x['defs']} · {x['obligations']} · {'해소' if x['resolved'] else '실패'}")
        asks = v["queries"]["a"][0] + v["queries"]["b"][0]
        wx = r["resolve"]["world"]
        wcell = "교환 · " + str(asks) + " · 0 · —" if wx is None else \
            f"충돌 · {asks} · {wx['defs']} · {'해소' if wx['resolved'] else '실패'}"
        print(f"| {r['k']} | `{r['family']}` | {r['shape']} | {r['n']} | {r['merge_proves']} | {cell('none')} | "
              f"{wcell} | {cpu(r, 'reads'):.1f} | {cpu(r, 'weaken_reader'):.1f} | {cpu(r, 'reprove_none'):.1f} | "
              f"{cpu(r, 'reprove_world'):.1f} | {cpu(r, 'verify_whole_cold'):.1f} | "
              f"{r['gate_encoded']} · {cpu(r, 'gate'):.1f} | {r['gate_after_encoded']} · "
              f"{cpu(r, 'reprove_shared') + cpu(r, 'gate_after'):.1f}"
              f"({cpu(r, 'reprove_shared'):.1f} · {cpu(r, 'gate_after'):.1f}) |")
    # Violations and false conflicts after the step, per verdict.
    print("\n| 판정 | 교환 | 충돌 | 해소 | **위반** | 병합 실패 중 최종 충돌 | 거짓 충돌(단계 뒤) |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for name in judge.RESOLVE:
        com = [r for r in used if r["resolve"][name] is None]
        conf = [r for r in used if r["resolve"][name] is not None]
        res = [r for r in conf if r["resolve"][name]["resolved"]]
        final = com + res
        fails = [r for r in used if r["merge_proves"] is not True]
        print(f"| `{name}` + 재증명 | {len(com)} | {len(conf)} | {len(res)} | "
              f"{sum(1 for r in final if r['merge_proves'] is not True)} | "
              f"{sum(1 for r in fails if r not in final)} / {len(fails)} | "
              f"{sum(1 for r in used if r not in final and r['merge_proves'] is True)} |")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    g = sub.add_parser("gen")
    g.add_argument("out")
    r = sub.add_parser("run")
    r.add_argument("pairs")
    r.add_argument("--work", required=True)
    r.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    r.add_argument("--jobs", type=int, default=1)
    r.add_argument("--only", help="comma-separated pair numbers")
    r.add_argument("--rows", help="append the per-pair rows (JSONL) here as they finish")
    t = sub.add_parser("report")
    t.add_argument("pairs")
    t.add_argument("rows")
    args = ap.parse_args()
    if args.cmd == "gen":
        with open(args.out, "w") as f:
            f.write(semantic.dump(generate(**FROZEN)))
    elif args.cmd == "report":
        header = json.loads(open(args.pairs).readline())
        report(header, [json.loads(x) for x in open(args.rows)])
    else:
        run(args)


if __name__ == "__main__":
    main()
