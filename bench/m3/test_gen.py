#!/usr/bin/env python3
"""Gate test for gen.py (backlog 3A-01): N=100 is deterministic, local
(growing N keeps existing units byte for byte), in normal form, and every
executable definition is proved. Uses $BMB or target/debug/bmb."""

import filecmp
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BMB = os.environ.get("BMB", os.path.join(ROOT, "target", "debug", "bmb"))


def gen(out, n):
    r = subprocess.run([sys.executable, os.path.join(HERE, "gen.py"), out, "--defs", str(n)],
                       check=True, capture_output=True, text=True)
    return json.loads(r.stdout)


def main():
    with tempfile.TemporaryDirectory() as t:
        a, b, c = (os.path.join(t, x) for x in "abc")
        s = gen(a, 100)
        assert s["defs"] == 100 and s["units"] == 5, s
        gen(b, 100)
        gen(c, 200)
        files = sorted(os.listdir(os.path.join(a, "src")))
        assert files == [f"m{u}.bmb" for u in range(5)], files
        for f in files:
            assert filecmp.cmp(os.path.join(a, "src", f), os.path.join(b, "src", f), shallow=False), f"not deterministic: {f}"
            assert filecmp.cmp(os.path.join(a, "src", f), os.path.join(c, "src", f), shallow=False), f"not local: {f}"
        fmt = subprocess.run([BMB, "fmt", "--check", a], capture_output=True, text=True)
        assert fmt.returncode == 0, "not in normal form:\n" + fmt.stdout + fmt.stderr
        v = subprocess.run([BMB, "verify", a], capture_output=True, text=True)
        rows = [json.loads(x) for x in v.stdout.splitlines() if x.startswith('{"def"')]
        bad = [r for r in rows if r["status"] != "verified"]
        assert v.returncode == 0 and not bad and len(rows) == s["exec_fns"], v.stdout[-2000:]
    print("bench/m3 gen: ok")


if __name__ == "__main__":
    main()
