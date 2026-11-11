#!/usr/bin/env python3
"""Rules of scripts/components.py (4D-08, 4D-10, ADR-0113)."""
import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import components  # noqa: E402


def comp(cid, stage0, port=()):
    return {"id": cid, "name": cid, "phase": "P0", "stage0": list(stage0), "port": list(port), "pin": None,
            "ledger": {"session_tokens": None, "rejected_writes": None, "undecided": None, "friction": []}}


def setup(t, comps):
    root = Path(t)
    (root / "compiler" / "src" / "sub").mkdir(parents=True)
    (root / "selfhost").mkdir()
    (root / "dev-docs" / "health").mkdir(parents=True)
    (root / "compiler" / "src" / "a.rs").write_text("fn a() {}\n// note\n#[cfg(test)]\nmod tests {}\n")
    (root / "compiler" / "src" / "sub" / "b.rs").write_text("fn b() {}\n")
    components.save({"components": comps}, root)
    return root


def expect_exit(f):
    try:
        f()
    except SystemExit as e:
        return str(e)
    raise AssertionError("no error")


def main():
    with tempfile.TemporaryDirectory() as t:
        root = setup(t, [comp("C0", ["a.rs"])])
        # a file in a subdirectory that no component lists
        assert any("sub/b.rs is in no component" in p for p in components.problems(root))
        components.save({"components": [comp("C0", ["a.rs"]), comp("C1", ["sub/b.rs", "a.rs"])]}, root)
        assert any("a.rs is in C0 and C1" in p for p in components.problems(root))
        components.save({"components": [comp("C0", ["a.rs", "gone.rs"]), comp("C1", ["sub/b.rs"])]}, root)
        assert any("gone.rs does not exist" in p for p in components.problems(root))
        components.save({"components": [comp("C0", ["a.rs"], ["src/x.bmb"]), comp("C1", ["sub/b.rs"])]}, root)
        assert any("not a file under selfhost/" in p for p in components.problems(root))

        # lines before the tests
        assert components.stage0_lines(root / "compiler" / "src" / "a.rs") == 2

        # the pin: only with a port; a pinned component's stage0 change needs a port change
        (root / "selfhost" / "a.bmb").write_text("mod a {\n}\n")
        components.save({"components": [comp("C0", ["a.rs"], ["selfhost/a.bmb"]), comp("C1", ["sub/b.rs"])]}, root)
        assert components.problems(root) == []
        assert "no port files" in expect_exit(lambda: components.pin("C1", False, root))
        assert "not pinned" in expect_exit(lambda: components.pin("C0", True, root))
        components.pin("C0", False, root)
        assert "already pinned" in expect_exit(lambda: components.pin("C0", False, root))
        assert components.problems(root) == []
        # an unpinned component changes freely
        (root / "compiler" / "src" / "sub" / "b.rs").write_text("fn b() { }\n")
        assert components.problems(root) == []
        # the pinned one does not
        (root / "compiler" / "src" / "a.rs").write_text("fn a() { 1; }\n")
        assert any("C0 is pinned" in p for p in components.problems(root))
        assert "the port did not" in expect_exit(lambda: components.pin("C0", True, root))
        (root / "selfhost" / "a.bmb").write_text("mod a {\n    // changed\n}\n")
        components.pin("C0", True, root)
        assert components.problems(root) == []
        # a port-only change (a proof added) is free
        (root / "selfhost" / "a.bmb").write_text("mod a {\n}\n")
        assert components.problems(root) == []
        # a file renamed inside the component counts as a change
        doc = components.load(root)
        assert json.dumps(doc["components"][0]["pin"]) != json.dumps(components.state(comp("C0", ["sub/b.rs"], ["selfhost/a.bmb"]), root))

    assert components.replace(f"a\n{components.BEGIN}\nold\n{components.END}\nb", "NEW") == "a\nNEW\nb"
    assert "no ledger markers" in expect_exit(lambda: components.replace("none", "x"))
    # the repository's components and ledger are current
    assert components.problems() == []
    print("ok")


if __name__ == "__main__":
    main()
