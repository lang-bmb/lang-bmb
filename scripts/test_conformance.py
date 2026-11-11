#!/usr/bin/env python3
"""Rules of scripts/conformance.py (5A-02). No compiler needed."""
import json
import sys
import tempfile
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import conformance  # noqa: E402

CHAPTER = "# 01 Lexing\n\nintro\n\n## Tokens\n\ntext\n\n```\n## not a heading\n```\n\n## `///` and | bars\n\ntext\n"


def setup(t):
    root = Path(t)
    (root / "docs" / "reference").mkdir(parents=True)
    (root / "docs" / "reference" / "01-lexing.md").write_text(CHAPTER, encoding="utf-8")
    (root / "docs" / "reference" / "02-items.md").write_text("# 02\n\n## Units\n", encoding="utf-8")
    (root / "docs" / "reference" / "README.md").write_text("## Not a chapter\n", encoding="utf-8")
    (root / "tests" / "conformance" / "01").mkdir(parents=True)
    return root


def put(root, chapter, name, spec, source="// empty\n"):
    d = root / conformance.DIR / chapter
    d.mkdir(parents=True, exist_ok=True)
    (d / f"{name}.bmb").write_text(source, encoding="utf-8")
    (d / f"{name}.json").write_text(spec if isinstance(spec, str) else json.dumps(spec), encoding="utf-8")


def case(**more):
    return {"sections": ["Tokens"], "note": "A rule.", "check": [], **more}


def write_table(root):
    (root / conformance.README).write_text(f"# suite\n\n{conformance.BEGIN}\n{conformance.END}\n", encoding="utf-8")
    text = (root / conformance.README).read_text(encoding="utf-8")
    (root / conformance.README).write_text(conformance.replace(text, conformance.table_block(root)), encoding="utf-8")


def has(problems, part):
    return any(part in p for p in problems)


def spec_rules():
    heads = ["Tokens", "Other"]
    bad = lambda spec: conformance.spec_problems(spec, heads)  # noqa: E731
    assert bad(case()) == []
    assert bad(case(check=[{"code": "E0401", "line": 3}, {"code": "E0401", "line": 3}])) == []
    assert bad(case(verify={"defs": {"a::f": "verified", "a::g": "failed"}, "diags": [{"code": "E0803", "line": 2}]})) == []
    assert bad(case(verify={"diags": []}, run="1\n")) == []
    assert bad(case(run="")) == []
    assert has(bad([]), "not a JSON object")
    assert has(bad(case(extra=1)), "unknown key `extra`")
    assert has(bad(case(sections=[])), "non-empty list")
    assert has(bad(case(sections="Tokens")), "non-empty list")
    assert has(bad(case(sections=["Nope"])), "`Nope` is not a section")
    assert has(bad(case(sections=["Tokens", "Tokens"])), "twice")
    assert has(bad(case(note=" ")), "`note`")
    assert has(bad({"sections": ["Tokens"], "note": "x"}), "`check`")
    assert has(bad(case(check=[{"code": "E0401"}])), "`check`")
    assert has(bad(case(check=[{"code": "E401", "line": 1}])), "`check`")
    assert has(bad(case(check=[{"code": "E0401", "line": 0}])), "`check`")
    assert has(bad(case(check=[{"code": "E0401", "line": True}])), "`check`")
    assert has(bad(case(check=[{"code": "E0401", "line": 1, "col": 2}])), "`check`")
    # verify and run need a program that checks, run one that is proved
    errors = [{"code": "E0401", "line": 1}]
    assert has(bad(case(check=errors, verify={"defs": {}})), "without errors")
    assert has(bad(case(verify={})), "`verify` is an object")
    assert has(bad(case(verify={"other": 1})), "`verify` is an object")
    assert has(bad(case(verify={"defs": {"a::f": "undecided"}})), "`verify.defs`")
    assert has(bad(case(verify={"diags": [{"code": "E0803"}]})), "`verify.diags`")
    assert has(bad(case(check=errors, run="")), "`run` needs")
    assert has(bad(case(verify={"defs": {"a::f": "failed"}}, run="")), "`run` needs")
    assert has(bad(case(verify={"diags": [{"code": "E0803", "line": 2}]}, run="")), "`run` needs")
    assert has(bad(case(run=1)), "`run` is the stdout")
    # a warning in `check` does not rule out `verify`
    assert bad(case(check=[{"code": "W0001", "line": 1}], verify={"defs": {"a::f": "verified"}})) == []


