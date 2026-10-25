#!/usr/bin/env python3
"""Token cost of M1 surface constructs (backlog 2R-02, input to 2R-04).

Deterministic, offline. No model runs. Two corpora:

  read  = bench/m1b/seed/src/*.bmb (canonical form: what slices and the agent read)
  write = the reference solutions of every write task in bench/m1 and bench/m1b
          (what an agent must emit to pass a task)

Each lexical token is attributed to exactly one construct category, so the
category shares add up to 100%. Token unit: lexical tokens (an approximation of
the BMB lexer) and bytes/4 (ADR-0021 §4). The script also reports the
counted-loop boilerplate, `#[trigger]` derivability and the `#[uses]` default
rules that `--uses-experiment` then checks with the real proof gate.

Usage:
  python3 bench/syntax/measure.py                 # tables (markdown)
  python3 bench/syntax/measure.py --uses-experiment target/release/bmb
  python3 bench/syntax/measure.py --trigger-experiment target/release/bmb
  python3 bench/syntax/measure.py --probe target/release/bmb  # habit_probes.txt: code, fix kind
  --seed 'GLOB' replaces the read corpus (e.g. 'bench/m1/seed/src/*.bmb',
  'tests/verify/pass/*.bmb'; the files must form one program).
  --uses-detail prints the #[uses] comparison per function.
"""

import glob
import json
import math
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

TOK = re.compile(
    r'"(?:[^"\\]|\\.)*"|[A-Za-z_][A-Za-z0-9_]*|\d[\d_]*|==>|::|->|=>|==|!=|<=|>=|&&|\|\||\.\.|//[^\n]*|\S'
)

CLAUSES = ("requires", "ensures", "decreases")


def lex(text):
    out = []
    for m in TOK.finditer(text):
        t = m.group(0)
        if t.startswith("//"):
            continue
        out.append(t)
    return out


def match_close(toks, i, open_, close):
    """Index of the token closing the group that opens at toks[i]."""
    depth = 0
    for j in range(i, len(toks)):
        if toks[j] == open_:
            depth += 1
        elif toks[j] == close:
            depth -= 1
            if depth == 0:
                return j
    raise ValueError("unbalanced")


class Item:
    def __init__(self, kind, name):
        self.kind = kind  # "fn" | "spec"
        self.name = name
        self.attrs = {}  # name -> list of entries
        self.clauses = {c: [] for c in CLAUSES}  # clause -> token list
        self.inv = []  # loop invariant tokens
        self.body = []
        self.mentions_spec_ctx = []  # tokens of contracts/invariants/asserts


def classify(toks, cats, items, unit):
    """Walk one unit's token list; add token counts to cats; collect items."""
    i = 0
    n = len(toks)
    pending_attrs = []

    def add(cat, k=1):
        cats[cat] = cats.get(cat, 0) + k

    while i < n:
        t = toks[i]
        if t == "#" and i + 1 < n and toks[i + 1] == "[":
            j = match_close(toks, i + 1, "[", "]")
            name = toks[i + 2]
            add("attr:" + name, j - i + 1)
            inner = toks[i + 4 : j - 1] if toks[i + 3] == "(" else []
            pending_attrs.append((name, [x for x in inner if x != ","]))
            i = j + 1
            continue
        if t == "mod":
            # `mod name {` and the closing `}` of the unit
            add("unit", 3)
            pending_attrs = []
            i += 3
            continue
        if t == "spec" and toks[i + 1] == "fn":
            name = toks[i + 2]
            j = toks.index("{", i)
            k = match_close(toks, j, "{", "}")
            it = Item("spec", unit + "::" + name)
            it.body = toks[j + 1 : k]
            it.total = k - i + 1
            items.append(it)
            add("spec_fn", k - i + 1)
            pending_attrs = []
            i = k + 1
            continue
        if t == "fn":
            name = toks[i + 1]
            it = Item("fn", unit + "::" + name)
            for a, entries in pending_attrs:
                it.attrs[a] = entries
            pending_attrs = []
            # signature: up to the first clause keyword or `{` at depth 0
            j = i
            depth = 0
            while True:
                x = toks[j]
                if x in "([":
                    depth += 1
                elif x in ")]":
                    depth -= 1
                if depth == 0 and (x in CLAUSES or x == "{"):
                    break
                j += 1
            sig = toks[i:j]
            # named result `-> (r: T)`: the `(`, name, `:` and `)` beyond `-> T`
            arrow = sig.index("->")
            ret = sig[arrow + 1 :]
            if ret[:1] == ["("] and len(ret) > 2 and ret[2] == ":":
                add("named_result", 4)
                add("sig", len(sig) - 4)
            elif ret == ["(", ")"]:
                add("unit_return", 3)
                add("sig", len(sig) - 3)
            else:
                add("sig", len(sig))
            # contract clauses
            cur = None
            while toks[j] != "{":
                if toks[j] in CLAUSES:
                    cur = toks[j]
                    add("clause_kw", 1)
                else:
                    it.clauses[cur].append(toks[j])
                    add("contract:" + cur, 1)
                j += 1
            k = match_close(toks, j, "{", "}")
            it.body = toks[j : k + 1]
            classify_body(it.body, add, it)
            items.append(it)
            i = k + 1
            continue
        if t == "}":
            add("unit", 0)  # counted in the `mod` triple
            i += 1
            continue
        add("other", 1)
        i += 1


