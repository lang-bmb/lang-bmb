#!/usr/bin/env python3
"""The commutation judgement of ADR-0065 over three versions of a program
(backlog T1-27): the base O and two edits A and B made on it, each a source
directory. Used by `run.py` (BCB pairs) and `semantic.py` (T1-19).

Everything comes from the compiler; nothing proves the merge.

  write set   the definitions and types whose own row changed (`bmb reads`:
              `self`, the qualified text with references by path; types by
              their declaration). ADR-0065 §1 says `id`; `id` is transitive
              (a callee's body edit changes every caller's `id`), so the
              `id` write set is computed too and reported (`ww_id`).
  read set    `bmb reads`: each definition's proof key as rows (own text,
              callee contracts, revealed spec fn texts, declarations, types).
              An edit X re-proves the definitions whose rows changed between
              O and X, except those whose only changes are callee contracts
              that are refinements for them (judged with their own
              `#[uses]`); the edit's read set is the rows of those.
  conflict    the two write sets overlap, or a definition one side re-proves
              reads a row the other side changed and that row is not exempt.
  exemption   only a `callee` row (an executable callee's contract) can be
              exempt: the change O -> X of that callee is a refinement for
              the reading proof. Variants:
                none     no exemption (every contract change read is a conflict)
                own      `bmb weaken O X`: each version's own `#[uses]`
                         (2D-19's comparison; unsound for readers that do not
                         reveal what the definition reveals)
                reader   `bmb weaken --readers`: the reader's `#[uses]`
                         (ADR-0065 §3 as drafted)
                world    reader, and no spec fn the query revealed (that the
                         two contracts reach through revealed bodies) was
                         written by the reader's side: the query ran on O's
                         bodies, the reader's proof sees its own side's
                         (correction found by T1-19, results.md)
              `undecided` and missing rows count as conflicts.
  old         the first replay's verdict (T1-12): written sets by `bmb hash`
              (body, iface), read sets by L2 edges out of the written
              definitions, every interface change read is a conflict.

`resolve` (T1-29, ADR-0065 §4) finishes the verdict of a pair judged
conflicting when the two write sets do not overlap: it checks the merge M
and proves in it only the conflict readers (the definitions whose read rows
the other side changed without exemption, that M still has) with `bmb
verify --def`. Every other definition of M keeps a proof by the relation: its
rows in M are one side's, or the other side's changes to them are exempt for
it. The pair is mergeable when that re-proof passes. `whole` proves the
whole merge without caches, the cost the step is compared with.

`decide` is the relation T1-31 recommends for ADR-0065 (decision 3 out of
the relation, decision 4 the default): `none` finished by `resolve`, the
readers proved through the incremental cache the build gate uses, so the
gate does not prove them again. No exemption query is asked. The other
variants stay for the comparisons of results.md.
"""
import json
import os
import resource
import subprocess
import tempfile

VARIANTS = ("none", "own", "reader", "world")
RESOLVE = ("none", "world")  # the sound variants `resolve` finishes (T1-29)
RECOMMENDED = "none"  # T1-31: no exemption, the overlap re-proved (`decide`)
QUERYING = ("own", "reader", "world")  # the variants that ask `bmb weaken`
EXEMPT = ("same", "refined", "compatible")


def jsonl(text):
    return [json.loads(x) for x in text.splitlines() if x.strip().startswith("{")]


def cpu():
    r = resource.getrusage(resource.RUSAGE_CHILDREN)
    return r.ru_utime + r.ru_stime


