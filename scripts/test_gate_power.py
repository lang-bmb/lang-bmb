#!/usr/bin/env python3
"""Which gate-power runs count (backlog 1B-59): a mutant is caught by a job
of the gate, not by a step before the jobs and not by the time limit."""
import contextlib
import io
import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gate_power as g  # noqa: E402

JOBS = ["== format", "== lint", "== build", "== backlog", "== test, bench m1, health (concurrent)"]


def test_a_job_catches():
    lines = JOBS + ["== test verify: FAILED (101) after 3.0s; output:", "failed: test verify, bench bcb"]
    assert g.classify(1, lines) == ("caught", "test", ["test verify", "bench bcb"])


def test_a_step_before_the_jobs_does_not():
    # clippy's unused-variable warning, exit code 101, six seconds
    assert g.classify(101, JOBS[:2] + ["error: unused variable: `end`"]) == ("invalid", "lint", [])
    assert g.classify(1, ["== format", "Diff in verify.rs"]) == ("invalid", "format", [])


def test_the_time_limit_does_not():
    assert g.classify(124, JOBS)[0] == "invalid"
    assert g.classify(124, JOBS + ["failed: health"])[0] == "invalid"


def test_a_passing_gate_misses():
    assert g.classify(0, JOBS + ["== tree", "all checks passed"]) == ("missed", "tree", [])


def test_rows_without_a_status_are_judged_the_same_way():
    old = {"caught": True, "exit": 1, "detected_by": ["test verify"]}
    assert g.status_of(old) == "caught"
    assert g.status_of(dict(old, detected_by=[], exit=101)) == "invalid"
    assert g.status_of(dict(old, exit=124)) == "invalid"
    assert g.status_of({"caught": False, "exit": 0, "detected_by": []}) == "missed"
    assert g.status_of(dict(old, status="invalid")) == "invalid"


def test_the_table_leaves_invalid_runs_out_of_the_rate():
    rows = [
        {"id": "a", "class": "unsound", "what": "w", "caught": True, "exit": 1, "detected_by": ["test verify"]},
        {"id": "b", "class": "unsound", "what": "w", "caught": True, "exit": 101, "detected_by": []},
        {"id": "c", "class": "miscompile", "what": "w", "caught": False, "exit": 0, "detected_by": []},
    ]
    with tempfile.NamedTemporaryFile("w", suffix=".jsonl") as f:
        f.write("".join(json.dumps(r) + "\n" for r in rows))
        f.flush()
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            g.table(type("A", (), {"results": f.name}))
    text = out.getvalue()
    assert "검출: 1/2" in text and "무효: 1" in text, text
    assert "- unsound: 1/1" in text and "- miscompile: 0/1" in text, text


def test_the_committed_record_has_no_invalid_run():
    path = Path(g.ROOT) / "dev-docs" / "measurements" / "gate-power" / "results.jsonl"
    rows = [json.loads(l) for l in path.read_text().splitlines() if l.strip()]
    assert [r["id"] for r in rows if g.status_of(r) == "invalid"] == []
    assert {r["id"] for r in rows} == {m[0] for m in g.MUTANTS}


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("test_"):
            fn()
    print("ok")