def classify_body(body, add, it):
    i = 0
    n = len(body)
    while i < n:
        t = body[i]
        if t == "while":
            # header up to `invariant`/`decreases`/`{` at depth 0
            j = i + 1
            depth = 0
            while not (depth == 0 and body[j] in ("invariant", "decreases", "{")):
                if body[j] in "([":
                    depth += 1
                elif body[j] in ")]":
                    depth -= 1
                j += 1
            add("code", j - i)
            cur = None
            while body[j] != "{":
                if body[j] in ("invariant", "decreases"):
                    cur = body[j]
                    add("clause_kw", 1)
                else:
                    add("loop:" + cur, 1)
                    if cur == "invariant":
                        it.inv.append(body[j])
                j += 1
            i = j
            continue
        if t == "let":
            # `let [mut] name: T = ..;`: the `: T` part is the annotation
            j = i + 1
            if body[j] == "mut":
                j += 1
            j += 1  # name
            if body[j] == ":":
                k = body.index("=", j)
                add("let_type", k - j)
                add("code", j - i)
                i = k
                continue
        if t == "assert" and body[i + 1] == "(":
            k = match_close(body, i + 1, "(", ")")
            add("assert", k - i + 2)
            it.inv.extend(body[i + 2 : k])
            i = k + 2
            continue
        add("code", 1)
        i += 1


def units_of(text):
    """Split a file into (unit name, tokens) pairs (top-level units only)."""
    toks = lex(text)
    out = []
    i = 0
    while i < len(toks):
        if toks[i] == "mod":
            name = toks[i + 1]
            j = match_close(toks, i + 2, "{", "}")
            # attributes before `mod` (the summary) belong to the unit
            k = i
            while k >= 2 and toks[k - 1] == "]":
                k = max(x for x in range(k) if toks[x] == "#")
            out.append((name, toks[k : j + 1]))
            i = j + 1
        else:
            i += 1
    return out


def read_corpus(pattern="bench/m1b/seed/src/*.bmb"):
    files = sorted(glob.glob(os.path.join(ROOT, pattern)))
    return [(os.path.basename(f), open(f, encoding="utf-8").read()) for f in files]


def write_corpus():
    out = []
    for f in sorted(glob.glob(os.path.join(ROOT, "bench/m1*/tasks/w*.json"))):
        task = json.load(open(f, encoding="utf-8"))
        for op in task.get("reference", []):
            if op.get("op") == "put":
                # one unit name per task, so that same-named helpers of different tasks stay apart
                out.append((os.path.basename(f), op["unit"] + "@" + task["id"], op["code"]))
    return out


def tally(sources):
    cats = {}
    items = []
    bytes_total = 0
    for unit, text in sources:
        bytes_total += len(text.encode("utf-8"))
        for name, toks in units_of(text) if unit is None else [(unit, lex(text))]:
            classify(toks, cats, items, name)
    return cats, items, bytes_total


