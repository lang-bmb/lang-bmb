from analysis.stats import wilcoxon_test, friedman_test, pairwise_wilcoxon, compute_aggregates

def test_wilcoxon_contract_fewer():
    contract = [1, 2, 1, 2, 1, 3, 1, 2]
    nocontract = [3, 4, 3, 5, 2, 4, 3, 4]
    result = wilcoxon_test(contract, nocontract)
    assert result["significant"] is True
    assert result["p_value"] < 0.05

def test_wilcoxon_equal():
    same = [2, 2, 2, 2]
    result = wilcoxon_test(same, same)
    assert result["significant"] is False

def test_wilcoxon_unequal_length():
    import pytest
    with pytest.raises(ValueError):
        wilcoxon_test([1, 2], [1, 2, 3])

def test_friedman():
    bmb = [1, 2, 1, 3, 2, 1]
    rust = [3, 4, 2, 5, 3, 2]
    python = [1, 1, 1, 2, 1, 1]
    result = friedman_test({"bmb": bmb, "rust": rust, "python": python})
    assert "statistic" in result
    assert "p_value" in result

def test_compute_aggregates():
    records = [
        {"condition": "bmb_contract", "loop_count": 2, "final_correct": True, "perf_ratio": 1.01, "loop_types": {"A": 1}},
        {"condition": "bmb_contract", "loop_count": 1, "final_correct": True, "perf_ratio": 1.02, "loop_types": {"A": 0}},
        {"condition": "rust", "loop_count": 3, "final_correct": True, "perf_ratio": 1.03, "loop_types": {"D": 2}},
    ]
    agg = compute_aggregates(records)
    assert "bmb_contract" in agg
    assert agg["bmb_contract"]["median_loops"] == 1.5
    assert agg["bmb_contract"]["correctness_rate"] == 1.0
    assert agg["rust"]["n"] == 1
