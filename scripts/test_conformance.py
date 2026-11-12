#!/usr/bin/env python3
"""Rules of scripts/conformance.py (5A-02, 5A-05). No compiler needed."""
import json
import os
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
    warned = [{"code": "W0001", "line": 1}]
    assert bad(case(check=warned, verify={"defs": {"a::f": "verified"}})) == []

    # check_mode: exact or includes, and includes only for a program with an error
    assert bad(case(check_mode="exact")) == []
    assert bad(case(check=errors, check_mode="includes")) == []
    assert has(bad(case(check_mode="at_least")), "`check_mode` is")
    assert has(bad(case(check_mode="includes")), "needs an error")
    assert has(bad(case(check=warned, check_mode="includes")), "needs an error")

    # verify.defs: a status, or an object of status-line fields
    line = lambda **v: case(verify={"defs": {"a::f": v}})  # noqa: E731
    assert bad(line(status="verified", trust="assumed", obligations=3, failed=0)) == []
    assert bad(line(trust=None)) == []
    assert bad(line(trust="external", obligations=0)) == []
    assert has(bad(line()), "`verify.defs`")
    assert has(bad(line(status="undecided")), "`verify.defs`")
    assert has(bad(line(trust="trusted")), "`verify.defs`")
    assert has(bad(line(trust=False)), "`verify.defs`")
    assert has(bad(line(obligations=-1)), "`verify.defs`")
    assert has(bad(line(obligations=True)), "`verify.defs`")
    assert has(bad(line(failed="1")), "`verify.defs`")
    assert has(bad(line(status="verified", rlimit=5)), "`verify.defs`")
    # a failed status inside an object is a failure for `run` and `build`
    assert has(bad({**line(status="failed"), "run": ""}), "`run` needs")
    assert bad({**line(obligations=1), "run": ""}) == []

    # deterministic: distinct commands, verify only for a program that checks
    assert bad(case(deterministic=["check", "verify"])) == []
    assert bad(case(check=errors, deterministic=["check"])) == []
    for v in ([], "check", ["build"], ["check", "check"], [1]):
        assert has(bad(case(deterministic=v)), "`deterministic` is"), v
    assert has(bad(case(check=errors, deterministic=["verify"])), "`deterministic` with")

    # emit: one entry, or two that differ in all_checks
    assert bad(case(emit={"exit": 0})) == []
    assert bad(case(check=warned, emit={"runtime_checks": 2, "all_checks": True})) == []
    assert bad(case(emit=[{"exit": 1, "runtime_checks": 1}, {"all_checks": True, "exit": 0}])) == []
    for v in ({}, [], {"all_checks": True}, {"exit": 2}, {"exit": True}, {"exit": "0"}, {"runtime_checks": -1},
              {"runtime_checks": 1.0}, {"exit": 0, "all_checks": 1}, {"exit": 0, "ir": "x"}, [{"exit": 0}] * 3, "ok"):
        assert has(bad(case(emit=v)), "`emit` is"), v
    assert has(bad(case(emit=[{"exit": 0}, {"exit": 1, "all_checks": False}])), "differ in `all_checks`")
    assert has(bad(case(check=errors, emit={"exit": 1})), "`emit` needs")

    # build: ok for a proved program, refused for one with a failure, never with run
    failing = {"defs": {"a::f": "failed"}}
    assert bad(case(build="ok")) == []
    assert bad(case(verify={"defs": {"a::f": {"trust": "assumed"}}}, build="ok")) == []
    assert bad(case(verify=failing, build="refused")) == []
    assert bad(case(check=warned, verify={"diags": [{"code": "E0811", "line": 2}]}, build="refused")) == []
    assert has(bad(case(build="yes")), "`build` is")
    assert has(bad(case(build=True)), "`build` is")
    assert has(bad(case(build="ok", run="")), "do not go together")
    assert has(bad(case(verify=failing, build="refused", run="")), "do not go together")
    assert has(bad(case(verify=failing, build="ok")), "`build` \"ok\" needs")
    assert has(bad(case(check=warned, build="ok")), "`build` \"ok\" needs")
    assert has(bad(case(build="refused")), "`build` \"refused\" needs")
    assert has(bad(case(verify={"defs": {"a::f": "verified"}}, build="refused")), "`build` \"refused\" needs")
    assert has(bad(case(check=errors, build="refused")), "`build` \"refused\" needs")

    # run: the stdout, or an object with the exit; a trap takes no stdout
    assert bad(case(run={"stdout": "1\n", "exit": "ok"})) == []
    assert bad(case(run={"exit": "trap"})) == []
    for v in ({}, {"stdout": "1\n"}, {"exit": "ok"}, {"exit": "trap", "stdout": "1\n"}, {"exit": 1}, {"exit": "signal"},
              {"stdout": 1, "exit": "ok"}, {"exit": "trap", "signal": 4}, ["1\n"]):
        assert has(bad(case(run=v)), "`run` is the stdout"), v
    assert has(bad(case(verify=failing, run={"exit": "trap"})), "`run` needs")


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
        conformance.call = lambda args: result(1, '{"code":"E0101","line":2}\n')
        assert conformance.run_case("bmb", "x.bmb", case(check=[{"code": "E0101", "line": 2}])) == []
        assert conformance.run_case("bmb", "x.bmb", case(run="1\n")) == ["check: missing none; unexpected E0101@2"]
        conformance.call = lambda args: result(101, "", "panicked")
        assert conformance.run_case("bmb", "x.bmb", case()) == ["check: no verdict (exit 101) panicked"]
    finally:
        conformance.call = real

    # options of `run`
    args = ["--jobs", "8", "01"]
    assert conformance.option(args, "--jobs", "4") == "8" and args == ["01"]
    assert conformance.option(args, "--bmb", "default") == "default"


