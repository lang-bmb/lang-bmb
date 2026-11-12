#!/usr/bin/env python3
"""The conformance suite: cases that pin the language reference (5A-02, 5A-05).

  conformance.py run [--bmb PATH] [--jobs N] [NN ...]
                                 run every case (or those of the chapters
                                 named); one line per failing case
  conformance.py check           the rules below (no compiler needed)
  conformance.py table [--write] the generated table of tests/conformance/README.md

A case is a pair tests/conformance/NN/name.bmb + name.json, NN the chapter
of docs/reference/NN-*.md. The .json:

  {"sections": ["<heading of a `## ` section of that chapter>", ..],
   "note": "<one sentence: the rule the case pins>",
   "check": [{"code": "E0401", "line": 12}, ..],
   "check_mode": "exact" | "includes",
   "verify": {"defs": {"unit::name": "verified" | "failed" | {"status": ..,
                       "trust": "assumed" | "external" | null,
                       "obligations": n, "failed": k}},
              "diags": [{"code": "E0805", "line": 9}]},
   "deterministic": ["check", "verify"],
   "emit": {"all_checks": false, "exit": 0 | 1, "runtime_checks": n},
   "build": "ok" | "refused",
   "run": "<exact stdout>" | {"stdout": "..", "exit": "ok"} | {"exit": "trap"}}

`check` (required) is the multiset of (code, line) of every diagnostic
`bmb check` prints; `[]` is a program that checks clean. With `check_mode`
"includes" (only with an error in `check`) the entries listed must all be
there and any further diagnostic must be on the line of a listed one.

`verify` (only with a `check` without errors) gives what the status line of
`bmb verify` says for the definitions listed, and the multiset of the
diagnostics it prints beyond those of `check`. A definition maps to its
status or to an object; of an object only the keys given are compared, and
`"trust": null` says that the status line has no `trust`.

`deterministic` lists commands (`verify` only with a `check` without
errors) that are run twice on the case: the two exit codes and the two
stdouts, byte for byte, must be equal. They are compared with each other,
never with a stored text.

`emit` (only with a `check` without errors; one object, or two that differ
in `all_checks`) runs `bmb emit-llvm` (`--all-checks` when `all_checks` is
true), which must write a non-empty IR file. `exit` is its exit code.
`runtime_checks` is the number of that name in the line `--stats FILE`
writes: the runtime checks kept for unproved obligations. With `all_checks`
the stdout has no status line and no diagnostic beyond those of `check`;
without it, in a case with `verify`, the diagnostics beyond those of
`check` are those of `verify.diags`. The IR text is never read.

`build` pins `bmb build` without running the result. "ok" (only for a
program that checks clean and is proved): exit code 0 and the executable
exists. "refused" (only with a `verify` that reports a failure): exit code
1 and nothing is written to the output directory. `run` (only for a program
that checks clean and is proved; not with `build`) builds and runs the
executable. A string, or `"exit": "ok"`, is the exact stdout of a run that
ends with exit code 0. `"exit": "trap"` is a run that does not end with
exit code 0; it takes no `stdout`, since the reference does not say what a
program has written when it traps.

Messages, columns, fix texts, IR text, rlimit values and times are not part
of a case: the suite does not depend on one implementation's wording.

The rules of `check`: both files of every case exist and nothing else is in
a chapter directory; the .json has the form above; every name in `sections`
is a `## ` heading of the chapter; every `## ` section of every chapter has
a case or a reason in tests/conformance/exempt.json
({"NN": {"<heading>": "<reason>"}}); an exemption names an existing section
without cases; the table in the README is current. Standard library only.
"""
import json
import os
import re
import subprocess
import sys
import tempfile
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REF = Path("docs/reference")
DIR = Path("tests/conformance")
README = DIR / "README.md"
EXEMPT = DIR / "exempt.json"
BEGIN = "<!-- conformance:table begin (scripts/conformance.py table --write; do not edit by hand) -->"
END = "<!-- conformance:table end -->"
NAME = re.compile(r"[a-z][a-z0-9_]*\Z")
CODE = re.compile(r"[EW]\d{4}\Z")
KEYS = {"sections", "note", "check", "check_mode", "verify", "deterministic", "emit", "build", "run"}
STATUS = ("verified", "failed")
TRUST = ("assumed", "external", None)
MODES = ("exact", "includes")
REPEATABLE = ("check", "verify")
BUILD = ("ok", "refused")
TIMEOUT = 300