class Bmb:
    """The compiler, with CPU accounting per kind of call."""

    def __init__(self, path, cache):
        self.path, self.cache = path, cache
        self.cost = {}

    def run(self, kind, *args):
        t = cpu()
        r = subprocess.run([self.path, *args], capture_output=True, text=True)
        c = self.cost.setdefault(kind, [0, 0.0])
        c[0] += 1
        c[1] += cpu() - t
        return r.returncode, r.stdout

    def take(self):
        """The costs since the last call: {kind: (calls, cpu seconds)}."""
        out, self.cost = self.cost, {}
        return {k: (n, round(s, 3)) for k, (n, s) in out.items()}

    def proves(self, src, kind="verify"):
        return self.run(kind, "verify", "--cache", self.cache, src)[0] == 0

    def proves_stats(self, src, kind="verify"):
        """`proves`, and the definitions the warm caches encoded again."""
        with tempfile.NamedTemporaryFile("r", suffix=".json") as f:
            ok = self.run(kind, "verify", "--cache", self.cache, "--stats", f.name, src)[0] == 0
            return ok, len(json.loads(f.read() or "{}").get("encoded", []))

    def verify(self, src, defs=None, kind="verify_cold", cache=None):
        """`bmb verify` without caches (or through `cache`), of the whole
        program or (`defs`) of those definitions alone: (proved, definitions
        proved, obligations)."""
        args = [a for d in sorted(defs or ()) for a in ("--def", d)]
        if cache:
            args = ["--cache", cache] + args
        code, out = self.run(kind, "verify", *args, src)
        if code == 2:
            raise RuntimeError(f"bmb verify {args} {src}: {out[:400]}")
        st = [j for j in jsonl(out) if "status" in j]
        return code == 0, len(st), sum(j["obligations"] for j in st)

    def checks(self, src):
        return self.run("check", "check", src)[0] == 0

    def reads(self, src):
        code, out = self.run("reads", "reads", src)
        if code != 0:
            raise RuntimeError(f"bmb reads {src}: {out[:400]}")
        return Version(jsonl(out))

    def ids(self, src):
        _, out = self.run("hash", "hash", src)
        return {j["def"]: (j["id"], j["body"], j["iface"]) for j in jsonl(out) if "def" in j}

    def l2(self, src):
        _, out = self.run("map", "map", "--cache", self.cache, "L2", src)
        edges = {}
        for j in jsonl(out):
            if j.get("level") == "L2":
                edges.setdefault(j["from"], set()).add(j["to"])
        return edges

    def weaken(self, old, new):
        """`bmb weaken OLD NEW`: {def: verdict}, solver obligations."""
        _, out = self.run("weaken_own", "weaken", old, new)
        rows = jsonl(out)
        verdicts = {r["def"] or r["old"]: r["verdict"] for r in rows if "verdict" in r}
        q = next((r["queries"] for r in rows if r.get("summary")), 0)
        return verdicts, q

    def weaken_readers(self, old, new, asks):
        """`bmb weaken --readers`: {(def, uses): (verdict, revealed)}, obligations."""
        if not asks:
            return {}, 0
        asks = sorted(asks)
        with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False) as f:
            for d, uses in asks:
                f.write(json.dumps({"def": d, "uses": sorted(uses)}) + "\n")
            path = f.name
        try:
            _, out = self.run("weaken_reader", "weaken", "--readers", path, old, new)
        finally:
            os.unlink(path)
        rows = [r for r in jsonl(out) if "verdict" in r]
        if len(rows) != len(asks):
            raise RuntimeError(f"bmb weaken --readers: {len(rows)} rows for {len(asks)} readers: {out[:400]}")
        q = next((r["queries"] for r in jsonl(out) if r.get("summary")), 0)
        return {a: (r["verdict"], frozenset(r["revealed"])) for a, r in zip(asks, rows)}, q


class Version:
    """`bmb reads` of one version."""

    def __init__(self, rows):
        self.defs, self.types = {}, {}
        for r in rows:
            if "type" in r:
                self.types[r["type"]] = r["digest"]
            else:
                r["rows"] = {(x["kind"], x["path"]): x["digest"] for x in r["reads"]}
                r["uses"] = frozenset(r["uses"])
                self.defs[r["def"]] = r

    def value(self, kind, path):
        """What a reader's row (kind, path) holds in this version; None when
        the definition or type is not there."""
        d = self.defs.get(path)
        if kind == "type":
            return self.types.get(path)
        if d is None:
            return None
        if kind in ("self", "spec", "invariant"):
            return d["rows"][("self", path)]
        if kind == "component":
            return d["rows"][("component", path)]
        if kind == "callee":
            return d["contract"]
        if kind == "decl":
            return d["decl"]
        if kind == "names":
            return ""
        raise ValueError(kind)


def side(o, x):
    """What edit x did to base o: (write set, changed rows, per-definition row diffs)."""
    w = {d for d in set(o.defs) | set(x.defs)
         if o.value("self", d) != x.value("self", d)}
    w |= {t for t in set(o.types) | set(x.types) if o.types.get(t) != x.types.get(t)}
    memo = {}

    def changed(c):
        if c not in memo:
            memo[c] = o.value(*c) != x.value(*c)
        return memo[c]

    diffs = {}
    for d, r in x.defs.items():
        if d not in o.defs:
            diffs[d] = None  # new: proved from scratch
            continue
        r0 = o.defs[d]["rows"]
        diff = {c for c in set(r["rows"]) | set(r0) if r["rows"].get(c) != r0.get(c)}
        if diff:
            diffs[d] = diff
    return w, changed, diffs