GROUPS = [
    ("실행 코드(식·문장)", ["code", "other"]),
    ("시그니처(이름·매개변수·반환 타입)", ["sig"]),
    ("`let`의 타입 표기 `: T`", ["let_type"]),
    ("`-> ()`", ["unit_return"]),
    ("이름 붙은 결과 `(r: …)`의 추가분", ["named_result"]),
    ("`#[effects(..)]`", ["attr:effects"]),
    ("`#[uses(..)]`", ["attr:uses"]),
    ("`#[summary(..)]`·`mod`", ["attr:summary", "unit"]),
    ("`requires` 절", ["contract:requires"]),
    ("`ensures` 절", ["contract:ensures"]),
    ("함수 `decreases` 절", ["contract:decreases"]),
    ("루프 `invariant`", ["loop:invariant"]),
    ("루프 `decreases`", ["loop:decreases"]),
    ("절 키워드(`requires`·`ensures`·`invariant`·`decreases`)", ["clause_kw"]),
    ("`assert(..);`", ["assert"]),
    ("`spec fn` 정의 전체", ["spec_fn"]),
]


def table(cats):
    total = sum(cats.values())
    rows = ["| 구성 | 어휘 토큰 | 비율 |", "| --- | ---: | ---: |"]
    seen = set()
    for label, keys in GROUPS:
        v = sum(cats.get(k, 0) for k in keys)
        seen.update(keys)
        rows.append(f"| {label} | {v} | {100 * v / total:.1f}% |")
    rest = sum(v for k, v in cats.items() if k not in seen)
    if rest:
        rows.append(f"| 기타 | {rest} | {100 * rest / total:.1f}% |")
    rows.append(f"| **합계** | **{total}** | 100% |")
    return "\n".join(rows)


def resolve(tok_list, i, unit):
    """Full path of the name at tok_list[i] (handles `unit::name`)."""
    if i + 2 < len(tok_list) and tok_list[i + 1] == "::":
        return tok_list[i] + "::" + tok_list[i + 2]
    return unit + "::" + tok_list[i]


def mentioned(tok_list, unit, names):
    out = set()
    for i, t in enumerate(tok_list):
        if i > 0 and tok_list[i - 1] == "::":
            continue
        if re.match(r"[A-Za-z_]", t) and i + 1 < len(tok_list) and tok_list[i + 1] in ("(", "::"):
            p = resolve(tok_list, i, unit)
            if p in names:
                out.add(p)
    return out


def called(tok_list, unit, names):
    return mentioned(tok_list, unit, names)


def uses_rules(items):
    """Compare each fn's #[uses] with deterministic default rules."""
    specs = {it.name: it for it in items if it.kind == "spec"}
    fns = {it.name: it for it in items if it.kind == "fn"}
    names = set(specs)
    fnames = set(fns)

    def unit_of(p):
        return p.split("::")[0]

    def closure(start):
        seen = set(start)
        todo = list(start)
        while todo:
            s = specs[todo.pop()]
            for m in mentioned(s.body, unit_of(s.name), names):
                if m not in seen:
                    seen.add(m)
                    todo.append(m)
        return seen

    rows = []
    for name, it in sorted(fns.items()):
        u = unit_of(name)
        actual = set()
        raw = it.attrs.get("uses", [])
        j = 0
        while j < len(raw):
            if j + 2 < len(raw) and raw[j + 1] == "::":
                actual.add(raw[j] + "::" + raw[j + 2])
                j += 3
            else:
                actual.add(u + "::" + raw[j])
                j += 1
        own = []
        for c in CLAUSES:
            own += it.clauses[c]
        own += it.inv
        r1 = mentioned(own, u, names)
        callee_ctx = set()
        for c in called(it.body, u, fnames):
            f = fns[c]
            ctoks = f.clauses["requires"] + f.clauses["ensures"]
            callee_ctx |= mentioned(ctoks, unit_of(c), names)
        r2 = r1 | callee_ctx
        rows.append(
            {
                "fn": name,
                "actual": sorted(actual),
                "r1": sorted(r1),
                "r1c": sorted(closure(r1)),
                "r2c": sorted(closure(r2)),
            }
        )
    return rows