# ---------------------------------------------------------------- the reference


def headings(text):
    """The `## ` headings of a chapter, outside code fences."""
    out, fenced = [], False
    for line in text.splitlines():
        if line.startswith("```"):
            fenced = not fenced
        elif not fenced and line.startswith("## "):
            out.append(line[3:].strip())
    return out


def sections(root=ROOT):
    """{NN: [heading, ..]} over docs/reference/NN-*.md."""
    return {p.name[:2]: headings(p.read_text(encoding="utf-8"))
            for p in sorted((root / REF).glob("[0-9][0-9]-*.md"))}


# ---------------------------------------------------------------- the cases


def chapter_dirs(root=ROOT):
    base = root / DIR
    return sorted(p for p in base.iterdir() if p.is_dir()) if base.is_dir() else []


def cases(root=ROOT, only=()):
    """[(chapter, name, .bmb path, .json path)] of the complete pairs."""
    out = []
    for d in chapter_dirs(root):
        if only and d.name not in only:
            continue
        for j in sorted(d.glob("*.json")):
            if j.with_suffix(".bmb").is_file():
                out.append((d.name, j.stem, j.with_suffix(".bmb"), j))
    return out


def diag_list(v):
    """Whether v is a list of {"code", "line"} entries."""
    return isinstance(v, list) and all(
        isinstance(d, dict) and set(d) == {"code", "line"} and isinstance(d["code"], str) and CODE.match(d["code"])
        and type(d["line"]) is int and d["line"] >= 1 for d in v)


def count(v):
    return type(v) is int and v >= 0


def status_line(v):
    """Whether v says what a status line holds: the status, or an object of the fields to compare."""
    if isinstance(v, str):
        return v in STATUS
    return (isinstance(v, dict) and bool(v) and set(v) <= {"status", "trust", "obligations", "failed"}
            and v.get("status", STATUS[0]) in STATUS and (v.get("trust") is None or v["trust"] in TRUST)
            and count(v.get("obligations", 0)) and count(v.get("failed", 0)))


def status_of(v):
    return v.get("status") if isinstance(v, dict) else v


def verify_problems(v, clean):
    out = []
    if not isinstance(v, dict) or not set(v) <= {"defs", "diags"} or not v:
        return ["`verify` is an object with `defs` and/or `diags`"]
    if not clean:
        out.append("`verify` needs a `check` without errors")
    defs = v.get("defs", {})
    if not isinstance(defs, dict) or not all(isinstance(k, str) and status_line(s) for k, s in defs.items()):
        out.append("`verify.defs` maps a definition to \"verified\", \"failed\" or an object with "
                   "`status`, `trust`, `obligations`, `failed`")
    if not diag_list(v.get("diags", [])):
        out.append("`verify.diags` is a list of {\"code\", \"line\"}")
    return out


def proved(spec):
    v = spec.get("verify", {})
    return isinstance(v, dict) and not v.get("diags") and "failed" not in (
        [status_of(s) for s in v["defs"].values()] if isinstance(v.get("defs"), dict) else [])


def mode_problems(spec, clean):
    if spec["check_mode"] not in MODES:
        return ["`check_mode` is \"exact\" or \"includes\""]
    if spec["check_mode"] == "includes" and clean:
        return ["`check_mode` \"includes\" needs an error in `check`"]
    return []


def deterministic_problems(v, clean):
    if not isinstance(v, list) or not v or not all(c in REPEATABLE for c in v) or len(set(v)) != len(v):
        return ["`deterministic` is a non-empty list of distinct commands out of \"check\", \"verify\""]
    return ["`deterministic` with \"verify\" needs a `check` without errors"] if "verify" in v and not clean else []


def emits(spec):
    """The `emit` entries of a case as a list."""
    v = spec.get("emit", [])
    return v if isinstance(v, list) else [v]