def exemptible(diff, r0, r1):
    """The callee rows a re-proof would need exempt, or None when another
    row changed (own text, a revealed body, a changed declaration or type)."""
    if diff is None:
        return None
    callees = []
    for c in diff:
        kind = c[0]
        if kind == "callee" and c in r0 and c in r1:
            callees.append(c[1])
        elif kind in ("decl", "names", "type", "invariant") and (c in r0) != (c in r1):
            continue  # reached through a changed callee contract
        else:
            return None
    return callees


def judge(bmb, src_o, src_a, src_b, old=False, variants=VARIANTS):
    vo, va, vb = bmb.reads(src_o), bmb.reads(src_a), bmb.reads(src_b)
    sides = {"a": side(vo, va), "b": side(vo, vb)}
    vers = {"a": va, "b": vb}
    srcs = {"a": src_a, "b": src_b}
    other = {"a": "b", "b": "a"}
    # Every reader-mode question the variants can ask, per side X (O -> X):
    # X's own re-proofs, and the other side's readers of X's changed callees.
    asks = {"a": set(), "b": set()}
    for x in "ab":
        _, changed_x, diffs_x = sides[x]
        for d, diff in diffs_x.items():
            r0, r1 = vo.defs[d]["rows"] if diff is not None else {}, vers[x].defs[d]["rows"]
            for f in exemptible(diff, r0, r1) or []:
                asks[x].add((f, vers[x].defs[d]["uses"]))
        y = other[x]
        for d in sides[y][2]:
            for c in vers[y].defs[d]["rows"]:
                if c[0] == "callee" and changed_x(c) and vo.value(*c) is not None \
                        and vers[x].value(*c) is not None:
                    asks[x].add((c[1], vers[y].defs[d]["uses"]))
    if not any(v in QUERYING for v in variants):
        asks = {"a": set(), "b": set()}  # nothing to exempt: ask nothing (T1-31)
    answers, own, queries = {}, {}, {}
    for x in "ab":
        answers[x], qr = bmb.weaken_readers(src_o, srcs[x], asks[x])
        needs_own = {f for f, _ in asks[x]}
        if needs_own and "own" in variants:
            own[x], qo = bmb.weaken(src_o, srcs[x])
        else:
            own[x], qo = {}, 0
        queries[x] = (len(asks[x]), qr, qo)

    def exempt(variant, x, f, uses, y_writes=None):
        if variant == "none":
            return False
        if variant == "own":
            return own[x].get(f) in EXEMPT
        verdict, revealed = answers[x][(f, uses)]
        if verdict not in EXEMPT:
            return False
        if variant == "world" and y_writes is not None and revealed & y_writes:
            return False
        return True

    out = {"queries": queries}
    wa, wb = sides["a"][0], sides["b"][0]

    def in_merge(d):
        """Whether the definition-level merge has d: the side that wrote d
        decides (the write sets are disjoint when there is a merge)."""
        for x in "ab":
            if d in sides[x][0]:
                return d in vers[x].defs
        return d in vo.defs
    out["ww"] = sorted(wa & wb)
    for variant in variants:
        reprove = {}
        for x in "ab":
            _, _, diffs_x = sides[x]
            rp = set()
            for d, diff in diffs_x.items():
                r0 = vo.defs[d]["rows"] if diff is not None else {}
                callees = exemptible(diff, r0, vers[x].defs[d]["rows"])
                uses = vers[x].defs[d]["uses"]
                if callees is None or not all(exempt(variant, x, f, uses) for f in callees):
                    rp.add(d)
            reprove[x] = rp
        conflicts = []
        for x in "ab":
            y = other[x]
            changed_x = sides[x][1]
            for d in sorted(reprove[y]):
                r = vers[y].defs[d]
                for c in sorted(r["rows"]):
                    if not changed_x(c):
                        continue
                    if c[0] == "callee" and vo.value(*c) is not None and vers[x].value(*c) is not None \
                            and exempt(variant, x, c[1], r["uses"], sides[y][0]):
                        continue
                    conflicts.append((x, d, c[0], c[1]))
        readers = {d for _, d, _, _ in conflicts}
        out[variant] = {"commute": not out["ww"] and not conflicts,
                        "reprove": {x: len(reprove[x]) for x in "ab"},
                        "conflicts": conflicts[:8], "n_conflicts": len(conflicts),
                        "readers": sorted(d for d in readers if in_merge(d)),
                        "readers_gone": len([d for d in readers if not in_merge(d)])}
    # The `id` write set (ADR-0065 §1 as drafted).
    ho, ha, hb = bmb.ids(src_o), bmb.ids(src_a), bmb.ids(src_b)
    wia = {d for d in set(ho) | set(ha) if ho.get(d, (None,))[0] != ha.get(d, (None,))[0]}
    wib = {d for d in set(ho) | set(hb) if ho.get(d, (None,))[0] != hb.get(d, (None,))[0]}
    out["ww_id"] = len(wia & wib)
    out["w"] = (len(wa), len(wb))
    out["w_id"] = (len(wia), len(wib))
    if old:
        out["old"] = old_judge(bmb, ho, ha, hb, src_a, src_b)
    return out


