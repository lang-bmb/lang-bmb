#!/usr/bin/env python3
"""The language reference's links to the diagnostics and the corpus (5A-01).

  reference.py check            the rules below, and the index is current
  reference.py index [--write]  the generated code index of docs/reference/README.md

docs/reference/NN-name.md are the chapters. The rules:

  - every diagnostic code of the language (docs/diagnostics.md, without the
    retired ones and the lint-only `W` codes) is named in a chapter;
  - a chapter names no code that does not exist or is retired;
  - every repository path a chapter names in backticks (`tests/..`, `lib/..`,
    `bench/..`) exists;
  - every `##` section of a chapter names a code or a corpus path.

Standard library only.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REF = Path("docs/reference")
BEGIN = "<!-- reference:index begin (scripts/reference.py index --write; do not edit by hand) -->"
END = "<!-- reference:index end -->"
CODE = re.compile(r"(?<![A-Za-z0-9_])([EW]\d{4})(?!\d)")
PATH = re.compile(r"`((?:tests|lib|bench)/[A-Za-z0-9_./-]+)`")


def codes(root=ROOT):
    """(live codes: {code: meaning}, retired codes) of docs/diagnostics.md."""
    text = (root / "docs" / "diagnostics.md").read_text(encoding="utf-8")
    live, retired, gone = {}, set(), False
    for line in text.splitlines():
        if line.startswith("## "):
            gone = "은퇴" in line
        m = re.match(r"\| ([EW]\d{4}) \| (.*) \|$", line)
        if m:
            if gone:
                retired.add(m.group(1))
            else:
                live[m.group(1)] = m.group(2)
    return live, retired


def chapters(root=ROOT):
    return sorted((root / REF).glob("[0-9][0-9]-*.md"))


def language(code):
    """`W` codes come from `bmb lint` and `bmb weaken`: tools, not the language."""
    return code.startswith("E")


def where(root=ROOT):
    """{code: [chapter file names]} over the chapters."""
    out = {}
    for p in chapters(root):
        for c in sorted(set(CODE.findall(p.read_text(encoding="utf-8")))):
            out.setdefault(c, []).append(p.name)
    return out


def problems(root=ROOT):
    live, retired = codes(root)
    out = []
    named = where(root)
    if not chapters(root):
        return [f"no chapters under {REF}"]
    for c in sorted(live):
        if language(c) and c not in named:
            out.append(f"{c} ({live[c]}) is in no chapter")
    for c, files in sorted(named.items()):
        if c in retired:
            out.append(f"{files[0]} names the retired code {c}")
        elif c not in live:
            out.append(f"{files[0]} names {c}, which is not a diagnostic code")
    for p in chapters(root):
        text = p.read_text(encoding="utf-8")
        for path in sorted(set(PATH.findall(text))):
            if not (root / path.rstrip("/.")).exists():
                out.append(f"{p.name} names `{path}`, which does not exist")
        for sec in re.split(r"^## ", text, flags=re.M)[1:]:
            title = sec.split("\n", 1)[0].strip()
            if not CODE.search(sec) and not PATH.search(sec):
                out.append(f"{p.name} section \"{title}\" names no diagnostic code and no corpus path")
    return out


def index_block(root=ROOT):
    live, _ = codes(root)
    named = where(root)
    lines = [BEGIN, "", "| 코드 | 뜻 | 장 |", "| --- | --- | --- |"]
    for c in sorted(live):
        if language(c):
            files = ", ".join(f"[{f[:2]}]({f})" for f in named.get(c, [])) or "—"
            lines.append(f"| {c} | {live[c]} | {files} |")
    return "\n".join(lines + ["", END])


def replace(text, block):
    if BEGIN not in text or END not in text:
        raise SystemExit(f"{REF}/README.md has no reference:index markers")
    return text[: text.index(BEGIN)] + block + text[text.index(END) + len(END):]


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    readme = ROOT / REF / "README.md"
    if cmd == "check":
        bad = problems()
        if replace(readme.read_text(encoding="utf-8"), index_block()) != readme.read_text(encoding="utf-8"):
            bad.append(f"{REF}/README.md index is stale: run `scripts/reference.py index --write`")
        for b in bad:
            print(f"reference: {b}", file=sys.stderr)
        return 1 if bad else 0
    if cmd == "index":
        if "--write" in sys.argv:
            readme.write_text(replace(readme.read_text(encoding="utf-8"), index_block()), encoding="utf-8")
        else:
            print(index_block())
        return 0
    raise SystemExit(__doc__)


if __name__ == "__main__":
    sys.exit(main())