def trigger_stats(items):
    """How many `forall`s have a single obvious trigger (array read of the bound var)?"""
    total = 0
    obvious = 0
    for it in items:
        toks = it.body if it.kind == "spec" else []
        for i, t in enumerate(toks):
            if t != "forall":
                continue
            total += 1
            # bound vars: `| k : usize , j : usize |`
            j = toks.index("|", i + 2)
            bound = [toks[x] for x in range(i + 2, j) if toks[x + 1 : x + 2] == [":"]]
            # body up to the end of the enclosing expression (approximate: rest of tokens)
            seg = toks[j + 1 :]
            trig = [x for x in range(len(seg)) if seg[x] == "trigger"]
            # obvious: one bound var, and every #[trigger] term is `name [ var ]`
            terms = set()
            for x in range(len(seg) - 4):
                if seg[x + 1] == "[" and seg[x + 3] == "]" and seg[x + 2] in bound:
                    terms.add((seg[x], seg[x + 2]))
            if len(bound) == 1 and len({a for a, _ in terms}) <= 2 and trig:
                obvious += 1
    return total, obvious


def counted_loops(sources):
    """Loops of the form `let mut k: usize = A; while k < X .. decreases X - k, { .. k = k + 1; }`."""
    total = 0
    counted = 0
    boiler = 0
    for unit, text in sources:
        toks = lex(text)
        for i, t in enumerate(toks):
            if t != "while":
                continue
            total += 1
            var = toks[i + 1]
            if toks[i + 2] != "<":
                continue
            # bound X: tokens until `&&`/invariant/decreases/{ at depth 0
            j = i + 3
            while toks[j] not in ("invariant", "decreases", "{", "&&"):
                j += 1
            if toks[j] == "&&":
                continue
            bound = toks[i + 3 : j]
            d = toks.index("decreases", i)
            dec = toks[d + 1 : toks.index(",", d)]
            if dec != bound + ["-", var]:
                continue
            ob = toks.index("{", d)
            cb = match_close(toks, ob, "{", "}")
            tail = toks[cb - 6 : cb]
            if tail != [var, "=", var, "+", "1", ";"]:
                continue
            counted += 1
            # boilerplate a `for var in A..X` form would not need:
            # `let mut k: usize = A;` (7 + |A|), the invariant `k <= X,` (3 + |X|),
            # `decreases X - k,` (3 + |X|), `k = k + 1;` (6), minus `for k in A..X` (4 + |A| + |X|)
            inv_s = toks.index("invariant", i) if "invariant" in toks[i:d] else None
            has_le = inv_s is not None and toks[inv_s + 1 : inv_s + 4 + len(bound)][:2] == [var, "<="]
            saved = 7 + 1 + (3 + len(bound) if has_le else 0) + (3 + len(bound)) + 6 - (4 + 1 + len(bound))
            boiler += saved
    return total, counted, boiler


def predicate_stats(items, srcs):
    """Where `spec fn`s are used, and what the named-predicate-only rule costs.

    A contract-only single-use predicate is a `spec fn` named by exactly one
    `requires`/`ensures` clause and by nothing else (no other spec body, no
    invariant). Inlining it would save its header (`spec fn name(params) -> bool { }`)
    and the call (`name(args)`), i.e. its tokens minus its body.
    """
    specs = {i.name: i for i in items if i.kind == "spec"}
    names = set(specs)
    where = {n: set() for n in names}
    contract_refs = {n: 0 for n in names}
    for it in items:
        u = it.name.split("::")[0]
        if it.kind == "fn":
            for c in ("requires", "ensures"):
                ms = mentioned(it.clauses[c], u, names)
                for m in ms:
                    where[m].add("contract")
                    contract_refs[m] += 1
            for m in mentioned(it.inv, u, names):
                where[m].add("inv")
        else:
            for m in mentioned(it.body, u, names):
                where[m].add("spec")
    single = [n for n in names if where[n] == {"contract"} and contract_refs[n] == 1]
    alltoks = []
    for _, text in srcs:
        alltoks += lex(text)
    # header and braces of the single-use predicates: what inlining them would save
    # (a lower bound: the call `name(args)` in the clause is saved too)
    single_overhead = sum(specs[n].total - len(specs[n].body) for n in single)
    single_total = sum(specs[n].total for n in single)
    no_forall = sum(1 for n in names if "forall" not in specs[n].body)
    clauses = 0
    for it in items:
        if it.kind == "fn":
            for c in ("requires", "ensures"):
                clauses += len(mentioned(it.clauses[c], it.name.split("::")[0], names))
    return {
        "spec_fns": len(names),
        "contract_only_single_use": len(single),
        "single_use_tokens": single_total,
        "single_use_header_tokens": single_overhead,
        "never_in_contract": sum(1 for n in names if "contract" not in where[n]),
        "without_forall": no_forall,
        "contract_predicate_calls": clauses,
        "trigger_tokens": 4 * alltoks.count("trigger"),
        "old_calls": alltoks.count("old"),
        "path_separators": alltoks.count("::"),
    }