def old_judge(bmb, h0, ha, hb, src_a, src_b):
    """The first replay's verdict (run.py of T1-12)."""
    def changed(h0, h1):
        b = lambda h: {d: (v[1], v[2]) for d, v in h.items()}  # noqa: E731
        h0, h1 = b(h0), b(h1)
        written = {d for d in set(h0) | set(h1) if h0.get(d) != h1.get(d)}
        iface = {d for d in h0 if d not in h1 or h0[d][1] != h1[d][1]}
        return written, iface
    ra, rb = bmb.l2(src_a), bmb.l2(src_b)
    wa, ia = changed(h0, ha)
    wb, ib = changed(h0, hb)
    reads_a = set().union(*(ra.get(d, set()) for d in wa)) if wa else set()
    reads_b = set().union(*(rb.get(d, set()) for d in wb)) if wb else set()
    return not (wa & wb) and not (ib & reads_a) and not (ia & reads_b)


def resolve(bmb, verdict, src_m, variant="world", cache=None):
    """ADR-0065 §4 (T1-29): the verdict of `variant` finished on the merge
    `src_m` (None when there is none). None when the pair commutes (nothing
    to do); otherwise {"resolved": the merge checks and every conflict reader
    it has proves, "defs": readers proved, "obligations"}. A pair whose
    write sets overlap has no merge and stays a conflict. With `cache` the
    readers are proved through that incremental cache (T1-31: the build
    gate on the merge then reuses their proofs)."""
    v = verdict[variant]
    if v["commute"]:
        return None
    if verdict["ww"] or src_m is None:
        return {"resolved": False, "defs": 0, "obligations": 0, "merge": False}
    kind = f"reprove_{variant}"
    if v["readers"]:
        ok, n, obl = bmb.verify(src_m, v["readers"], kind, cache)
    else:
        ok, n, obl = bmb.run(kind, "check", src_m)[0] == 0, 0, 0
    return {"resolved": ok, "defs": n, "obligations": obl, "merge": True}


def decide(bmb, src_o, src_a, src_b, src_m, cache=None):
    """The verdict T1-31 recommends for ADR-0065: the write and read sets
    without exemption (decision 3 out of the relation) and, for a conflict
    whose write sets do not overlap, the overlapping readers re-proved on
    the merge `src_m` (decision 4), through `cache` when given.
    {"mergeable", "commute", "verdict", "resolve"}."""
    verdict = judge(bmb, src_o, src_a, src_b, variants=(RECOMMENDED,))
    r = resolve(bmb, verdict, src_m, RECOMMENDED, cache)
    commute = verdict[RECOMMENDED]["commute"]
    return {"mergeable": commute or bool(r and r["resolved"]), "commute": commute,
            "verdict": verdict, "resolve": r}


