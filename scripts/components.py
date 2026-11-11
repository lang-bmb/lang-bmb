#!/usr/bin/env python3
"""Compiler components: sizes, the port ledger and the pin (4D-08, 4D-10).

  components.py check            every stage0 file is in one component, port
                                 files exist, no pinned component drifted,
                                 the ledger file is current
  components.py table            component sizes (port map §7.1)
  components.py ledger [--write] the generated block of the port ledger
  components.py b0               gate B0: the whole port estimated from the
                                 components ported so far (ADR-0116, ADR-0122)
  components.py pin ID           pin a component at the current files
  components.py repin ID         accept a change to a pinned component

dev-docs/health/components.json lists, for each component of the port map,
its stage0 files (under compiler/src), its port files (under selfhost/) and
its pin. A component is pinned once its port passes the differential test
(ADR-0113): from then on a change to its stage0 files must come with a change
to the port. `check` fails when a pinned component's stage0 files differ from
the pin; `repin` records the new state and refuses when the port did not
change too. The differential test itself is a gate job of its own.

The ledger (dev-docs/measurements/port-ledger.md) has one row per component:
what the files give (lines, definitions, escape hatches, the proved share of
the port) and what the sessions that wrote the port cost (entered under
`ledger` in components.json). Standard library only.
"""
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FILE = Path("dev-docs/health/components.json")
LEDGER = Path("dev-docs/measurements/port-ledger.md")
BEGIN = "<!-- ledger:data begin (scripts/components.py ledger --write; do not edit by hand) -->"
END = "<!-- ledger:data end -->"
# The end of the early fixed point (port map §4): components before it run
# without the prover.
E1 = ("C0", "C1", "C2", "C3", "C4", "C8")


def load(root=ROOT):
    return json.loads((root / FILE).read_text(encoding="utf-8"))


