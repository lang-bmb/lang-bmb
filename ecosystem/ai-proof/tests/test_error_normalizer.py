from orchestrator.error_normalizer import normalize_error


def test_normalize_bmb_contract_error():
    raw = "error[E001]: pre-condition violated: idx < len\n  --> solution.bmb:5:3"
    result = normalize_error(raw, lang="bmb")
    assert result["type"] == "compile_error"
    assert "pre-condition" in result["normalized"]
    assert result["location"] == "solution.bmb:5:3"
    assert result["raw"] == raw


def test_normalize_rust_borrow_error():
    raw = "error[E0505]: cannot move out of `x` because it is borrowed\n  --> solution.rs:10:5"
    result = normalize_error(raw, lang="rust")
    assert result["type"] == "compile_error"
    assert "cannot move" in result["normalized"]
    assert result["location"] == "solution.rs:10:5"


def test_normalize_test_failure():
    raw = "Test 3: expected '42', got '0'"
    result = normalize_error(raw, lang="bmb", is_test_failure=True)
    assert result["type"] == "test_failure"


def test_normalize_python_error():
    raw = 'Traceback:\n  File "solution.py", line 5\nNameError: name \'x\' is not defined'
    result = normalize_error(raw, lang="python")
    assert result["type"] == "compile_error"