def report_resolve(rows, families=False):
    """Markdown tables of `resolve` (T1-29) over rows of one set. Each row:
    "verdict" (`judge`'s output), "merge_proves" (the ground truth: the
    definition-level merge proved; None when both sides wrote one
    definition), "resolve", "whole", "cost" ({kind: (calls, seconds)} of the
    pair), and "family" when `families`, "merge_encoded" when the merge was
    also proved through the warm caches."""
    def final(r, v):
        return r["verdict"][v]["commute"] or bool(r["resolve"][v] and r["resolve"][v]["resolved"])

    def cpu_of(r, kind):
        return r["cost"].get(kind, (0, 0.0))[1]

    fails = [r for r in rows if r["merge_proves"] is not True]
    print("| 판정 | 충돌 판정 | 그중 쓰기 겹침(병합 없음) | 겹친 정의를 재증명한 쌍 | 해소(병합 가능) | "
          "**위반(해소인데 병합 실패)** | 병합 실패 중 최종 충돌 | 거짓 충돌: 단계 전 → 후 |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |")
    for v in RESOLVE:
        conf = [r for r in rows if not r["verdict"][v]["commute"]]
        ww = [r for r in conf if not r["resolve"][v]["merge"]]
        step = [r for r in conf if r["resolve"][v]["merge"]]
        res = [r for r in step if r["resolve"][v]["resolved"]]
        before = sum(1 for r in conf if r["merge_proves"] is True)
        after = sum(1 for r in rows if not final(r, v) and r["merge_proves"] is True)
        print(f"| `{v}` + 재증명 | {len(conf)} | {len(ww)} | {len(step)} | {len(res)} | "
              f"{sum(1 for r in res if r['merge_proves'] is not True)} | "
              f"{sum(1 for r in fails if not final(r, v))} / {len(fails)} | {before} → {after} |")
    print("\n| 판정 | 재증명한 쌍 | 겹친 정의 재증명: 정의 · 의무 · CPU초 | 병합 전체(캐시 없음): 정의 · 의무 · CPU초 | "
          "CPU 비율 | 의무 비율 | 면제 질의 CPU초(이 쌍들) |")
    print("| --- | ---: | --- | --- | ---: | ---: | ---: |")
    for v in RESOLVE:
        step = [r for r in rows if r["resolve"][v] and r["resolve"][v]["merge"]]
        sd = sum(r["resolve"][v]["defs"] for r in step)
        so = sum(r["resolve"][v]["obligations"] for r in step)
        sc = sum(cpu_of(r, f"reprove_{v}") for r in step)
        wd = sum(r["whole"][1] for r in step)
        wo = sum(r["whole"][2] for r in step)
        wc = sum(cpu_of(r, "verify_whole_cold") for r in step)
        q = sum(cpu_of(r, "weaken_reader") for r in step) if v != "none" else 0.0
        print(f"| `{v}` | {len(step)} | {sd} · {so} · {sc:.1f} | {wd} · {wo} · {wc:.1f} | "
              f"{sc / wc if wc else 0:.3f} | {so / wo if wo else 0:.3f} | {q:.1f} |")
    warm = [r for r in rows if r["resolve"]["world"] and r["resolve"]["world"]["merge"]
            and r.get("merge_encoded") is not None]
    if warm:
        print(f"\n같은 `world` 쌍 {len(warm)}개의 병합을 따뜻한 캐시(기준·A·B를 증명한 증분 캐시)로 증명하면 "
              f"다시 인코딩한 정의 {sum(r['merge_encoded'] for r in warm)}개, CPU "
              f"{sum(cpu_of(r, 'verify_merge') for r in warm):.1f}초다.")
    if families:
        print("\n| 가족 | 쌍 | 병합 실패 | `world` 충돌 | 쓰기 겹침 | 해소 | 위반 | 거짓 충돌 전 → 후 | "
              "재증명 정의 · 의무 | 병합 전체 정의 · 의무 |")
        print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |")
        for fam in sorted({r["family"] for r in rows}):
            rs = [r for r in rows if r["family"] == fam]
            conf = [r for r in rs if not r["verdict"]["world"]["commute"]]
            step = [r for r in conf if r["resolve"]["world"]["merge"]]
            res = [r for r in step if r["resolve"]["world"]["resolved"]]
            print(f"| `{fam}` | {len(rs)} | {sum(1 for r in rs if r['merge_proves'] is not True)} | {len(conf)} | "
                  f"{len(conf) - len(step)} | {len(res)} | "
                  f"{sum(1 for r in res if r['merge_proves'] is not True)} | "
                  f"{sum(1 for r in conf if r['merge_proves'] is True)} → "
                  f"{sum(1 for r in rs if not final(r, 'world') and r['merge_proves'] is True)} | "
                  f"{sum(r['resolve']['world']['defs'] for r in step)} · "
                  f"{sum(r['resolve']['world']['obligations'] for r in step)} | "
                  f"{sum(r['whole'][1] for r in step)} · {sum(r['whole'][2] for r in step)} |")