def main():
    seed = "bench/m1b/seed/src/*.bmb"
    if "--seed" in sys.argv:
        seed = sys.argv[sys.argv.index("--seed") + 1]
    read = read_corpus(seed)
    rc, ritems, rbytes = tally([(None, t) for _, t in read])
    wsrc = write_corpus()
    wc, witems, wbytes = tally([(u, c) for _, u, c in wsrc])
    print(f"## read corpus: {seed}")
    print(f"files {len(read)}, bytes {rbytes}, approx tokens {math.ceil(rbytes / 4)}")
    print(f"defs: fn {sum(1 for i in ritems if i.kind == 'fn')}, spec fn {sum(1 for i in ritems if i.kind == 'spec')}")
    print(table(rc))
    print()
    print("## write corpus: reference solutions of m1 + m1b write tasks")
    print(f"puts {len(wsrc)}, bytes {wbytes}, approx tokens {math.ceil(wbytes / 4)}")
    print(f"defs: fn {sum(1 for i in witems if i.kind == 'fn')}, spec fn {sum(1 for i in witems if i.kind == 'spec')}")
    print(table(wc))
    print()
    for label, items, srcs in (
        ("read", ritems, [(None, t) for _, t in read]),
        ("write", witems, [(u, c) for _, u, c in wsrc]),
    ):
        tt, ob = trigger_stats(items)
        lt, lc, lb = counted_loops(srcs)
        print(f"{label}: forall {tt}, single-var array-read trigger {ob}; while {lt}, counted {lc}, boilerplate tokens a for-range form saves {lb}")
        print(f"{label}: {json.dumps(predicate_stats(items, srcs))}")
    rows = uses_rules(ritems)
    n = len(rows)
    eq_r1c = sum(1 for r in rows if set(r["actual"]) <= set(r["r1c"]))
    eq_r2c = sum(1 for r in rows if set(r["actual"]) <= set(r["r2c"]))
    entries = sum(len(r["actual"]) for r in rows)
    print(
        f"uses (read corpus): fns {n}, entries {entries}, "
        f"actual ⊆ closure(own clauses) {eq_r1c}/{n}, actual ⊆ closure(own + callee contracts) {eq_r2c}/{n}"
    )
    if "--uses-detail" in sys.argv:
        for r in rows:
            print(json.dumps(r, ensure_ascii=False))
    if "--uses-experiment" in sys.argv:
        bmb = sys.argv[sys.argv.index("--uses-experiment") + 1]
        uses_experiment(bmb, read, rows)


def uses_experiment(bmb, read, rows):
    """Replace every #[uses(..)] with a default rule's set and run the proof gate."""
    by_fn = {r["fn"]: r for r in rows}
    for rule in ("none", "r1c", "r2c"):
        with tempfile.TemporaryDirectory() as d:
            paths = []
            for fname, text in read:
                unit = None
                out = []
                pending_uses = False
                lines = text.split("\n")
                for li, line in enumerate(lines):
                    m = re.match(r"\s*mod (\w+) \{", line)
                    if m:
                        unit = m.group(1)
                    if line.strip().startswith("#[uses("):
                        continue
                    fm = re.match(r"(\s*)fn (\w+)\(", line)
                    if fm and rule != "none":
                        r = by_fn[unit + "::" + fm.group(2)]
                        s = r[rule]
                        if s:
                            short = [p.split("::", 1)[1] if p.split("::")[0] == unit else p for p in s]
                            out.append(f"{fm.group(1)}#[uses({', '.join(short)})]")
                    out.append(line)
                p = os.path.join(d, fname)
                open(p, "w", encoding="utf-8").write("\n".join(out))
                paths.append(p)
            res = subprocess.run([bmb, "verify"] + paths, capture_output=True, text=True)
            st = [json.loads(x) for x in res.stdout.splitlines() if x.startswith('{"def"')]
            ok = sum(1 for x in st if x["status"] == "verified")
            rl = sum(x.get("rlimit", 0) for x in st)
            rmax = max((x.get("rlimit_max", 0) for x in st), default=0)
            fails = [x["def"] for x in st if x["status"] != "verified"]
            print(f"uses rule {rule}: verified {ok}/{len(st)}, rlimit sum {rl}, max {rmax}, failed {fails}")
    # baseline
    with tempfile.TemporaryDirectory() as d:
        paths = []
        for fname, text in read:
            p = os.path.join(d, fname)
            open(p, "w", encoding="utf-8").write(text)
            paths.append(p)
        res = subprocess.run([bmb, "verify"] + paths, capture_output=True, text=True)
    st = [json.loads(x) for x in res.stdout.splitlines() if x.startswith('{"def"')]
    print(
        f"uses rule actual: verified {sum(1 for x in st if x['status'] == 'verified')}/{len(st)}, "
        f"rlimit sum {sum(x.get('rlimit', 0) for x in st)}, max {max(x.get('rlimit_max', 0) for x in st)}"
    )