def emit_entry(e):
    return (isinstance(e, dict) and set(e) <= {"all_checks", "exit", "runtime_checks"}
            and ("exit" in e or "runtime_checks" in e) and isinstance(e.get("all_checks", False), bool)
            and count(e.get("exit", 0)) and e.get("exit", 0) <= 1 and count(e.get("runtime_checks", 0)))


def emit_problems(v, clean):
    es = v if isinstance(v, list) else [v]
    if not es or len(es) > 2 or not all(emit_entry(e) for e in es):
        return ["`emit` is an object (or a list of two) with `exit` (0 or 1) and/or `runtime_checks`, "
                "and an optional boolean `all_checks`"]
    out = [] if clean else ["`emit` needs a `check` without errors"]
    if len(es) == 2 and es[0].get("all_checks", False) == es[1].get("all_checks", False):
        out.append("the two entries of `emit` differ in `all_checks`")
    return out


def build_problems(spec):
    if spec["build"] not in BUILD:
        return ["`build` is \"ok\" or \"refused\""]
    if "run" in spec:
        return ["`run` builds the program itself: `build` and `run` do not go together"]
    if spec["build"] == "ok" and (spec["check"] or not proved(spec)):
        return ["`build` \"ok\" needs a program that checks clean and is proved"]
    if spec["build"] == "refused" and ("verify" not in spec or proved(spec)):
        return ["`build` \"refused\" needs a `verify` with a failed definition or a diagnostic"]
    return []


def run_form(v):
    """Whether v is the stdout, {"stdout", "exit": "ok"} or {"exit": "trap"}."""
    if isinstance(v, dict) and set(v) <= {"stdout", "exit"}:
        return isinstance(v.get("stdout"), str) if v.get("exit") == "ok" else v == {"exit": "trap"}
    return isinstance(v, str)


def spec_problems(spec, heads):
    """What is wrong with the .json of a case of a chapter with these headings."""
    if not isinstance(spec, dict):
        return ["not a JSON object"]
    out = [f"unknown key `{k}`" for k in sorted(set(spec) - KEYS)]
    secs = spec.get("sections")
    if not isinstance(secs, list) or not secs or not all(isinstance(s, str) for s in secs):
        out.append("`sections` is a non-empty list of headings")
    else:
        out += [f"`{s}` is not a section of the chapter" for s in secs if s not in heads]
        if len(set(secs)) != len(secs):
            out.append("`sections` names a section twice")
    if not isinstance(spec.get("note"), str) or not spec["note"].strip():
        out.append("`note` is a sentence")
    if not diag_list(spec.get("check")):
        out.append("`check` is a list of {\"code\", \"line\"}")
        return out
    clean = not any(d["code"].startswith("E") for d in spec["check"])
    if "check_mode" in spec:
        out += mode_problems(spec, clean)
    if "verify" in spec:
        out += verify_problems(spec["verify"], clean)
    if "deterministic" in spec:
        out += deterministic_problems(spec["deterministic"], clean)
    if "emit" in spec:
        out += emit_problems(spec["emit"], clean)
    if "build" in spec:
        out += build_problems(spec)
    if "run" in spec:
        if not run_form(spec["run"]):
            out.append("`run` is the stdout as a string, {\"stdout\", \"exit\": \"ok\"} or {\"exit\": \"trap\"}")
        elif spec["check"] or not proved(spec):
            out.append("`run` needs a program that checks clean and is proved")
    return out


def load(path):
    """(spec, None) or (None, why it is not JSON)."""
    try:
        return json.loads(path.read_text(encoding="utf-8")), None
    except (ValueError, OSError) as e:
        return None, str(e)


def file_problems(root=ROOT):
    """Chapter directories hold pairs name.bmb + name.json and nothing else."""
    out, known = [], sections(root)
    for d in chapter_dirs(root):
        if d.name not in known:
            out.append(f"{DIR}/{d.name} is not a chapter of {REF}")
        for p in sorted(d.iterdir()):
            rel = f"{DIR}/{d.name}/{p.name}"
            if p.suffix not in (".bmb", ".json") or not p.is_file():
                out.append(f"{rel} is neither a .bmb nor a .json")
            elif not NAME.match(p.stem):
                out.append(f"{rel}: a case name is lowercase snake case")
            elif not p.with_suffix(".json" if p.suffix == ".bmb" else ".bmb").is_file():
                out.append(f"{rel} has no {'.json' if p.suffix == '.bmb' else '.bmb'}")
    return out


