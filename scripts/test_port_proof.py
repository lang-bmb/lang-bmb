#!/usr/bin/env python3
"""Self-test of scripts/port_proof.py: the merge of cache directories."""
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import port_proof


def main() -> int:
    with tempfile.TemporaryDirectory() as d:
        root = Path(d)
        a, b, out = root / "a", root / "b", root / "out"
        a.mkdir()
        b.mkdir()
        (a / "proofs").write_text("bmb proof cache v5\nsha256:1 1 1 0\nsha256:2 2 2 0\n")
        (b / "proofs").write_text("bmb proof cache v5\nsha256:2 2 2 0\nsha256:3 3 3 0\n")
        (a / "defs").write_text("bmb incremental cache v3\nid x\nP a::f 1\n")
        (b / "defs").write_text("bmb incremental cache v3\nid x\nP a::g 2\n")
        port_proof.merge([a, b, root / "missing"], out)
        assert (out / "proofs").read_text() == "bmb proof cache v5\nsha256:1 1 1 0\nsha256:2 2 2 0\nsha256:3 3 3 0\n"
        assert (out / "defs").read_text() == "bmb incremental cache v3\nid x\nP a::f 1\nP a::g 2\n"
        # Caches of another compiler are not mixed.
        (b / "defs").write_text("bmb incremental cache v3\nid y\nP a::g 2\n")
        try:
            port_proof.merge([a, b], out)
        except SystemExit as e:
            assert "another header" in str(e)
        else:
            raise AssertionError("merged caches with different headers")
    print("port_proof: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
