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
"""
import json
import os
import resource
import subprocess
import tempfile

VARIANTS = ("none", "own", "reader", "world")
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
        out[variant] = {"commute": not out["ww"] and not conflicts,
                        "reprove": {x: len(reprove[x]) for x in "ab"},
                        "conflicts": conflicts[:8], "n_conflicts": len(conflicts)}
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