def result(code, stdout="", stderr=""):
    return type("R", (), {"returncode": code, "stdout": stdout, "stderr": stderr})


def includes_rules():
    want = conformance.wanted([{"code": "E0602", "line": 3}, {"code": "E0607", "line": 6}])
    got = lambda *ds: Counter(ds)  # noqa: E731
    both = (("E0602", 3), ("E0607", 6))
    assert conformance.includes("check", want, got(*both)) == []
    # more diagnostics on a listed line are a cascade, whatever their code and number
    assert conformance.includes("check", want, got(*both, ("E0602", 3), ("E0103", 6), ("W0001", 3))) == []
    # a listed entry must be there, as often as it is listed
    assert conformance.includes("check", want, got(("E0602", 3))) == [
        "check: missing E0607@6; unexpected on other lines none"]
    assert conformance.includes("check", want + want, got(*both, ("E0602", 3))) == [
        "check: missing E0607@6; unexpected on other lines none"]
    # a diagnostic on another line is not a cascade: an error or a warning
    assert conformance.includes("check", want, got(*both, ("E0401", 9))) == [
        "check: missing none; unexpected on other lines E0401@9"]
    assert conformance.includes("check", want, got(*both, ("W0001", 1))) == [
        "check: missing none; unexpected on other lines W0001@1"]

    # run_case picks the comparison by `check_mode`
    real = conformance.call
    try:
        conformance.call = lambda args: result(1, '{"code":"E0602","line":3}\n{"code":"E0602","line":3}\n')
        listed = [{"code": "E0602", "line": 3}]
        assert conformance.run_case("bmb", "x.bmb", case(check=listed, check_mode="includes")) == []
        assert conformance.run_case("bmb", "x.bmb", case(check=listed, check_mode="exact")) == [
            "check: missing none; unexpected E0602@3"]
        assert conformance.run_case("bmb", "x.bmb", case(check=listed)) == ["check: missing none; unexpected E0602@3"]
    finally:
        conformance.call = real


def status_line_rules():
    out = result(0, '{"def":"a::f","status":"verified","obligations":2,"failed":0,"rlimit":7,"trust":"assumed"}\n'
                    '{"def":"a::g","status":"verified","obligations":0,"failed":0,"trust":"external"}\n'
                    '{"def":"a::h","status":"failed","obligations":3,"failed":1}\n')
    diffs = lambda defs: conformance.verify_diffs({"defs": defs}, Counter(), out)  # noqa: E731
    assert diffs({"a::f": {"status": "verified", "trust": "assumed", "obligations": 2, "failed": 0},
                  "a::g": {"trust": "external", "obligations": 0},
                  "a::h": {"status": "failed", "trust": None, "failed": 1}}) == []
    # only the keys given are compared
    assert diffs({"a::f": {"obligations": 2}, "a::h": "failed"}) == []
    # null pins that the status line has no trust
    assert diffs({"a::f": {"trust": None}}) == ['verify: a::f has trust "assumed", expected null']
    assert diffs({"a::h": {"trust": "assumed"}}) == ['verify: a::h has trust null, expected "assumed"']
    assert diffs({"a::g": {"trust": "assumed"}}) == ['verify: a::g has trust "external", expected "assumed"']
    assert diffs({"a::f": {"status": "failed", "obligations": 1, "failed": 1}}) == [
        "verify: a::f is verified, expected failed", "verify: a::f has obligations 2, expected 1",
        "verify: a::f has failed 0, expected 1"]
    assert diffs({"a::x": {"status": "failed"}}) == ["verify: a::x is not reported, expected failed"]
    assert diffs({"a::x": {"obligations": 0}}) == ["verify: a::x is not reported, expected a status line"]


