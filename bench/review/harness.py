#!/usr/bin/env python3
"""Review harness (ADR-0078 H-REVIEW, backlog T1-44): a reviewer is shown the
intent and one view of a change and says whether the change is defective.

  harness.py [--set DIR] manifest [--write]
  harness.py [--set DIR] sizes                         bytes and tokens each arm shows (no model)
  harness.py [--set DIR] verdicts                      what `bmb weaken` says of each change, by label
  harness.py [--set DIR] view --arm ARM --change ID    the message a reviewer gets
  harness.py [--set DIR] run --arm ARM --agent KIND --out FILE [--changes ID,..] [--jobs N]
  harness.py [--set DIR] table [--d PP] FILE..         rates per arm and the hypothesis verdict

A set is a directory with `set.json` (arms as lists of view parts, the two
arms the hypothesis compares, the token ratio, extra frozen files),
`prompt.md` (the reviewer's system prompt), `changes/` and `MANIFEST`. What
the reviewer is shown is `views.py`; how an answer is judged is `judge.py`.
The default set is this directory.

Agents (`--agent`), one request per change, no tools, no retry (ADR-0021 §3):

  oracle            answers from the hidden label (every review right)
  blind             always answers "ok" (detects nothing, raises no alarm)
  openai:MODEL      a real reviewer (bench/m1/model_api.py, ADR-0066)
  anthropic:MODEL

A results file is JSONL: a header (`header: true`: set, arm, agent, commit,
manifest, change ids), one row per review as it finishes, and a summary row
last. Started again on a file whose header matches, a run skips the changes
that have a row (a change with a row is never reviewed again). Needs `bmb`
($BMB or target/debug/bmb) for the `weaken` part and nothing else.
"""
import argparse
import concurrent.futures
import hashlib
import json
import os
import subprocess
import sys
import threading

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.dont_write_bytecode = True
sys.path.insert(0, HERE)
import judge  # noqa: E402
import views  # noqa: E402

SET = HERE
SCRIPTED = ("oracle", "blind")
# ADR-0077 §4: the minimum difference of a comparison measured once, for a set
# without repeated runs of its own (ADR-0086 §2), in percentage points.
GLOBAL_D = 30


def bmb_bin():
    return os.environ.get("BMB", os.path.join(ROOT, "target", "debug", "bmb"))


def run_commit():
    """HEAD, with `+dirty` when sources differ from it (result files do not
    count), as the other harnesses record it."""
    commit = subprocess.run(["git", "rev-parse", "--short=8", "HEAD"], cwd=ROOT, capture_output=True,
                            text=True).stdout.strip()
    dirty = subprocess.run(["git", "status", "--porcelain", "--", ".", ":!bench/*/results/*"], cwd=ROOT,
                           capture_output=True, text=True).stdout.strip()
    return commit + ("+dirty" if dirty else "")


# ---------- frozen inputs ----------

def set_name():
    return os.path.basename(os.path.normpath(SET))


def frozen_files():
    cfg = views.load_set(SET)
    files = [os.path.join(SET, "prompt.md"), os.path.join(SET, "set.json")]
    files += [os.path.join(SET, f) for f in cfg.get("frozen", [])]
    d = os.path.join(SET, "changes")
    return files + sorted(os.path.join(d, f) for f in os.listdir(d))


def manifest_text():
    lines = []
    for f in frozen_files():
        with open(f, "rb") as fh:
            lines.append(f"{hashlib.sha256(fh.read()).hexdigest()}  {os.path.relpath(f, SET)}")
    return "\n".join(lines) + "\n"


