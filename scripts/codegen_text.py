#!/usr/bin/env python3
"""The code generator's constant IR texts as a BMB unit, for the port.

stage0 holds the module's head and its runtime parts as string constants in
`compiler/src/codegen.rs` (`bmb dump runtime` lists them by name). The port
holds the same bytes in this generated file, `selfhost/codegen_text.bmb`:
one function per text (backlog 4D-05c1).

    codegen_text.py write     regenerate the file (in normal form: `bmb fmt`)
    codegen_text.py check     fail if the file is stale (the gate)
"""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from std_units import lit  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "selfhost" / "codegen_text.bmb"


def raw(bmb: str) -> str:
    r = subprocess.run([bmb, "dump", "runtime"], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"codegen_text: bmb dump runtime failed\n{r.stdout}{r.stderr}")
    rows = [json.loads(line) for line in r.stdout.splitlines()]
    fns = "\n".join(
        f"    #[effects(alloc)]\n    fn {row['name']}() -> String {{ String::from({lit(row['text'])}) }}\n"
        for row in sorted(rows, key=lambda row: row["name"])
    )
    return (
        '#[summary("코드 생성의 상수 IR 글: scripts/codegen_text.py가 stage0의 bmb dump runtime에서 만든다'
        "(손으로 고치지 않는다). stage0 codegen.rs의 문자열 상수의 자리 (백로그 4D-05c1)\")]\n"
        "mod codegen_text {\n" + fns + "}\n"
    )


def generate(bmb: str) -> str:
    with tempfile.TemporaryDirectory() as d:
        f = Path(d) / "codegen_text.bmb"
        f.write_text(raw(bmb), encoding="utf-8")
        r = subprocess.run([bmb, "fmt", str(f)], capture_output=True, text=True)
        if r.returncode != 0:
            sys.exit(f"codegen_text: bmb fmt failed\n{r.stdout}{r.stderr}")
        return r.stdout


def main() -> int:
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    bmb = str(ROOT / "target" / "debug" / "bmb")
    if cmd == "write":
        OUT.write_text(generate(bmb), encoding="utf-8")
        return 0
    if cmd == "check":
        want = generate(bmb)
        have = OUT.read_text(encoding="utf-8") if OUT.exists() else ""
        if want != have:
            print("codegen_text: selfhost/codegen_text.bmb is stale; run scripts/codegen_text.py write")
            return 1
        print("codegen_text: fresh")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
