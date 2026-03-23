import json, tempfile
from pathlib import Path
from analysis.report_generator import generate_report, _load_all_records


def test_generate_report_minimal(tmp_path):
    """Generate report from minimal mock data."""
    results = tmp_path / "results"
    results.mkdir()
    raw = results / "raw" / "01_test" / "run1"
    raw.mkdir(parents=True)

    # Write minimal result
    record = {"run_id": 1, "condition": "bmb_contract", "loop_count": 2,
              "loop_types": {"A": 1, "B": 0, "C": 0, "D": 0},
              "final_correct": True, "perf_ratio": 1.01, "attempts": []}
    (raw / "bmb_contract_result.json").write_text(json.dumps(record))

    record2 = {**record, "condition": "bmb_nocontract", "loop_count": 3, "perf_ratio": 1.02}
    (raw / "bmb_nocontract_result.json").write_text(json.dumps(record2))

    # Write summary
    summary = {"date": "2026-03-24", "llm": {"model": "test"}, "total_problems": 1, "runs_per_condition": 1}
    (results / "summary.json").write_text(json.dumps(summary))

    output = tmp_path / "report" / "AI_NATIVE_PROOF.md"
    result_path = generate_report(results, output)
    assert result_path.exists()
    content = result_path.read_text()
    assert "BMB AI-Native Proof" in content
    assert "Executive Summary" in content
    assert "H1 Results" in content


def test_load_all_records(tmp_path):
    raw = tmp_path / "raw" / "01_test" / "run1"
    raw.mkdir(parents=True)
    (raw / "bmb_contract_result.json").write_text('{"condition": "bmb_contract", "loop_count": 1}')
    records = _load_all_records(tmp_path / "raw")
    assert len(records) == 1
    assert records[0]["condition"] == "bmb_contract"


def test_load_empty(tmp_path):
    records = _load_all_records(tmp_path / "nonexistent")
    assert records == []