def deterministic_rules():
    real = conformance.call
    asked = []

    def stand_in(outputs):
        runs = iter(outputs)

        def call(args, text=True):
            asked.append((args[1], text))
            code, stdout = next(runs)
            return result(code, stdout)
        return call

    try:
        conformance.call = stand_in([(1, b"a\n"), (1, b"a\n")])
        assert conformance.repeat_diffs("bmb", "verify", "x.bmb") == []
        # the command is run twice and its stdout is read as bytes
        assert asked == [("verify", False), ("verify", False)]
        conformance.call = stand_in([(0, b"a\n"), (0, b"a\r\n")])
        assert conformance.repeat_diffs("bmb", "check", "x.bmb") == ["deterministic: two runs of `check` print different bytes"]
        conformance.call = stand_in([(0, b"a\n"), (1, b"a\n")])
        assert conformance.repeat_diffs("bmb", "check", "x.bmb") == [
            "deterministic: two runs of `check` end with exit codes 0 and 1"]
        conformance.call = stand_in([(0, b""), (-1, b"")])
        assert conformance.repeat_diffs("bmb", "verify", "x.bmb") == ["deterministic: `verify` gave no verdict (exit -1)"]

        # run_case: the first `check` and each listed command twice
        del asked[:]
        conformance.call = stand_in([(0, "")] + [(0, b"")] * 4)
        assert conformance.run_case("bmb", "x.bmb", case(deterministic=["check", "verify"])) == []
        assert [a for a, _ in asked] == ["check", "check", "check", "verify", "verify"]
    finally:
        conformance.call = real


def emit_rules():
    """A stand-in `bmb emit-llvm` writes the IR and the --stats line it is told to."""
    real = conformance.call
    seen = []

    def stand_in(code, stdout, ir="define\n", stats='{"encoded":[],"runtime_checks":2}'):
        def call(args, text=True):
            seen.append([a for a in args[1:-3] if not a.endswith("stats.json")])
            if ir is not None:
                Path(args[-1]).write_text(ir)
            if "--stats" in args:
                Path(args[args.index("--stats") + 1]).write_text(stats)
            return result(code, stdout)
        return call

    failure = '{"code":"W0001","line":1}\n{"code":"E0803","line":4}\n{"def":"a::f","status":"failed"}\n'
    checked = Counter({("W0001", 1): 1})
    verify = {"defs": {"a::f": "failed"}, "diags": [{"code": "E0803", "line": 4}]}
    diffs = lambda want, v=verify: conformance.emit_diffs("bmb", "x.bmb", want, v, checked)  # noqa: E731
    try:
        conformance.call = stand_in(1, failure)
        assert diffs({"exit": 1, "runtime_checks": 2}) == []
        assert diffs({"exit": 1}) == []
        # --stats is asked for only when the case pins the count, --all-checks only when it says so
        assert seen == [["emit-llvm", "--stats"], ["emit-llvm"]]
        assert diffs({"exit": 0, "runtime_checks": 0}) == [
            "emit: exit code 1, expected 0", "emit: runtime_checks is 2, expected 0"]
        # without --all-checks the diagnostics beyond those of check are those of verify.diags, when the case has them
        assert diffs({"exit": 1}, {"defs": {}}) == ["emit: missing none; unexpected E0803@4"]
        assert diffs({"exit": 1}, None) == []

        # --all-checks: no status line, no diagnostic beyond those of check
        del seen[:]
        conformance.call = stand_in(0, '{"code":"W0001","line":1}\n')
        assert diffs({"all_checks": True, "exit": 0, "runtime_checks": 2}) == []
        assert seen == [["emit-llvm", "--all-checks", "--stats"]]
        conformance.call = stand_in(0, failure)
        assert diffs({"all_checks": True, "exit": 0}) == [
            "emit --all-checks: prints status lines", "emit --all-checks: missing none; unexpected E0803@4"]

        # the IR file must be written and not empty; a missing or damaged stats line is a difference
        conformance.call = stand_in(1, failure, ir=None)
        assert diffs({"exit": 1}) == ["emit: no IR written"]
        conformance.call = stand_in(1, failure, ir="")
        assert diffs({"exit": 1}) == ["emit: no IR written"]
        conformance.call = stand_in(1, failure, stats='{"encoded":[]}')
        assert diffs({"runtime_checks": 0}) == ["emit: runtime_checks is None, expected 0"]
        conformance.call = stand_in(1, failure, stats="not json")
        assert diffs({"runtime_checks": 0}) == ["emit: runtime_checks is None, expected 0"]

        # run_case runs every entry of `emit`
        del seen[:]
        conformance.call = stand_in(0, "")
        spec = case(emit=[{"exit": 0, "runtime_checks": 2}, {"all_checks": True, "exit": 1}])
        assert conformance.run_case("bmb", "x.bmb", spec) == ["emit --all-checks: exit code 0, expected 1"]
        assert seen[1:] == [["emit-llvm", "--stats"], ["emit-llvm", "--all-checks"]]
        assert conformance.emits(case()) == [] and conformance.emits(case(emit={"exit": 0})) == [{"exit": 0}]
    finally:
        conformance.call = real