def manifest_digest():
    with open(os.path.join(SET, "MANIFEST"), "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def check_manifest():
    path = os.path.join(SET, "MANIFEST")
    text = None
    if os.path.exists(path):
        with open(path) as f:
            text = f.read()
    if text != manifest_text():
        sys.exit("frozen inputs differ from MANIFEST: refusing to run (ADR-0021 §1)")


# ---------- reviewers ----------

def answer_json(verdict, reason):
    return json.dumps({"verdict": verdict, "reason": reason})


def model_reviewer(kind, system_prompt, message, make=None):
    """One request to a real model, no tools: (answer text, record). A model
    that asks for a tool anyway gets none; its text so far is its answer.
    `make` is `model_api.make_model_agent` unless a test passes its own."""
    if make is None:
        sys.path.append(os.path.join(ROOT, "bench", "m1"))
        from model_api import make_model_agent as make  # noqa: E402
    agent = make(kind, system_prompt, message)
    if agent is None:
        sys.exit(f"unknown agent {kind}")
    agent.set_tools([])
    agent.next_calls([])
    text = "".join(r.get("text", "") for r in agent.requests)
    return text, {"stop": agent.stop, "model_usage": agent.usage, "agent_settings": agent.settings}


def review(change, arm, agent, cfg, system_prompt, make=None):
    """One review row: the view, the reviewer's answer, the judgement."""
    view = views.view(SET, cfg, change, arm, bmb_bin())
    message = views.message(change, view)
    vb, vt = views.size(view)
    mb, mt = views.size(message)
    extra = {}
    if agent == "oracle":
        text = answer_json("defect" if change["label"] == "defect" else "ok", "scripted: the hidden label")
    elif agent == "blind":
        text = answer_json("ok", "scripted: always ok")
    else:
        text, extra = model_reviewer(agent, system_prompt, message, make)
    verdict, reason, malformed = judge.parse(text)
    return {"id": change["id"], "arm": arm, "label": change["label"], "kind": change.get("kind"),
            "verdict": verdict, "reason": reason, "malformed": malformed,
            "outcome": judge.outcome(change["label"], verdict), "answer": text,
            "view_bytes": vb, "view_tokens": vt, "message_bytes": mb, "message_tokens": mt,
            "view_sha256": hashlib.sha256(view.encode()).hexdigest(), **extra}


# ---------- results ----------

def read(path):
    """The JSON lines of a results file; a last line cut off mid-write is dropped."""
    with open(path) as f:
        lines = [line for line in f.read().split("\n") if line.strip()]
    out = []
    for i, line in enumerate(lines):
        try:
            out.append(json.loads(line))
        except json.JSONDecodeError:
            if i != len(lines) - 1:
                raise
    return out


class Log:
    """Rows written as reviews finish; a matching unfinished file is resumed."""

    def __init__(self, path, header):
        self.path, self.header, self.rows, self.lock = path, header, {}, threading.Lock()
        if not path:
            return
        if os.path.exists(path) and os.path.getsize(path):
            lines = read(path)
            if any(r.get("summary") for r in lines):
                sys.exit(f"{path}: a finished run (a frozen set is run once, ADR-0021)")
            if not lines or lines[0] != header:
                sys.exit(f"{path}: not an unfinished run of {json.dumps(header, ensure_ascii=False)}")
            self.rows = {r["id"]: r for r in lines[1:]}
            self._write([header] + list(self.rows.values()))
        else:
            self._write([header])

    def _write(self, objs):
        with open(self.path, "w") as f:
            for o in objs:
                f.write(json.dumps(o, ensure_ascii=False) + "\n")

    def add(self, row):
        with self.lock:
            self.rows[row["id"]] = row
            if self.path:
                with open(self.path, "a") as f:
                    f.write(json.dumps(row, ensure_ascii=False) + "\n")
                    f.flush()
                    if os.path.isfile(self.path):
                        os.fsync(f.fileno())

    def finish(self, ids):
        rows = [self.rows[i] for i in ids]
        summary = {"summary": True, **{k: v for k, v in self.header.items() if k not in ("header", "changes")},
                   **judge.summary(rows)}
        if self.path:
            self._write([self.header] + rows + [summary])
        return rows, summary


def run(arm, agent, out=None, only=None, jobs=1, make=None):
    cfg = views.load_set(SET)
    if arm not in cfg["arms"]:
        sys.exit(f"unknown arm {arm} (the set has {', '.join(cfg['arms'])})")
    changes = views.load_changes(SET)
    if only:
        changes = [c for c in changes if c["id"] in only]
    with open(os.path.join(SET, "prompt.md")) as f:
        system_prompt = f.read()
    header = {"header": True, "task_set": set_name(), "hypothesis": cfg.get("hypothesis"), "arm": arm,
              "agent": agent, "commit": run_commit(), "manifest": manifest_digest(),
              "changes": [c["id"] for c in changes]}
    log = Log(out, header)
    todo = [c for c in changes if c["id"] not in log.rows]

    def one(change):
        log.add(review(change, arm, agent, cfg, system_prompt, make))

    if jobs > 1:
        with concurrent.futures.ThreadPoolExecutor(jobs) as pool:
            list(pool.map(one, todo))
    else:
        for c in todo:
            one(c)
    return log.finish(header["changes"])


# ---------- table ----------

def load_run(path):
    lines = read(path)
    head = lines[0] if lines and lines[0].get("header") else None
    summary = next((r for r in lines if r.get("summary")), None)
    if head is None or summary is None:
        sys.exit(f"{path}: not a finished review run")
    return head, [r for r in lines if "id" in r], summary


def not_judged(runs, cfg):
    """Why the hypothesis verdict is not read from these runs, or None. It is
    read only from the two compared arms of one real-model run: one results
    file per arm, the same set, manifest, model, commit and changes, the
    whole set as frozen, no uncommitted sources."""
    cmp = cfg["compare"]
    by_arm = {}
    for head, _, _ in runs:
        if head["arm"] in by_arm:
            return f"more than one run of arm `{head['arm']}`"
        by_arm[head["arm"]] = head
    for arm in (cmp["candidate"], cmp["baseline"]):
        if arm not in by_arm:
            return f"no run of arm `{arm}`"
    a, b = by_arm[cmp["candidate"]], by_arm[cmp["baseline"]]
    for key, what in (("task_set", "sets"), ("manifest", "frozen inputs"), ("agent", "agents"),
                      ("commit", "compiler commits"), ("changes", "changes")):
        if a[key] != b[key]:
            return f"the two arms have different {what}"
    if a["agent"] in SCRIPTED:
        return f"scripted agent `{a['agent']}`, not a model"
    if a["commit"].endswith("+dirty"):
        return "uncommitted sources"
    if a["manifest"] != manifest_digest() or a["task_set"] != set_name():
        return "the runs are not of this set as frozen"
    if a["changes"] != [c["id"] for c in views.load_changes(SET)]:
        return "not the whole set"
    return None


def fmt_rate(k, n):
    return f"{k}/{n} ({100 * k / n:.1f}%)" if n else "—"


def table(paths, d=GLOBAL_D):
    cfg = views.load_set(SET)
    runs = [load_run(p) for p in paths]
    out = ["| 팔 | 에이전트 | 커밋 | 탐지율 | 오탐율 | 형식 오류 | 평균 토큰(보기) | 평균 토큰(의도 포함) |",
           "| --- | --- | --- | --- | --- | --- | --- | --- |"]
    sums = {}
    for head, rows, _ in runs:
        s = judge.summary(rows)
        sums[head["arm"]] = s
        out.append(f"| {head['arm']} | {head['agent']} | {head['commit']} | "
                   f"{fmt_rate(s['detected'], s['defects'])} | {fmt_rate(s['false_alarms'], s['cleans'])} | "
                   f"{s['malformed']}/{s['reviews']} | {s['mean_view_tokens']:.1f} | {s['mean_message_tokens']:.1f} |")
    kinds = sorted({r.get("kind") for _, rows, _ in runs for r in rows if r.get("kind")})
    if len(kinds) > 1:
        out += ["", "| 종류 | 라벨 | " + " | ".join(f"{h['arm']} ({h['agent']})" for h, _, _ in runs) + " |",
                "| --- | --- |" + " --- |" * len(runs)]
        for kind in kinds:
            cells, label = [], None
            for _, rows, _ in runs:
                rs = [r for r in rows if r.get("kind") == kind]
                label = rs[0]["label"] if rs else label
                hit = sum(1 for r in rs if r["outcome"] in ("detected", "false_alarm"))
                cells.append(fmt_rate(hit, len(rs)))
            out.append(f"| {kind} | {label} (`defect` 답) | " + " | ".join(cells) + " |")
    cmp = cfg["compare"]
    name = cfg.get("hypothesis", "hypothesis")
    why = not_judged(runs, cfg)
    c = None
    if cmp["candidate"] in sums and cmp["baseline"] in sums:
        c = judge.compare(sums[cmp["candidate"]], sums[cmp["baseline"]], d, cmp["token_ratio"])
    out.append("")
    if c:
        out.append(f"{cmp['candidate']} 대 {cmp['baseline']}: 탐지율 차이 {c['detection_diff_pp']:+.1f}%p (D = {d:g}%p), "
                   f"토큰 비 {c['token_ratio']:.3f} (기준 ≤ {cmp['token_ratio'][0]}/{cmp['token_ratio'][1]}; "
                   f"의도 포함 {c['message_token_ratio']:.3f}), 규칙의 값: {c['verdict']}")
    if why or not c:
        out.append(f"{name} 판정: not judged ({why or 'no rates to compare'})")
    else:
        word = {"confirmed": "확정 조건 충족", "refuted": "반증", "neither": "확정 아님(탐지율은 D 안, 토큰 비가 기준 초과)"}
        out.append(f"{name} 판정({runs[0][0]['agent']}): {word[c['verdict']]}")
    return "\n".join(out) + "\n"


def sizes():
    """Mean bytes and tokens of each arm's view and of each part, over the
    set and per kind of change, and the candidate/baseline token ratio with
    and without the intent sentence."""
    cfg = views.load_set(SET)
    changes = views.load_changes(SET)
    cmp = cfg["compare"]
    kinds = [None] + sorted({c.get("kind") for c in changes if c.get("kind")})
    data = {}  # arm -> change id -> {part or "view" or "message": (bytes, tokens)}
    for arm, parts in cfg["arms"].items():
        data[arm] = {}
        for ch in changes:
            texts = [views.part(p, SET, ch, bmb_bin()) for p in parts]
            view = views.join(cfg, parts, texts)
            row = {p: views.size(t) for p, t in zip(parts, texts)}
            row["view"], row["message"] = views.size(view), views.size(views.message(ch, view))
            data[arm][ch["id"]] = row

    def mean(arm, ids, key, i):
        return sum(data[arm][c][key][i] for c in ids) / len(ids)

    out = ["| 종류 | 변경 | 팔 | 부분 | 평균 바이트 | 평균 토큰 |", "| --- | --- | --- | --- | --- | --- |"]
    ratios = ["", "| 종류 | 변경 | 토큰 비(보기) | 토큰 비(의도 포함) |", "| --- | --- | --- | --- |"]
    for kind in kinds:
        ids = [c["id"] for c in changes if kind is None or c.get("kind") == kind]
        name = kind or "(전체)"
        for arm, parts in cfg["arms"].items():
            for key in list(parts) + ["view", "message"]:
                label = {"view": "(보기 전체)", "message": "(의도 포함)"}.get(key, key)
                out.append(f"| {name} | {len(ids)} | {arm} | {label} | {mean(arm, ids, key, 0):.1f} | "
                           f"{mean(arm, ids, key, 1):.1f} |")
        a, b = cmp["candidate"], cmp["baseline"]
        ratios.append(f"| {name} | {len(ids)} | {mean(a, ids, 'view', 1) / mean(b, ids, 'view', 1):.3f} | "
                      f"{mean(a, ids, 'message', 1) / mean(b, ids, 'message', 1):.3f} |")
    ratios += ["", f"토큰 비는 {cmp['candidate']}/{cmp['baseline']}, 기준 ≤ {cmp['token_ratio'][0]}/{cmp['token_ratio'][1]}"]
    return "\n".join(out + ratios) + "\n"


ORDER = ("weakened", "undecided", "signature", "removed", "added", "refined", "compatible", "same")


def weaken_verdict(text):
    """The verdict of `bmb weaken` on a change: of its rows, the first in
    `ORDER` (a change is weakened when any definition is)."""
    found = {json.loads(line).get("verdict") for line in text.split("\n") if line.startswith('{"def"')}
    return next(v for v in ORDER if v in found)


def verdicts():
    """{(kind, label): {weaken verdict: changes}} over the set."""
    out = {}
    for ch in views.load_changes(SET):
        v = weaken_verdict(views.weaken(SET, ch, bmb_bin()))
        row = out.setdefault((ch.get("kind"), ch["label"]), {})
        row[v] = row.get(v, 0) + 1
    return out


def main(argv=None):
    global SET
    ap = argparse.ArgumentParser()
    ap.add_argument("--set", default=HERE, help="review set directory (default: bench/review)")
    sub = ap.add_subparsers(dest="cmd", required=True)
    mf = sub.add_parser("manifest")
    mf.add_argument("--write", action="store_true")
    sub.add_parser("sizes")
    sub.add_parser("verdicts")
    vw = sub.add_parser("view")
    vw.add_argument("--arm", required=True)
    vw.add_argument("--change", required=True)
    rn = sub.add_parser("run")
    rn.add_argument("--arm", required=True)
    rn.add_argument("--agent", required=True)
    rn.add_argument("--out")
    rn.add_argument("--changes")
    rn.add_argument("--jobs", type=int, default=1)
    rn.add_argument("--expect", choices=["all-right", "none-detected"])
    tb = sub.add_parser("table")
    tb.add_argument("--d", type=float, default=GLOBAL_D, help="minimum difference in percentage points (ADR-0077 §4)")
    tb.add_argument("results", nargs="+")
    args = ap.parse_args(argv)
    SET = os.path.abspath(args.set)

    if args.cmd == "manifest":
        if args.write:
            with open(os.path.join(SET, "MANIFEST"), "w") as f:
                f.write(manifest_text())
        else:
            check_manifest()
            print("MANIFEST ok")
        return
    check_manifest()
    if args.cmd == "sizes":
        sys.stdout.write(sizes())
    elif args.cmd == "verdicts":
        for (kind, label), counts in sorted(verdicts().items()):
            print(kind, label, json.dumps(dict(sorted(counts.items()))))
    elif args.cmd == "view":
        cfg = views.load_set(SET)
        change = next((c for c in views.load_changes(SET) if c["id"] == args.change), None)
        if change is None or args.arm not in cfg["arms"]:
            sys.exit("unknown change or arm")
        sys.stdout.write(views.message(change, views.view(SET, cfg, change, args.arm, bmb_bin())))
    elif args.cmd == "run":
        if args.agent not in SCRIPTED:
            sys.path.append(os.path.join(ROOT, "bench", "m1"))
            from model_api import is_model_kind  # noqa: E402
            if not is_model_kind(args.agent):
                sys.exit(f"unknown agent {args.agent}")
        rows, summary = run(args.arm, args.agent, args.out, set(args.changes.split(",")) if args.changes else None,
                            args.jobs)
        print(json.dumps(summary, ensure_ascii=False))
        if args.expect == "all-right" and (summary["detected"] != summary["defects"] or summary["false_alarms"]
                                           or summary["malformed"]):
            sys.exit("expected every review right")
        if args.expect == "none-detected" and (summary["detected"] or summary["false_alarms"]):
            sys.exit("expected no detection and no false alarm")
    else:
        sys.stdout.write(table(args.results, args.d))


if __name__ == "__main__":
    main()