def coverage(root=ROOT):
    """({(chapter, heading): [case names]}, problems of the .json files)."""
    known = sections(root)
    cover = {(ch, h): [] for ch, hs in known.items() for h in hs}
    out = []
    for ch, name, _, j in cases(root):
        spec, why = load(j)
        bad = [f"not JSON ({why})"] if why else spec_problems(spec, known.get(ch, []))
        out += [f"{DIR}/{ch}/{name}.json: {b}" for b in bad]
        if not why and isinstance(spec, dict) and isinstance(spec.get("sections"), list):
            for s in spec["sections"]:
                if (ch, s) in cover and name not in cover[(ch, s)]:
                    cover[(ch, s)].append(name)
    return cover, out


def exemptions(root=ROOT):
    p = root / EXEMPT
    return json.loads(p.read_text(encoding="utf-8")) if p.is_file() else {}


def exempt_problems(cover, exempt):
    out = []
    for ch, entries in sorted(exempt.items()):
        for h, why in sorted(entries.items()):
            if (ch, h) not in cover:
                out.append(f"{EXEMPT}: `{h}` is not a section of chapter {ch}")
            elif cover[(ch, h)]:
                out.append(f"{EXEMPT}: `{h}` of chapter {ch} has cases: drop the exemption")
            elif not isinstance(why, str) or not why.strip():
                out.append(f"{EXEMPT}: `{h}` of chapter {ch} has no reason")
    for (ch, h), names in sorted(cover.items()):
        if not names and h not in exempt.get(ch, {}):
            out.append(f"chapter {ch}, section `{h}` has no case (and no exemption in {EXEMPT})")
    return out


def problems(root=ROOT):
    if not sections(root):
        return [f"no chapters under {REF}"]
    cover, out = coverage(root)
    out = file_problems(root) + out + exempt_problems(cover, exemptions(root))
    readme = root / README
    if not readme.is_file():
        out.append(f"{README} does not exist")
    else:
        text = readme.read_text(encoding="utf-8")
        if BEGIN not in text or END not in text:
            out.append(f"{README} has no table markers")
        elif replace(text, table_block(root)) != text:
            out.append(f"{README} is stale: run `scripts/conformance.py table --write`")
    return out


# ---------------------------------------------------------------- the table


def cell(s):
    return s.replace("|", "\\|")


def table_block(root=ROOT):
    cover, _ = coverage(root)
    exempt = exemptions(root)
    lines = [BEGIN, "", "| 장 | 절 | 사례 수 | 사례 (또는 면제 사유) |", "| --- | --- | --- | --- |"]
    bare = 0
    for (ch, h), names in cover.items():  # in chapter order, then the order of the headings
        why = exempt.get(ch, {}).get(h)
        what = ", ".join(f"`{n}`" for n in names) if names else f"면제: {why}" if why else "없음"
        bare += not names and not why
        lines.append(f"| {ch} | {cell(h)} | {len(names)} | {cell(what)} |")
    lines += ["", f"절 {len(cover)}개, 사례 {len(cases(root))}개, 사례도 면제도 없는 절 {bare}개.", "", END]
    return "\n".join(lines)


def replace(text, block):
    if BEGIN not in text or END not in text:
        raise SystemExit(f"{README} has no table markers")
    return text[: text.index(BEGIN)] + block + text[text.index(END) + len(END):]


# ---------------------------------------------------------------- running


def bmb_bin(root=ROOT):
    return os.environ.get("BMB", str(root / "target" / "debug" / "bmb"))


def rows(stdout):
    out = []
    for line in stdout.splitlines():
        if line.startswith("{"):
            try:
                out.append(json.loads(line))
            except ValueError:
                pass
    return out


def diags(rs):
    """The multiset of (code, line) of the diagnostic rows."""
    return Counter((r["code"], r.get("line")) for r in rs if "code" in r)


def wanted(entries):
    return Counter((d["code"], d["line"]) for d in entries)


def show(c):
    return " ".join(f"{code}@{line}" + (f"x{n}" if n > 1 else "") for (code, line), n in sorted(c.items(), key=str)) or "none"