def strip_and_place_triggers(text):
    """Remove `#[trigger]` from every one-variable `forall` and put one on the first
    `name[var]` term instead (the rule a fill tool would apply). Multi-variable
    `forall`s are left as written. Returns (new text, foralls rewritten)."""
    out_lines = []
    rewritten = 0
    for line in text.split("\n"):
        parts = re.split(r"(?=forall\|)", line)
        new = [parts[0]]
        for seg in parts[1:]:
            m = re.match(r"forall\|([^|]*)\|", seg)
            bound = [b.split(":")[0].strip() for b in m.group(1).split(",")]
            if len(bound) == 1:
                v = bound[0]
                seg2 = seg.replace("#[trigger] ", "")
                t = re.search(r"\b\w+\[" + re.escape(v) + r"\]", seg2)
                if t:
                    seg2 = seg2[: t.start()] + "#[trigger] " + seg2[t.start() :]
                    rewritten += 1
                    seg = seg2
            new.append(seg)
        out_lines.append("".join(new))
    return "\n".join(out_lines), rewritten


def trigger_experiment(bmb, read):
    with tempfile.TemporaryDirectory() as d:
        paths = []
        total = 0
        changed = 0
        for fname, text in read:
            new, k = strip_and_place_triggers(text)
            total += k
            changed += sum(1 for a, b in zip(new.split("\n"), text.split("\n")) if a != b)
            p = os.path.join(d, fname)
            open(p, "w", encoding="utf-8").write(new)
            paths.append(p)
        res = subprocess.run([bmb, "verify"] + paths, capture_output=True, text=True)
        st = [json.loads(x) for x in res.stdout.splitlines() if x.startswith('{"def"')]
        ok = sum(1 for x in st if x["status"] == "verified")
        print(
            f"trigger rule (first name[var] term): foralls rewritten {total}, lines that differ from the source {changed}, verified {ok}/{len(st)}, "
            f"rlimit sum {sum(x.get('rlimit', 0) for x in st)}, failed {[x['def'] for x in st if x['status'] != 'verified']}"
        )


def probe(bmb, path):
    """Check each line of PATH (one item, wrapped in a unit) and print the first
    diagnostic's code and its fix kind: `replacement` (mechanical), `message`
    (advice only) or `none`."""
    for line in open(path, encoding="utf-8").read().splitlines():
        if not line.strip():
            continue
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "x.bmb")
            open(p, "w", encoding="utf-8").write('#[summary("t")]\nmod t {\n' + line + "\n}\n")
            res = subprocess.run([bmb, "check", p], capture_output=True, text=True)
            diags = [json.loads(x) for x in res.stdout.splitlines() if x.startswith("{")]
            if diags:
                fix = diags[0].get("fix", {})
                kind = "replacement" if "replacement" in fix else ("message" if fix else "none")
                print(f"{diags[0]['code']}\t{kind}\t{line}")
            else:
                print(f"ok\t-\t{line}")


if __name__ == "__main__":
    if "--probe" in sys.argv:
        probe(sys.argv[sys.argv.index("--probe") + 1], os.path.join(ROOT, "bench/syntax/habit_probes.txt"))
    elif "--trigger-experiment" in sys.argv:
        seed = sys.argv[sys.argv.index("--seed") + 1] if "--seed" in sys.argv else "bench/m1b/seed/src/*.bmb"
        trigger_experiment(sys.argv[sys.argv.index("--trigger-experiment") + 1], read_corpus(seed))
    else:
        main()
