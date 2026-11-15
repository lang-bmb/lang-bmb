#!/usr/bin/env python3
"""The standard library's source text as a BMB unit, for the port.

stage0 embeds `lib/std/*.bmb` at build time (`compiler/build.rs` writes
`STD_UNITS`, `stdlib.rs` includes it). BMB has no such include, so the
port's table is this generated file, `selfhost/std_units.bmb`: the units
by stem in byte order, each text as one string literal.

    std_units.py write     regenerate the file (in normal form: `bmb fmt`)
    std_units.py check     fail if the file is stale (the gate)
"""
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "selfhost" / "std_units.bmb"


def lit(s: str) -> str:
    out = []
    for ch in s:
        if ch == "\\":
            out.append("\\\\")
        elif ch == '"':
            out.append('\\"')
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\r":
            out.append("\\r")
        elif ch == "\t":
            out.append("\\t")
        else:
            out.append(ch)
    return '"' + "".join(out) + '"'


def raw(root: Path) -> str:
    paths = sorted((root / "lib" / "std").glob("*.bmb"), key=lambda p: p.stem.encode())
    # stage0 orders the units by path; the port by stem. They must agree.
    assert paths == sorted(paths), "std unit order: stem and path order differ"
    rows = [(p.stem, p.read_text(encoding="utf-8")) for p in paths]

    def arms(pick) -> str:
        return "".join(
            f"        if i == {k} {{ return String::from({lit(pick(r))}); }}\n" for k, r in enumerate(rows)
        )

    return (
        '#[summary("표준 라이브러리 단위의 글: scripts/std_units.py가 lib/std/*.bmb에서 만든다(손으로 고치지 않는다). '
        'stage0의 build.rs가 만드는 STD_UNITS의 자리 (ADR-0092 §1, 백로그 4D-03b)")]\n'
        "mod std_units {\n"
        '    #[summary("단위의 수")]\n'
        "    #[effects(pure)]\n"
        f"    fn count() -> usize {{ {len(rows)} }}\n\n"
        '    #[summary("i번째 단위의 이름(이름의 바이트 순서). 범위 밖이면 빈 글")]\n'
        "    #[effects(alloc)]\n"
        "    fn name(i: usize) -> String {\n" + arms(lambda r: r[0]) + "        String::new()\n    }\n\n"
        '    #[summary("i번째 단위의 소스. 범위 밖이면 빈 글")]\n'
        "    #[effects(alloc)]\n"
        "    fn text(i: usize) -> String {\n" + arms(lambda r: r[1]) + "        String::new()\n    }\n"
        "}\n"
    )


def generate(root: Path, bmb: str) -> str:
    with tempfile.TemporaryDirectory() as d:
        f = Path(d) / "std_units.bmb"
        f.write_text(raw(root), encoding="utf-8")
        r = subprocess.run([bmb, "fmt", str(f)], capture_output=True, text=True)
        if r.returncode != 0:
            sys.exit(f"std_units: bmb fmt failed\n{r.stdout}{r.stderr}")
        return r.stdout


def main() -> int:
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    bmb = str(ROOT / "target" / "debug" / "bmb")
    if cmd == "write":
        OUT.write_text(generate(ROOT, bmb), encoding="utf-8")
        return 0
    if cmd == "check":
        want = generate(ROOT, bmb)
        have = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
        if want != have:
            print("std_units: selfhost/std_units.bmb is stale; run scripts/std_units.py write")
            return 1
        print("std_units: fresh")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