def differ(what, want, got):
    """[] or one message: what is missing and what is extra."""
    if want == got:
        return []
    return [f"{what}: missing {show(want - got)}; unexpected {show(got - want)}"]


def includes(what, want, got):
    """[] or one message: a listed entry is missing, or a diagnostic is on a line that has no listed one."""
    lines = {line for _, line in want}
    stray = Counter({k: n for k, n in (got - want).items() if k[1] not in lines})
    if not want - got and not stray:
        return []
    return [f"{what}: missing {show(want - got)}; unexpected on other lines {show(stray)}"]


def call(args, text=True):
    try:
        return subprocess.run(args, capture_output=True, text=text, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return subprocess.CompletedProcess(args, -1, "" if text else b"", f"timeout after {TIMEOUT}s")


def line_diffs(name, want, row):
    """What a status line (None: there is none) says differently from the entry of `verify.defs`."""
    want = {"status": want} if isinstance(want, str) else want
    if row is None:
        return [f"verify: {name} is not reported, expected {want.get('status', 'a status line')}"]
    out = []
    if "status" in want and row.get("status") != want["status"]:
        out.append(f"verify: {name} is {row.get('status')}, expected {want['status']}")
    return out + [f"verify: {name} has {k} {json.dumps(row.get(k))}, expected {json.dumps(want[k])}"
                  for k in ("trust", "obligations", "failed") if k in want and row.get(k) != want[k]]


def verify_diffs(spec, checked, r):
    rs = rows(r.stdout)
    lines = {x["def"]: x for x in rs if "def" in x and "status" in x}
    out = [m for d, s in sorted(spec.get("defs", {}).items()) for m in line_diffs(d, s, lines.get(d))]
    return out + differ("verify", wanted(spec.get("diags", [])), diags(rs) - checked)


def repeat_diffs(bmb, cmd, src):
    """Two runs of a command give the same exit code and the same stdout bytes."""
    a, b = (call([bmb, cmd, str(src)], text=False) for _ in range(2))
    if a.returncode < 0 or b.returncode < 0:
        return [f"deterministic: `{cmd}` gave no verdict (exit {min(a.returncode, b.returncode)})"]
    if a.returncode != b.returncode:
        return [f"deterministic: two runs of `{cmd}` end with exit codes {a.returncode} and {b.returncode}"]
    return [] if a.stdout == b.stdout else [f"deterministic: two runs of `{cmd}` print different bytes"]


def kept_checks(path):
    """The `runtime_checks` of a --stats file, or None."""
    spec, _ = load(path)
    return spec.get("runtime_checks") if isinstance(spec, dict) else None


def emit_diffs(bmb, src, want, verify, checked):
    """`bmb emit-llvm` against one entry of `emit`; `verify` is the `verify` of the case or None."""
    flag = ["--all-checks"] if want.get("all_checks") else []
    what = " ".join(["emit"] + flag)
    with tempfile.TemporaryDirectory() as t:
        ir, stats = Path(t) / "case.ll", Path(t) / "stats.json"
        asked = ["--stats", str(stats)] if "runtime_checks" in want else []
        r = call([bmb, "emit-llvm", *flag, *asked, str(src), "-o", str(ir)])
        out = []
        if "exit" in want and r.returncode != want["exit"]:
            out.append(f"{what}: exit code {r.returncode}, expected {want['exit']}")
        if not ir.is_file() or not ir.stat().st_size:
            out.append(f"{what}: no IR written {r.stderr.strip()[:200]}".rstrip())
        if asked and kept_checks(stats) != want["runtime_checks"]:
            out.append(f"{what}: runtime_checks is {kept_checks(stats)}, expected {want['runtime_checks']}")
    rs = rows(r.stdout)
    if flag and any("def" in x for x in rs):
        out.append(f"{what}: prints status lines")
    if flag or verify is not None:
        out += differ(what, wanted([] if flag else verify.get("diags", [])), diags(rs) - checked)
    return out


def refusal_diffs(r, folder):
    out = [] if r.returncode == 1 else [f"build: exit code {r.returncode}, expected 1 (refused)"]
    return out + [f"build: refused, but wrote {p.name}" for p in sorted(folder.iterdir())]


def executable_diffs(exe, want):
    """`want` is a `run`. The stdout is compared as bytes: no newline translation."""
    expected, trap = (want, False) if isinstance(want, str) else (want.get("stdout"), want["exit"] == "trap")
    try:
        r = subprocess.run([exe], capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return [f"run: timeout after {TIMEOUT}s"]
    if trap:
        return [] if r.returncode != 0 else ["run: exit code 0, expected a trap"]
    if r.returncode != 0:
        return [f"run: exit code {r.returncode}"]
    got = r.stdout.decode("utf-8", errors="replace")
    return [] if got == expected else [f"run: stdout is {got!r}, expected {expected!r}"]


def build_diffs(bmb, src, build, run=None):
    """`bmb build` into an empty directory: refused, or made (and then run when `run` is given)."""
    with tempfile.TemporaryDirectory() as t:
        exe = Path(t) / "case"
        r = call([bmb, "build", str(src), "-o", str(exe)])
        if build == "refused":
            return refusal_diffs(r, Path(t))
        if r.returncode != 0 or not exe.is_file():
            return [f"build failed: {show(diags(rows(r.stdout)))} {r.stderr.strip()[:200]}".rstrip()]
        return [] if run is None else executable_diffs(str(exe), run)


def run_case(bmb, src, spec):
    """What differs between the case and what the compiler does."""
    r = call([bmb, "check", str(src)])
    if r.returncode < 0 or (not r.stdout.strip() and r.returncode != 0):
        return [f"check: no verdict (exit {r.returncode}) {r.stderr.strip()[:200]}".rstrip()]
    checked = diags(rows(r.stdout))
    compare = includes if spec.get("check_mode") == "includes" else differ
    out = compare("check", wanted(spec["check"]), checked)
    if out:
        return out
    if "verify" in spec:
        out += verify_diffs(spec["verify"], checked, call([bmb, "verify", str(src)]))
    for cmd in spec.get("deterministic", []):
        out += repeat_diffs(bmb, cmd, src)
    for e in emits(spec):
        out += emit_diffs(bmb, src, e, spec.get("verify"), checked)
    if ("build" in spec or "run" in spec) and not out:
        out += build_diffs(bmb, src, spec.get("build", "ok"), spec.get("run"))
    return out


def run_all(bmb, only=(), jobs=4, root=ROOT):
    """[(chapter/name, [differences])] in case order."""
    known = sections(root)

    def one(c):
        ch, name, src, j = c
        spec, why = load(j)
        bad = [f"not JSON ({why})"] if why else spec_problems(spec, known.get(ch, []))
        return f"{ch}/{name}", bad or run_case(bmb, src, spec)

    with ThreadPoolExecutor(max_workers=max(1, jobs)) as pool:
        return list(pool.map(one, cases(root, only)))


def option(args, name, default):
    if name in args:
        i = args.index(name)
        if i + 1 >= len(args):
            raise SystemExit(__doc__)
        value = args[i + 1]
        del args[i:i + 2]
        return value
    return default


def main():
    args = sys.argv[1:]
    cmd = args.pop(0) if args else "check"
    if cmd == "check" and not args:
        bad = problems()
        for b in bad:
            print(f"conformance: {b}", file=sys.stderr)
        return 1 if bad else 0
    if cmd == "table" and args in ([], ["--write"]):
        block = table_block()
        if args:
            p = ROOT / README
            p.write_text(replace(p.read_text(encoding="utf-8"), block), encoding="utf-8")
        else:
            print(block)
        return 0
    if cmd == "run":
        bmb = option(args, "--bmb", bmb_bin())
        jobs = int(option(args, "--jobs", "4"))
        if any(not re.match(r"\d\d\Z", a) for a in args):
            raise SystemExit(__doc__)
        results = run_all(bmb, tuple(args), jobs)
        failed = [(n, d) for n, d in results if d]
        for n, d in failed:
            print(f"conformance: {n}: {'; '.join(d)}")
        print(f"conformance: {len(results)} cases, {len(failed)} failed")
        return 1 if failed else 0
    raise SystemExit(__doc__)


if __name__ == "__main__":
    sys.exit(main())
