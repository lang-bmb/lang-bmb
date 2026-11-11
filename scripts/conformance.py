#!/usr/bin/env python3
"""The conformance suite: cases that pin the language reference (5A-02).

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
   "verify": {"defs": {"unit::name": "verified" | "failed"},
              "diags": [{"code": "E0805", "line": 9}]},
   "run": "<exact stdout>"}

`check` (required) is the multiset of (code, line) of every diagnostic
`bmb check` prints; `[]` is a program that checks clean. `verify` (only with
a clean `check`) gives the status `bmb verify` reports for the definitions
listed and the multiset of the diagnostics it prints beyond those of
`check`. `run` (only for a program that is proved) is the stdout of the
executable `bmb build` makes. Messages, columns and fix texts are not part
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
KEYS = {"sections", "note", "check", "verify", "run"}
STATUS = ("verified", "failed")
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


def verify_problems(v, clean):
    out = []
    if not isinstance(v, dict) or not set(v) <= {"defs", "diags"} or not v:
        return ["`verify` is an object with `defs` and/or `diags`"]
    if not clean:
        out.append("`verify` needs a `check` without errors")
    defs = v.get("defs", {})
    if not isinstance(defs, dict) or not all(isinstance(k, str) and s in STATUS for k, s in defs.items()):
        out.append("`verify.defs` maps a definition to \"verified\" or \"failed\"")
    if not diag_list(v.get("diags", [])):
        out.append("`verify.diags` is a list of {\"code\", \"line\"}")
    return out


def proved(spec):
    v = spec.get("verify", {})
    return isinstance(v, dict) and not v.get("diags") and "failed" not in (
        v["defs"].values() if isinstance(v.get("defs"), dict) else [])


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
    if "verify" in spec:
        out += verify_problems(spec["verify"], clean)
    if "run" in spec:
        if not isinstance(spec["run"], str):
            out.append("`run` is the stdout as a string")
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


def call(args):
    try:
        return subprocess.run(args, capture_output=True, text=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return subprocess.CompletedProcess(args, -1, "", f"timeout after {TIMEOUT}s")


def verify_diffs(spec, checked, r):
    rs = rows(r.stdout)
    status = {x["def"]: x["status"] for x in rs if "def" in x and "status" in x}
    out = [f"verify: {d} is {status.get(d, 'not reported')}, expected {s}"
           for d, s in sorted(spec.get("defs", {}).items()) if status.get(d) != s]
    return out + differ("verify", wanted(spec.get("diags", [])), diags(rs) - checked)


def executable_diffs(exe, expected):
    """The stdout is compared as bytes: no newline translation."""
    try:
        r = subprocess.run([exe], capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return [f"run: timeout after {TIMEOUT}s"]
    if r.returncode != 0:
        return [f"run: exit code {r.returncode}"]
    got = r.stdout.decode("utf-8", errors="replace")
    return [] if got == expected else [f"run: stdout is {got!r}, expected {expected!r}"]


def run_diffs(bmb, src, expected):
    with tempfile.TemporaryDirectory() as t:
        exe = str(Path(t) / "case")
        r = call([bmb, "build", str(src), "-o", exe])
        if r.returncode != 0 or not Path(exe).is_file():
            return [f"build failed: {show(diags(rows(r.stdout)))} {r.stderr.strip()[:200]}".rstrip()]
        return executable_diffs(exe, expected)


def run_case(bmb, src, spec):
    """What differs between the case and what the compiler does."""
    r = call([bmb, "check", str(src)])
    if r.returncode < 0 or (not r.stdout.strip() and r.returncode != 0):
        return [f"check: no verdict (exit {r.returncode}) {r.stderr.strip()[:200]}".rstrip()]
    checked = diags(rows(r.stdout))
    out = differ("check", wanted(spec["check"]), checked)
    if out:
        return out
    if "verify" in spec:
        out += verify_diffs(spec["verify"], checked, call([bmb, "verify", str(src)]))
    if "run" in spec and not out:
        out += run_diffs(bmb, src, spec["run"])
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