def save(doc, root=ROOT):
    (root / FILE).write_text(json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def stage0_lines(path):
    """Lines before the tests, as the port map counts (selfhost-gap-stats.py)."""
    text = path.read_text(encoding="utf-8")
    i = text.find("#[cfg(test)]")
    return len((text if i < 0 else text[:i]).splitlines())


def digest(paths):
    """One hash of the files' names and bytes, in name order."""
    h = hashlib.sha256()
    for p in sorted(paths):
        h.update(str(p[0]).encode() + b"\0" + hashlib.sha256(p[1].read_bytes()).digest())
    return h.hexdigest()


def stage0_files(c, root):
    return [(f, root / "compiler" / "src" / f) for f in c["stage0"]]


def port_files(c, root):
    return [(f, root / f) for f in c["port"]]


def state(c, root=ROOT):
    return {"stage0": digest(stage0_files(c, root)), "port": digest(port_files(c, root))}


def problems(root=ROOT):
    doc = load(root)
    out, owner = [], {}
    src = root / "compiler" / "src"
    for c in doc["components"]:
        for f in c["stage0"]:
            if f in owner:
                out.append(f"{f} is in {owner[f]} and {c['id']}")
            owner[f] = c["id"]
            if not (src / f).is_file():
                out.append(f"{c['id']}: compiler/src/{f} does not exist")
        for f in c["port"]:
            if not f.startswith("selfhost/") or not (root / f).is_file():
                out.append(f"{c['id']}: port file {f} is not a file under selfhost/")
        if c["pin"] and not c["port"]:
            out.append(f"{c['id']} is pinned without a port")
    for p in sorted(src.rglob("*.rs")):
        rel = p.relative_to(src).as_posix()
        if rel not in owner:
            out.append(f"compiler/src/{rel} is in no component: add it to {FILE}")
    if out:
        return out  # the hashes below need the files
    for c in doc["components"]:
        if c["pin"] and state(c, root)["stage0"] != c["pin"]["stage0"]:
            out.append(f"{c['id']} is pinned (ADR-0113) and its stage0 files changed: "
                       f"change the port in the same commit, then `scripts/components.py repin {c['id']}`")
    return out


def pin(cid, again, root=ROOT):
    doc = load(root)
    c = next((x for x in doc["components"] if x["id"] == cid), None)
    if c is None:
        raise SystemExit(f"no component {cid}")
    if not c["port"]:
        raise SystemExit(f"{cid} has no port files: a component is pinned when its port passes the differential test")
    now = state(c, root)
    if again:
        if not c["pin"]:
            raise SystemExit(f"{cid} is not pinned: use `pin`")
        if now["stage0"] != c["pin"]["stage0"] and now["port"] == c["pin"]["port"]:
            raise SystemExit(f"{cid}: the stage0 files changed and the port did not (ADR-0113 decision 2)")
    elif c["pin"]:
        raise SystemExit(f"{cid} is already pinned: use `repin`")
    c["pin"] = now
    save(doc, root)
    return now


# ---------------------------------------------------------------- ledger


def bmb_bin(root):
    return os.environ.get("BMB", str(root / "target" / "debug" / "bmb"))


SPEC_ITEM = re.compile(r"(spec|proof) fn ")
SPEC_LINE = re.compile(r"(requires|ensures|invariant|decreases) |assert\(|#\[(uses|invariant|trigger)\b|proof \{")


def spec_lines(text):
    """Lines of a normal-form source (`bmb fmt`) that only the proof reads
    (ADR-0122 decision 1): `spec fn` and `proof fn` items, `proof { }` blocks,
    and the lines that start with a clause (`requires`, `ensures`,
    `invariant`, `decreases`), an `assert(` or the attributes `#[uses]`,
    `#[invariant]`, `#[trigger]`. The rest is what an unproved build runs."""
    n = depth = 0
    inside = opened = False
    for line in text.splitlines():
        t = line.strip()
        if not inside and (SPEC_ITEM.match(t) or t.startswith("proof {")):
            inside, opened, depth = True, False, 0
        if inside:
            n += 1
            depth += t.count("{") - t.count("}")
            opened = opened or "{" in t
            inside = not (opened and depth <= 0)
        elif SPEC_LINE.match(t):
            n += 1
    return n


def port_facts(c, root):
    """What the port files give (ADR-0122): lines and those only the proof
    reads, definitions and those proved without a trusted fact, obligations,
    the proof's cost, escape hatches. None for a component without a port."""
    files = [p for _, p in port_files(c, root)]
    if not files:
        return None
    texts = [p.read_text(encoding="utf-8") for p in files]
    text = "".join(texts)
    facts = {
        "lines": len(text.splitlines()),
        "spec": sum(spec_lines(t) for t in texts),
        "escapes": len(re.findall(r"#\[external_body\]|\bassume\(", text)),
    }
    r = subprocess.run([bmb_bin(root), "verify", *map(str, files)], capture_output=True, text=True, cwd=root)
    rows = [json.loads(l) for l in r.stdout.splitlines() if l.startswith("{")]
    defs = [x for x in rows if "status" in x and "def" in x]
    facts["defs"] = len(defs)
    facts["proved"] = sum(x["status"] == "verified" and "trust" not in x for x in defs)
    facts["obligations"] = sum(x.get("obligations", 0) for x in defs)
    facts["rlimit"] = sum(x.get("rlimit", 0) for x in defs)
    return facts


def num(v):
    return "—" if v is None else f"{v:,}" if isinstance(v, int) else str(v)


def ledger_block(root=ROOT):
    doc = load(root)
    head = ["ID", "구성 요소", "국면", "stage0 줄", "포트 줄", "명세 줄", "실행 배", "전체 배", "정의", "증명된 정의",
            "의무", "rlimit", "탈출구", "세션 토큰", "게이트 실행", "미결정으로 끝난 실행", "고정"]
    lines = [BEGIN, "", "| " + " | ".join(head) + " |", "|" + " --- |" * len(head)]
    total0 = total1 = 0
    friction = []
    for c in doc["components"]:
        s0 = sum(stage0_lines(p) for _, p in stage0_files(c, root))
        f = port_facts(c, root)
        total0 += s0
        total1 += f["lines"] if f else 0
        led = c["ledger"]
        g = f or dict.fromkeys(("lines", "spec", "defs", "obligations", "rlimit", "escapes"))
        lines.append("| " + " | ".join([
            c["id"], c["name"], c["phase"], num(s0), num(g["lines"]), num(g["spec"]),
            f"{(f['lines'] - f['spec']) / s0:.1f}" if f else "—", f"{f['lines'] / s0:.1f}" if f else "—",
            num(g["defs"]), f"{f['proved']}/{f['defs']}" if f else "—", num(g["obligations"]), num(g["rlimit"]),
            num(g["escapes"]), num(led["session_tokens"]), num(led["gate_runs"]), num(led["undecided_runs"]),
            "고정" if c["pin"] else "—"]) + " |")
        friction += [f"- {c['id']}: {x}" for x in led["friction"]]
    e1 = sum(sum(stage0_lines(p) for _, p in stage0_files(c, root)) for c in doc["components"] if c["id"] in E1)
    lines += ["", f"stage0 합계 {total0:,}줄, 조기 고정점(E1: {', '.join(E1)}) 범위 {e1:,}줄({e1 * 100 // total0}%), "
              f"포트 합계 {total1:,}줄.", "", "마찰:" if friction else "마찰: 아직 없음.", *friction, "", END]
    return "\n".join(lines)


def proved_share(root=ROOT):
    """Health S7: proved definitions / definitions over every port file, or
    None while there is no port."""
    proved = defs = 0
    for c in load(root)["components"]:
        f = port_facts(c, root)
        if f:
            proved, defs = proved + f["proved"], defs + f["defs"]
    return round(proved / defs, 4) if defs else None


B0 = ("C0", "C1", "C2")
B0_RANGE = (3.5, 5.7)  # port map §7.3: the ratios the plan was made with


def b0(root=ROOT):
    """Gate B0 (ADR-0116 decision 2, measures of ADR-0122): the whole port
    estimated from the components ported so far, and the three conditions
    that stop before P3. None while nothing is ported."""
    doc = load(root)
    rows = [(c, sum(stage0_lines(p) for _, p in stage0_files(c, root)), port_facts(c, root)) for c in doc["components"]]
    done = [(c, s0, f) for c, s0, f in rows if f]
    if not done:
        return None
    s0 = sum(s for _, s, _ in done)
    lines = sum(f["lines"] for _, _, f in done)
    total0 = sum(s for _, s, _ in rows)
    whole = lines / s0
    runs = [c["ledger"]["gate_runs"] for c, _, _ in done]
    undecided = [c["ledger"]["undecided_runs"] for c, _, _ in done]
    c2 = next((f for c, _, f in done if c["id"] == "C2"), None)
    out = {
        "ported": [c["id"] for c, _, _ in done],
        "missing": [i for i in B0 if i not in [c["id"] for c, _, _ in done]],
        "stage0_lines": s0,
        "port_lines": lines,
        "ratio_exec": round((lines - sum(f["spec"] for _, _, f in done)) / s0, 2),
        "ratio_whole": round(whole, 2),
        "estimate_lines": round(total0 * whole),
        "estimate_obligations": round(total0 * whole * sum(f["obligations"] for _, _, f in done) / lines),
        "estimate_rlimit": round(total0 * whole * sum(f["rlimit"] for _, _, f in done) / lines),
        "c2_proved_share": round(c2["proved"] / c2["defs"], 4) if c2 and c2["defs"] else None,
        "undecided_share": round(sum(undecided) / sum(runs), 4) if None not in runs + undecided and sum(runs) else None,
    }
    out["stop"] = [why for why, hit in [
        (f"the whole ratio {out['ratio_whole']} is above {B0_RANGE[1]}", whole > B0_RANGE[1]),
        ("less than half of C2's definitions are proved", out["c2_proved_share"] is not None and out["c2_proved_share"] < 0.5),
        ("more than half of the gate runs ended undecided", out["undecided_share"] is not None and out["undecided_share"] > 0.5),
    ] if hit]
    return out


def replace(text, block):
    if BEGIN not in text or END not in text:
        raise SystemExit(f"{LEDGER} has no ledger markers")
    return text[: text.index(BEGIN)] + block + text[text.index(END) + len(END):]


def table(root=ROOT):
    rows = ["| ID | 구성 요소 | 파일(테스트 제외 줄) | 합계 |", "| --- | --- | --- | --- |"]
    total = 0
    for c in load(root)["components"]:
        sizes = [(f[:-3], stage0_lines(p)) for f, p in stage0_files(c, root)]
        n = sum(s for _, s in sizes)
        total += n
        rows.append(f"| {c['id']} | {c['name']} | " + ", ".join(f"`{f}`({s:,})" for f, s in sizes) + f" | {n:,} |")
    rows.append(f"| | 합계 | | {total:,} |")
    return "\n".join(rows)


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    if cmd == "check":
        bad = problems()
        if not bad and replace((ROOT / LEDGER).read_text(encoding="utf-8"), ledger_block()) != (ROOT / LEDGER).read_text(encoding="utf-8"):
            bad.append(f"{LEDGER} is stale: run `scripts/components.py ledger --write`")
        for b in bad:
            print(f"components: {b}", file=sys.stderr)
        return 1 if bad else 0
    if cmd == "table":
        print(table())
        return 0
    if cmd == "ledger":
        block = ledger_block()
        if "--write" in sys.argv:
            p = ROOT / LEDGER
            p.write_text(replace(p.read_text(encoding="utf-8"), block), encoding="utf-8")
        else:
            print(block)
        return 0
    if cmd == "b0":
        print(json.dumps(b0(), ensure_ascii=False, indent=2))
        return 0
    if cmd in ("pin", "repin") and len(sys.argv) == 3:
        print(json.dumps(pin(sys.argv[2], cmd == "repin")))
        return 0
    raise SystemExit(__doc__)


if __name__ == "__main__":
    sys.exit(main())