def script(folder, name, body):
    """An executable that stands in for a built program."""
    path = Path(folder) / name
    path.write_text("#!/bin/sh\n" + body, encoding="utf-8")
    os.chmod(path, 0o755)
    return str(path)


def build_and_run_rules():
    real = conformance.call

    def stand_in(code, body=None, extra=None):
        """`bmb build`: writes the program `body` to -o (and a file `extra` beside it) and exits with `code`."""
        def call(args, text=True):
            if args[1] != "build":
                return result(0, "")
            exe = Path(args[-1])
            if body is not None:
                script(exe.parent, exe.name, body)
            if extra:
                (exe.parent / extra).write_text("")
            return result(code, "", "bmb: build refused" if code else "")
        return call

    try:
        # refused: exit code 1 and nothing written
        conformance.call = stand_in(1)
        assert conformance.build_diffs("bmb", "x.bmb", "refused") == []
        conformance.call = stand_in(1, extra="case.ll")
        assert conformance.build_diffs("bmb", "x.bmb", "refused") == ["build: refused, but wrote case.ll"]
        conformance.call = stand_in(0, "exit 0\n")
        assert conformance.build_diffs("bmb", "x.bmb", "refused") == [
            "build: exit code 0, expected 1 (refused)", "build: refused, but wrote case"]
        conformance.call = stand_in(101)
        assert conformance.build_diffs("bmb", "x.bmb", "refused") == ["build: exit code 101, expected 1 (refused)"]

        # ok: exit code 0 and the executable exists; it is not run
        conformance.call = stand_in(0, "exit 7\n")
        assert conformance.build_diffs("bmb", "x.bmb", "ok") == []
        conformance.call = stand_in(0)
        assert conformance.build_diffs("bmb", "x.bmb", "ok") == ["build failed: none"]
        conformance.call = stand_in(1)
        assert conformance.build_diffs("bmb", "x.bmb", "ok") == ["build failed: none bmb: build refused"]

        # run: the stdout of a run that ends with 0, or a trap (any other end) whose stdout is not read
        conformance.call = stand_in(0, "printf '1\\n'\n")
        run = lambda want: conformance.build_diffs("bmb", "x.bmb", "ok", want)  # noqa: E731
        assert run("1\n") == [] and run({"stdout": "1\n", "exit": "ok"}) == []
        assert run("2\n") == ["run: stdout is '1\\n', expected '2\\n'"]
        assert run({"exit": "trap"}) == ["run: exit code 0, expected a trap"]
        conformance.call = stand_in(0, "printf '1\\n'\nexit 3\n")
        assert run({"exit": "trap"}) == []
        assert run("1\n") == ["run: exit code 3"]
        conformance.call = stand_in(0, "printf '1\\n'\nkill -ILL $$\n")
        assert run({"exit": "trap"}) == []
        assert run({"stdout": "1\n", "exit": "ok"}) == ["run: exit code -4"]

        # run_case: `build` and `run` are tried only when nothing else differs
        conformance.call = stand_in(1)
        failing = {"defs": {}, "diags": [{"code": "E0803", "line": 4}]}
        assert conformance.run_case("bmb", "x.bmb", case(verify=failing, build="refused")) == [
            "verify: missing E0803@4; unexpected none"]
        assert conformance.run_case("bmb", "x.bmb", case(build="refused")) == []
        assert conformance.run_case("bmb", "x.bmb", case(build="ok")) == ["build failed: none bmb: build refused"]
    finally:
        conformance.call = real


def main():
    spec_rules()
    tree_rules()
    comparison_rules()
    includes_rules()
    status_line_rules()
    deterministic_rules()
    emit_rules()
    build_and_run_rules()
    print("ok")


if __name__ == "__main__":
    main()