def tree_rules():
    with tempfile.TemporaryDirectory() as t:
        root = setup(t)
        # headings: `## ` lines outside code fences, chapters only
        assert conformance.sections(root) == {"01": ["Tokens", "`///` and | bars"], "02": ["Units"]}

        # every section needs a case or an exemption
        put(root, "01", "tokens_ok", case())
        write_table(root)
        bad = conformance.problems(root)
        assert has(bad, "chapter 01, section ``///` and | bars` has no case")
        assert has(bad, "chapter 02, section `Units` has no case")
        assert not has(bad, "`Tokens`")

        # an incomplete chapter is tolerated by the table: it says what is missing
        assert "| 02 | Units | 0 | 없음 |" in conformance.table_block(root)
        assert "| 01 | `///` and \\| bars | 0 | 없음 |" in conformance.table_block(root)

        put(root, "01", "doc_ok", case(sections=["`///` and | bars", "Tokens"]))
        (root / conformance.EXEMPT).write_text(json.dumps({"02": {"Units": "색인뿐이다"}}), encoding="utf-8")
        assert has(conformance.problems(root), "README.md is stale")
        write_table(root)
        assert conformance.problems(root) == []
        block = conformance.table_block(root)
        assert "| 01 | Tokens | 2 | `doc_ok`, `tokens_ok` |" in block
        assert "| 02 | Units | 0 | 면제: 색인뿐이다 |" in block
        assert "절 3개, 사례 2개, 사례도 면제도 없는 절 0개." in block

        # exemptions: of an existing section, without cases, with a reason
        (root / conformance.EXEMPT).write_text(json.dumps(
            {"01": {"Tokens": "x", "Gone": "y"}, "02": {"Units": " "}}), encoding="utf-8")
        bad = conformance.problems(root)
        assert has(bad, "`Tokens` of chapter 01 has cases")
        assert has(bad, "`Gone` is not a section of chapter 01")
        assert has(bad, "`Units` of chapter 02 has no reason")
        (root / conformance.EXEMPT).write_text(json.dumps({"02": {"Units": "색인뿐이다"}}), encoding="utf-8")
        write_table(root)
        assert conformance.problems(root) == []

        # files: pairs, names, known chapters, nothing else
        d = root / conformance.DIR / "01"
        (d / "lonely.bmb").write_text("")
        (d / "orphan.json").write_text(json.dumps(case()))
        (d / "Bad-Name.bmb").write_text("")
        (d / "notes.txt").write_text("")
        (root / conformance.DIR / "09").mkdir()
        bad = conformance.problems(root)
        assert has(bad, "01/lonely.bmb has no .json")
        assert has(bad, "01/orphan.json has no .bmb")
        assert has(bad, "01/Bad-Name.bmb: a case name is lowercase snake case")
        assert has(bad, "01/notes.txt is neither")
        assert has(bad, "tests/conformance/09 is not a chapter")
        for name in ("lonely.bmb", "orphan.json", "Bad-Name.bmb", "notes.txt"):
            (d / name).unlink()
        (root / conformance.DIR / "09").rmdir()
        assert conformance.problems(root) == []

        # a .json that is not JSON, and one of the wrong form
        put(root, "01", "broken", "{")
        assert has(conformance.problems(root), "01/broken.json: not JSON")
        put(root, "01", "broken", case(sections=["Units"]))
        assert has(conformance.problems(root), "01/broken.json: `Units` is not a section of the chapter")
        (d / "broken.bmb").unlink()
        (d / "broken.json").unlink()

        # the README needs its markers
        (root / conformance.README).write_text("# suite\n", encoding="utf-8")
        assert has(conformance.problems(root), "has no table markers")

        # cases(): complete pairs, optionally of some chapters
        assert [c[:2] for c in conformance.cases(root)] == [("01", "doc_ok"), ("01", "tokens_ok")]
        assert conformance.cases(root, ("02",)) == []


def comparison_rules():
    rows = conformance.rows('{"code":"E0401","line":3,"col":1}\nnot json\n{"def":"a::f","status":"verified"}\n{"code":"E0401","line":3}\n')
    assert conformance.diags(rows) == Counter({("E0401", 3): 2})
    want = conformance.wanted([{"code": "E0401", "line": 3}])
    assert conformance.differ("check", want, Counter({("E0401", 3): 1})) == []
    # a multiset: the same diagnostic twice is not the same as once
    assert conformance.differ("check", want, conformance.diags(rows)) == ["check: missing none; unexpected E0401@3"]
    assert conformance.differ("check", want, Counter({("E0402", 4): 1})) == ["check: missing E0401@3; unexpected E0402@4"]

    # verify: the listed definitions only, and the diagnostics beyond those of check
    class Out:
        stdout = ('{"code":"W0001","line":1}\n{"code":"E0803","line":5}\n'
                  '{"def":"a::f","status":"failed"}\n{"def":"a::g","status":"verified"}\n')
    checked = Counter({("W0001", 1): 1})
    spec = {"defs": {"a::f": "failed"}, "diags": [{"code": "E0803", "line": 5}]}
    assert conformance.verify_diffs(spec, checked, Out) == []
    assert conformance.verify_diffs({"defs": {"a::f": "verified"}, "diags": spec["diags"]}, checked, Out) == [
        "verify: a::f is failed, expected verified"]
    assert conformance.verify_diffs({"defs": {"a::h": "verified"}, "diags": spec["diags"]}, checked, Out) == [
        "verify: a::h is not reported, expected verified"]
    assert conformance.verify_diffs({"defs": {}}, checked, Out) == ["verify: missing none; unexpected E0803@5"]

    # run_case with a stand-in compiler: check decides before verify and run are tried
    real = conformance.call
    try:
        conformance.call = lambda args: type("R", (), {"returncode": 1, "stdout": '{"code":"E0101","line":2}\n', "stderr": ""})
        assert conformance.run_case("bmb", "x.bmb", case(check=[{"code": "E0101", "line": 2}])) == []
        assert conformance.run_case("bmb", "x.bmb", case(run="1\n")) == ["check: missing none; unexpected E0101@2"]
        conformance.call = lambda args: type("R", (), {"returncode": 101, "stdout": "", "stderr": "panicked"})
        assert conformance.run_case("bmb", "x.bmb", case()) == ["check: no verdict (exit 101) panicked"]
    finally:
        conformance.call = real

    # options of `run`
    args = ["--jobs", "8", "01"]
    assert conformance.option(args, "--jobs", "4") == "8" and args == ["01"]
    assert conformance.option(args, "--bmb", "default") == "default"


def main():
    spec_rules()
    tree_rules()
    comparison_rules()
    print("ok")


if __name__ == "__main__":
    main()
